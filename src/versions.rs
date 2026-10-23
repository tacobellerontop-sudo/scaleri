//! Minecraft version metadata from Mojang: the version list, each version's JSON, the
//! library and argument rules inside it, and the files a version needs.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;

use crate::net::{self, Download, Result};
use crate::paths;

pub const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const RESOURCES_URL: &str = "https://resources.download.minecraft.net";

#[derive(Deserialize, Clone, Debug)]
pub struct Manifest {
    pub latest: Latest,
    pub versions: Vec<VersionEntry>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Latest {
    pub release: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    #[serde(default)]
    pub sha1: Option<String>,
}

impl VersionEntry {
    pub fn is_release(&self) -> bool {
        self.kind == "release"
    }
}

/// Loads the version list, falling back to the last copy saved on disk when offline.
pub fn load_manifest() -> Result<Manifest> {
    let cache = paths::versions_dir().join("version_manifest_v2.json");
    match net::get_json::<Value>(MANIFEST_URL) {
        Ok(value) => {
            if let Ok(data) = serde_json::to_vec(&value) {
                let _ = paths::write_atomic(&cache, &data);
            }
            serde_json::from_value(value).map_err(|e| format!("Mojang's version list looks wrong: {e}"))
        }
        Err(err) => std::fs::read(&cache)
            .ok()
            .and_then(|data| serde_json::from_slice(&data).ok())
            .ok_or(err),
    }
}

/// The parts of a version JSON (`versions/<id>/<id>.json`) Scaleri uses.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub id: String,
    #[serde(rename = "type", default)]
    pub kind: String,
    pub main_class: String,
    #[serde(default)]
    pub arguments: Option<Arguments>,
    /// Pre-1.13 versions give the game arguments as one string.
    #[serde(default)]
    pub minecraft_arguments: Option<String>,
    pub asset_index: AssetIndexRef,
    pub downloads: HashMap<String, FileRef>,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub java_version: Option<JavaVersion>,
    #[serde(default)]
    pub logging: Option<Logging>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<Value>,
    #[serde(default)]
    pub jvm: Vec<Value>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct AssetIndexRef {
    pub id: String,
    pub url: String,
    pub sha1: String,
    #[serde(default)]
    pub size: Option<u64>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct FileRef {
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    pub url: String,
    pub sha1: String,
    #[serde(default)]
    pub size: Option<u64>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Logging {
    #[serde(default)]
    pub client: Option<LoggingClient>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LoggingClient {
    pub argument: String,
    pub file: FileRef,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Library {
    pub name: String,
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub rules: Option<Vec<Rule>>,
    /// Old-style natives: OS name to classifier, e.g. `"windows": "natives-windows-${arch}"`.
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
    #[serde(default)]
    pub url: Option<String>,
    /// Checksum and size given by mod loader profiles, which have no `downloads` section.
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LibraryDownloads {
    #[serde(default)]
    pub artifact: Option<FileRef>,
    #[serde(default)]
    pub classifiers: Option<HashMap<String, FileRef>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Rule {
    pub action: String,
    #[serde(default)]
    pub os: Option<OsRule>,
    #[serde(default)]
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct OsRule {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub arch: Option<String>,
}

/// Mojang's name for the OS Scaleri runs on.
pub fn os_name() -> &'static str {
    if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

fn os_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86" => "x86",
        "aarch64" => "arm64",
        _ => "x86_64",
    }
}

/// The classifier of 1.19+ native jars built for this platform.
fn native_classifier() -> &'static str {
    match (os_name(), os_arch()) {
        ("windows", "arm64") => "natives-windows-arm64",
        ("windows", "x86") => "natives-windows-x86",
        ("windows", _) => "natives-windows",
        ("osx", "arm64") => "natives-macos-arm64",
        ("osx", _) => "natives-macos",
        (_, "arm64") => "natives-linux-aarch64",
        _ => "natives-linux",
    }
}

fn rule_matches(rule: &Rule) -> bool {
    if let Some(os) = &rule.os {
        if os.name.as_deref().is_some_and(|n| n != os_name()) {
            return false;
        }
        if os.arch.as_deref().is_some_and(|a| a != os_arch()) {
            return false;
        }
    }
    // Scaleri turns on none of the optional features (demo mode, quick play, custom resolution
    // through features), so a rule that needs one never applies.
    if let Some(features) = &rule.features {
        if features.values().any(|wanted| *wanted) {
            return false;
        }
    }
    true
}

/// Mojang's rule logic: with no rules everything is allowed, otherwise the last matching rule wins.
pub fn rules_allow(rules: Option<&[Rule]>) -> bool {
    let Some(rules) = rules else {
        return true;
    };
    let mut allowed = false;
    for rule in rules {
        if rule_matches(rule) {
            allowed = rule.action == "allow";
        }
    }
    allowed
}

impl Library {
    fn applies(&self) -> bool {
        rules_allow(self.rules.as_deref())
    }

    /// Libraries whose classifier is a native jar for another platform (1.19+ style).
    fn is_foreign_native(&self) -> bool {
        let Some(classifier) = self.name.split(':').nth(3) else {
            return false;
        };
        classifier.starts_with("natives-") && classifier != native_classifier()
    }

    pub fn is_native_jar(&self) -> bool {
        self.name.split(':').nth(3).is_some_and(|c| c.starts_with("natives-"))
    }

    fn maven_path(&self) -> Option<String> {
        let parts: Vec<&str> = self.name.split(':').collect();
        if parts.len() < 3 {
            return None;
        }
        let (group, artifact, version) = (parts[0].replace('.', "/"), parts[1], parts[2]);
        let classifier = parts.get(3).map(|c| format!("-{c}")).unwrap_or_default();
        Some(format!(
            "{group}/{artifact}/{version}/{artifact}-{version}{classifier}.jar"
        ))
    }

    /// The jar that goes on the classpath.
    pub fn artifact(&self) -> Option<Download> {
        if let Some(artifact) = self.downloads.as_ref().and_then(|d| d.artifact.as_ref()) {
            let path = artifact.path.clone().or_else(|| self.maven_path())?;
            return Some(download(&artifact.url, paths::libraries_dir().join(path), artifact));
        }
        // Libraries without download info (some old or third-party versions) use a Maven repository
        if self.natives.is_some() {
            return None;
        }
        let path = self.maven_path()?;
        let base = self
            .url
            .clone()
            .unwrap_or_else(|| "https://libraries.minecraft.net/".into());
        Some(Download {
            url: format!("{}/{}", base.trim_end_matches('/'), path),
            path: paths::libraries_dir().join(path),
            sha1: self.sha1.clone(),
            size: self.size,
            executable: false,
        })
    }

    /// Old-style native jar to extract for this platform, if any.
    pub fn native(&self) -> Option<Download> {
        let classifier = self.natives.as_ref()?.get(os_name())?;
        let bits = if cfg!(target_pointer_width = "64") { "64" } else { "32" };
        let classifier = classifier.replace("${arch}", bits);
        let file = self.downloads.as_ref()?.classifiers.as_ref()?.get(&classifier)?;
        let path = file.path.clone()?;
        Some(download(&file.url, paths::libraries_dir().join(path), file))
    }
}

fn download(url: &str, path: PathBuf, file: &FileRef) -> Download {
    Download {
        url: url.to_string(),
        path,
        sha1: Some(file.sha1.clone()),
        size: file.size,
        executable: false,
    }
}

impl Version {
    pub fn json_path(id: &str) -> PathBuf {
        paths::versions_dir().join(id).join(format!("{id}.json"))
    }

    pub fn client_jar_path(&self) -> PathBuf {
        paths::versions_dir().join(&self.id).join(format!("{}.jar", self.id))
    }

    /// Loads a version's JSON, downloading it first if needed.
    pub fn fetch(entry: &VersionEntry) -> Result<Version> {
        let path = Self::json_path(&entry.id);
        let item = Download {
            url: entry.url.clone(),
            path: path.clone(),
            sha1: entry.sha1.clone(),
            size: None,
            executable: false,
        };
        if !item.is_current(true) {
            item.fetch()?;
        }
        Self::load(&entry.id)
    }

    pub fn load(id: &str) -> Result<Version> {
        let data =
            std::fs::read(Self::json_path(id)).map_err(|e| format!("Minecraft {id} isn't downloaded yet ({e})"))?;
        serde_json::from_slice(&data).map_err(|e| format!("The files for Minecraft {id} look damaged: {e}"))
    }

    /// Libraries that apply to this platform.
    pub fn active_libraries(&self) -> impl Iterator<Item = &Library> {
        self.libraries.iter().filter(|l| l.applies() && !l.is_foreign_native())
    }

    pub fn client_download(&self) -> Result<Download> {
        let client = self
            .downloads
            .get("client")
            .ok_or("This version has no client download")?;
        Ok(download(&client.url, self.client_jar_path(), client))
    }

    pub fn logging_download(&self) -> Option<(String, Download)> {
        let client = self.logging.as_ref()?.client.as_ref()?;
        let id = client.file.id.clone()?;
        let path = paths::assets_dir().join("log_configs").join(&id);
        Some((client.argument.clone(), download(&client.file.url, path, &client.file)))
    }

    pub fn java_component(&self) -> (String, u32) {
        match &self.java_version {
            Some(java) => (java.component.clone(), java.major_version),
            None => ("jre-legacy".into(), 8),
        }
    }
}

/// An asset index (`assets/indexes/<id>.json`).
#[derive(Deserialize, Debug)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
    #[serde(default, rename = "virtual")]
    pub is_virtual: bool,
    #[serde(default)]
    pub map_to_resources: bool,
}

#[derive(Deserialize, Debug)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

impl AssetObject {
    pub fn object_path(&self) -> PathBuf {
        paths::assets_dir()
            .join("objects")
            .join(&self.hash[..2])
            .join(&self.hash)
    }

    pub fn download(&self) -> Download {
        Download {
            url: format!("{RESOURCES_URL}/{}/{}", &self.hash[..2], self.hash),
            path: self.object_path(),
            sha1: Some(self.hash.clone()),
            size: Some(self.size),
            executable: false,
        }
    }
}

pub fn fetch_asset_index(version: &Version) -> Result<AssetIndex> {
    let index = &version.asset_index;
    let path = paths::assets_dir().join("indexes").join(format!("{}.json", index.id));
    let item = Download {
        url: index.url.clone(),
        path: path.clone(),
        sha1: Some(index.sha1.clone()),
        size: index.size,
        executable: false,
    };
    if !item.is_current(true) {
        item.fetch()?;
    }
    let data = std::fs::read(&path).map_err(|e| format!("Could not read the asset index: {e}"))?;
    serde_json::from_slice(&data).map_err(|e| format!("The asset index looks damaged: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(action: &str, os: Option<&str>) -> Rule {
        Rule {
            action: action.into(),
            os: os.map(|n| OsRule {
                name: Some(n.into()),
                arch: None,
            }),
            features: None,
        }
    }

    #[test]
    fn last_matching_rule_wins() {
        assert!(rules_allow(None));
        assert!(rules_allow(Some(&[rule("allow", None)])));
        let not_here = if os_name() == "osx" { "linux" } else { "osx" };
        assert!(rules_allow(Some(&[
            rule("allow", None),
            rule("disallow", Some(not_here))
        ])));
        assert!(!rules_allow(Some(&[
            rule("allow", None),
            rule("disallow", Some(os_name()))
        ])));
        assert!(!rules_allow(Some(&[rule("allow", Some(not_here))])));
    }

    #[test]
    fn feature_rules_are_off() {
        let mut features = HashMap::new();
        features.insert("is_demo_user".to_string(), true);
        let r = Rule {
            action: "allow".into(),
            os: None,
            features: Some(features),
        };
        assert!(!rules_allow(Some(&[r])));
    }

    #[test]
    fn maven_paths() {
        let lib = Library {
            name: "org.lwjgl:lwjgl:3.3.3:natives-windows".into(),
            downloads: None,
            rules: None,
            natives: None,
            url: None,
            sha1: None,
            size: None,
        };
        assert_eq!(
            lib.maven_path().unwrap(),
            "org/lwjgl/lwjgl/3.3.3/lwjgl-3.3.3-natives-windows.jar"
        );
        assert!(lib.is_native_jar());
    }
}
