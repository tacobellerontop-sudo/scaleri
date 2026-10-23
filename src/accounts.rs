//! Minecraft accounts and Microsoft sign-in.
//!
//! Sign-in uses the OAuth device code flow, so no web view or local web server is needed:
//! Microsoft token -> Xbox Live token -> XSTS token -> Minecraft token -> Minecraft profile.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::net::{self, Progress, Result};
use crate::paths;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum AccountKind {
    Microsoft,
    Offline,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    /// Minecraft profile UUID, without dashes.
    pub uuid: String,
    pub name: String,
    pub kind: AccountKind,
    #[serde(default)]
    pub access_token: String,
    /// Unix time the Minecraft access token expires.
    #[serde(default)]
    pub expires_at: u64,
    #[serde(default)]
    pub refresh_token: String,
}

impl Account {
    pub fn offline(name: &str) -> Self {
        Self {
            uuid: offline_uuid(name),
            name: name.to_string(),
            kind: AccountKind::Offline,
            access_token: "0".into(),
            expires_at: u64::MAX,
            refresh_token: String::new(),
        }
    }

    pub fn needs_refresh(&self) -> bool {
        self.kind == AccountKind::Microsoft && now() + 300 >= self.expires_at
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Accounts {
    pub accounts: Vec<Account>,
    /// UUID of the account used to play.
    pub active: Option<String>,
}

impl Accounts {
    fn path() -> std::path::PathBuf {
        paths::data_dir().join("accounts.json")
    }

    pub fn load() -> Self {
        std::fs::read(Self::path())
            .ok()
            .and_then(|data| serde_json::from_slice(&data).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(data) = serde_json::to_vec_pretty(self) {
            let _ = paths::write_atomic(&Self::path(), &data);
        }
    }

    pub fn active(&self) -> Option<&Account> {
        let active = self.active.as_deref()?;
        self.accounts.iter().find(|a| a.uuid == active)
    }

    pub fn has_microsoft(&self) -> bool {
        self.accounts.iter().any(|a| a.kind == AccountKind::Microsoft)
    }

    /// Adds or replaces an account (matched by UUID) and makes it the active one.
    pub fn upsert(&mut self, account: Account) {
        self.active = Some(account.uuid.clone());
        match self.accounts.iter_mut().find(|a| a.uuid == account.uuid) {
            Some(existing) => *existing = account,
            None => self.accounts.push(account),
        }
    }

    pub fn remove(&mut self, uuid: &str) {
        self.accounts.retain(|a| a.uuid != uuid);
        if self.active.as_deref() == Some(uuid) {
            self.active = self.accounts.first().map(|a| a.uuid.clone());
        }
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// The UUID Minecraft itself derives for an offline player name (a version 3 UUID).
pub fn offline_uuid(name: &str) -> String {
    use md5::{Digest, Md5};
    let mut hash: [u8; 16] = Md5::digest(format!("OfflinePlayer:{name}").as_bytes()).into();
    hash[6] = (hash[6] & 0x0f) | 0x30;
    hash[8] = (hash[8] & 0x3f) | 0x80;
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn valid_player_name(name: &str) -> bool {
    (3..=16).contains(&name.len()) && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

// ---------------------------------------------------------------------------------------------
// Microsoft sign-in

const MS_DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const MS_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const MS_SCOPE: &str = "XboxLive.signin offline_access";

#[derive(Deserialize, Clone, Debug)]
pub struct DeviceCode {
    pub user_code: String,
    pub device_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    #[serde(default = "default_interval")]
    pub interval: u64,
}

fn default_interval() -> u64 {
    5
}

#[derive(Deserialize)]
struct MsToken {
    access_token: String,
    refresh_token: String,
}

pub fn start_device_code(client_id: &str) -> Result<DeviceCode> {
    let res = net::agent()
        .post(MS_DEVICE_CODE_URL)
        .send_form([("client_id", client_id), ("scope", MS_SCOPE)])
        .map_err(|e| format!("Could not reach Microsoft: {e}"))?;
    net::json_body("Starting Microsoft sign-in", res)
}

/// Waits for the user to enter the code in their browser, then signs in to Minecraft.
pub fn finish_device_code(client_id: &str, code: &DeviceCode, progress: &Progress) -> Result<Account> {
    let deadline = now() + code.expires_in;
    let mut interval = code.interval.max(1);
    let token = loop {
        for _ in 0..interval * 4 {
            progress.cancelled()?;
            std::thread::sleep(Duration::from_millis(250));
        }
        if now() > deadline {
            return Err("The sign-in code expired. Please try again.".into());
        }
        let mut res = net::agent()
            .post(MS_TOKEN_URL)
            .send_form([
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("client_id", client_id),
                ("device_code", code.device_code.as_str()),
            ])
            .map_err(|e| format!("Could not reach Microsoft: {e}"))?;
        if res.status().is_success() {
            break net::json_body::<MsToken>("Microsoft sign-in", res)?;
        }
        let body: Value = res.body_mut().read_json().unwrap_or_default();
        match body["error"].as_str().unwrap_or_default() {
            "authorization_pending" => continue,
            "slow_down" => interval += 5,
            "authorization_declined" => return Err("Sign-in was declined.".into()),
            "expired_token" => return Err("The sign-in code expired. Please try again.".into()),
            other => {
                let detail = body["error_description"].as_str().unwrap_or(other);
                return Err(format!("Microsoft sign-in failed: {detail}"));
            }
        }
    };
    progress.set_stage("Signing in to Xbox Live", 0);
    minecraft_account(&token.access_token, token.refresh_token)
}

/// Gets a fresh Minecraft token for a Microsoft account using its saved refresh token.
pub fn refresh(client_id: &str, account: &Account) -> Result<Account> {
    let res = net::agent()
        .post(MS_TOKEN_URL)
        .send_form([
            ("grant_type", "refresh_token"),
            ("client_id", client_id),
            ("refresh_token", account.refresh_token.as_str()),
            ("scope", MS_SCOPE),
        ])
        .map_err(|e| format!("Could not reach Microsoft: {e}"))?;
    let token: MsToken = net::json_body("Refreshing your Microsoft sign-in", res)
        .map_err(|e| format!("{e}\nTry removing the account and signing in again."))?;
    minecraft_account(&token.access_token, token.refresh_token)
}

fn post_json(url: &str, what: &str, body: Value) -> Result<Value> {
    let res = net::agent()
        .post(url)
        .header("Accept", "application/json")
        .send_json(body)
        .map_err(|e| format!("{what}: {e}"))?;
    if res.status() == 401 && url.contains("xsts") {
        let mut res = res;
        let body: Value = res.body_mut().read_json().unwrap_or_default();
        return Err(match body["XErr"].as_u64() {
            Some(2148916233) => {
                "This Microsoft account has no Xbox profile. Sign in at minecraft.net once to create one.".into()
            }
            Some(2148916235) => "Xbox Live isn't available in your country.".into(),
            Some(2148916236 | 2148916237) => "This account needs adult verification on xbox.com first.".into(),
            Some(2148916238) => "This is a child account. An adult needs to add it to a Microsoft family first.".into(),
            _ => format!("Xbox Live refused the sign-in: {body}"),
        });
    }
    net::json_body(what, res)
}

fn minecraft_account(ms_access_token: &str, refresh_token: String) -> Result<Account> {
    let xbl = post_json(
        "https://user.auth.xboxlive.com/user/authenticate",
        "Signing in to Xbox Live",
        json!({
            "Properties": {
                "AuthMethod": "RPS",
                "SiteName": "user.auth.xboxlive.com",
                "RpsTicket": format!("d={ms_access_token}"),
            },
            "RelyingParty": "http://auth.xboxlive.com",
            "TokenType": "JWT",
        }),
    )?;
    let xbl_token = xbl["Token"].as_str().ok_or("Xbox Live returned no token")?;

    let xsts = post_json(
        "https://xsts.auth.xboxlive.com/xsts/authorize",
        "Authorizing with Xbox Live",
        json!({
            "Properties": { "SandboxId": "RETAIL", "UserTokens": [xbl_token] },
            "RelyingParty": "rp://api.minecraftservices.com/",
            "TokenType": "JWT",
        }),
    )?;
    let xsts_token = xsts["Token"].as_str().ok_or("Xbox Live returned no XSTS token")?;
    let uhs = xsts["DisplayClaims"]["xui"][0]["uhs"]
        .as_str()
        .ok_or("Xbox Live returned no user hash")?;

    let mc = post_json(
        "https://api.minecraftservices.com/authentication/login_with_xbox",
        "Signing in to Minecraft",
        json!({ "identityToken": format!("XBL3.0 x={uhs};{xsts_token}") }),
    )?;
    let mc_token = mc["access_token"]
        .as_str()
        .ok_or("Minecraft returned no access token")?
        .to_string();
    let expires_in = mc["expires_in"].as_u64().unwrap_or(86400);

    let res = net::agent()
        .get("https://api.minecraftservices.com/minecraft/profile")
        .header("Authorization", &format!("Bearer {mc_token}"))
        .call()
        .map_err(|e| format!("Could not load your Minecraft profile: {e}"))?;
    if res.status() == 404 {
        return Err("This Microsoft account doesn't own Minecraft: Java Edition.".into());
    }
    let profile: Value = net::json_body("Loading your Minecraft profile", res)?;
    Ok(Account {
        uuid: profile["id"]
            .as_str()
            .ok_or("Your Minecraft profile has no ID")?
            .to_string(),
        name: profile["name"].as_str().unwrap_or("Player").to_string(),
        kind: AccountKind::Microsoft,
        access_token: mc_token,
        expires_at: now() + expires_in,
        refresh_token,
    })
}

// ---------------------------------------------------------------------------------------------
// Skins

/// The player's face from their skin, with the hat layer on top, as 8×8 RGBA pixels.
/// The skin is cached so the face still shows when offline.
pub fn fetch_face(uuid: &str) -> Result<Vec<u8>> {
    let cache = paths::data_dir().join("skins").join(format!("{uuid}.png"));
    let skin = match download_skin(uuid) {
        Ok(png) => {
            let _ = paths::write_atomic(&cache, &png);
            png
        }
        Err(e) => std::fs::read(&cache).map_err(|_| e)?,
    };
    face_from_skin(&skin)
}

fn download_skin(uuid: &str) -> Result<Vec<u8>> {
    use base64::Engine;
    let profile: Value = net::get_json(&format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{uuid}"
    ))?;
    let encoded = profile["properties"]
        .as_array()
        .and_then(|props| props.iter().find(|p| p["name"] == "textures"))
        .and_then(|p| p["value"].as_str())
        .ok_or("This profile has no skin")?;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| format!("Unreadable skin data: {e}"))?;
    let textures: Value = serde_json::from_slice(&decoded).map_err(|e| format!("Unreadable skin data: {e}"))?;
    let url = textures["textures"]["SKIN"]["url"]
        .as_str()
        .ok_or("This profile uses the default skin")?
        .replace("http://", "https://");
    let mut res = net::agent()
        .get(&url)
        .call()
        .map_err(|e| format!("Could not download the skin: {e}"))?;
    res.body_mut()
        .read_to_vec()
        .map_err(|e| format!("Could not download the skin: {e}"))
}

fn face_from_skin(png: &[u8]) -> Result<Vec<u8>> {
    let skin = image::load_from_memory(png)
        .map_err(|e| format!("Unreadable skin: {e}"))?
        .to_rgba8();
    if skin.width() < 64 || skin.height() < 32 {
        return Err("Unexpected skin size".into());
    }
    let mut face = Vec::with_capacity(8 * 8 * 4);
    for y in 0..8 {
        for x in 0..8 {
            let base = skin.get_pixel(8 + x, 8 + y).0;
            let hat = skin.get_pixel(40 + x, 8 + y).0;
            face.extend_from_slice(if hat[3] > 0 { &hat } else { &base });
        }
    }
    Ok(face)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_uuid_matches_minecraft() {
        // The UUID the vanilla server gives the offline player "Notch"
        assert_eq!(offline_uuid("Notch"), "b50ad385829d3141a2167e7d7539ba7f");
    }

    #[test]
    fn face_uses_the_hat_layer_on_top() {
        let mut skin = image::RgbaImage::new(64, 64);
        skin.put_pixel(8, 8, image::Rgba([10, 20, 30, 255]));
        skin.put_pixel(9, 8, image::Rgba([1, 2, 3, 255]));
        skin.put_pixel(41, 8, image::Rgba([200, 0, 0, 255]));
        let mut png = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut png);
        skin.write_to(&mut cursor, image::ImageFormat::Png).unwrap();
        let face = face_from_skin(&png).unwrap();
        assert_eq!(&face[0..4], &[10, 20, 30, 255]);
        assert_eq!(&face[4..8], &[200, 0, 0, 255]);
    }

    #[test]
    fn player_names() {
        assert!(valid_player_name("Steve_01"));
        assert!(!valid_player_name("no"));
        assert!(!valid_player_name("has space"));
    }
}
