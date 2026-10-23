//! Launcher settings, saved to `settings.json`.

use serde::{Deserialize, Serialize};

use crate::paths;

/// Scaleri's own Azure application, approved by Microsoft for the Minecraft services.
const SCALERI_CLIENT_ID: &str = "174db889-6412-4846-869e-faafd5a578d0";

/// The Azure application used for Microsoft sign-in. Builds can swap in another one with the
/// `SCALERI_MSA_CLIENT_ID` environment variable; users can also paste one into Settings.
pub fn built_in_client_id() -> &'static str {
    match option_env!("SCALERI_MSA_CLIENT_ID").map(str::trim) {
        Some(id) if !id.is_empty() => id,
        _ => SCALERI_CLIENT_ID,
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(default)]
pub struct Settings {
    /// Maximum Java heap, in megabytes.
    pub memory_mb: u32,
    /// Extra JVM arguments, separated by spaces.
    pub java_args: String,
    /// Use this Java instead of the runtime Scaleri downloads from Mojang.
    pub java_path: String,
    /// Close Scaleri once the game is running, so nothing stays open in the background.
    pub close_on_launch: bool,
    /// Microsoft sign-in application (client) ID.
    pub msa_client_id: String,
    pub game_width: u32,
    pub game_height: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            memory_mb: 4096,
            java_args: String::new(),
            java_path: String::new(),
            close_on_launch: true,
            msa_client_id: String::new(),
            game_width: 1280,
            game_height: 720,
        }
    }
}

impl Settings {
    fn path() -> std::path::PathBuf {
        paths::data_dir().join("settings.json")
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

    /// The client ID to sign in with: the one pasted into Settings, or Scaleri's own.
    pub fn client_id(&self) -> String {
        let id = self.msa_client_id.trim();
        if id.is_empty() { built_in_client_id() } else { id }.to_string()
    }
}
