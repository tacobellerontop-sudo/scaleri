//! Fabric and Quilt mod loaders. Both publish a "profile" for each game and loader version: a
//! partial version file that names the loader's main class, libraries and extra arguments, which
//! Scaleri layers on top of the vanilla version.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::net::{self, Download, Result};
use crate::paths;
use crate::versions::{Arguments, Library, Version};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoaderKind {
    Fabric,
    Quilt,
}

impl LoaderKind {
    pub const ALL: [LoaderKind; 2] = [LoaderKind::Fabric, LoaderKind::Quilt];

    pub fn name(self) -> &'static str {
        match self {
            LoaderKind::Fabric => "Fabric",
            LoaderKind::Quilt => "Quilt",
        }
    }

    fn meta(self) -> &'static str {
        match self {
            LoaderKind::Fabric => "https://meta.fabricmc.net/v2",
            LoaderKind::Quilt => "https://meta.quiltmc.org/v3",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Loader {
    pub kind: LoaderKind,
    /// Loader version; empty means "the latest stable one", filled in on first launch.
    #[serde(default)]
    pub version: String,
}

/// The newest stable loader version that supports `game`.
pub fn latest_version(kind: LoaderKind, game: &str) -> Result<String> {
    let list: Vec<Value> = net::get_json(&format!("{}/versions/loader/{game}", kind.meta()))?;
    let versions = list.iter().filter_map(|entry| {
        let loader = &entry["loader"];
        let version = loader["version"].as_str()?;
        Some((version, loader["stable"].as_bool()))
    });
    let mut first = None;
    for (version, stable) in versions {
        first.get_or_insert(version);
        // Fabric marks stable builds; Quilt marks pre-releases in the version name
        let is_stable = stable.unwrap_or_else(|| !version.contains("beta") && !version.contains("pre"));
        if is_stable {
            return Ok(version.to_string());
        }
    }
    first
        .map(String::from)
        .ok_or_else(|| format!("{} doesn't support Minecraft {game} yet", kind.name()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Profile {
    main_class: String,
    #[serde(default)]
    arguments: Option<Arguments>,
    #[serde(default)]
    libraries: Vec<Library>,
}

/// Adds the loader to the vanilla version: its main class, its libraries in front of the game's,
/// and its extra arguments.
pub fn apply(vanilla: Version, loader: &Loader) -> Result<Version> {
    let id = format!(
        "{}-{}-{}",
        loader.kind.name().to_lowercase(),
        loader.version,
        vanilla.id
    );
    let path = paths::versions_dir().join(&id).join(format!("{id}.json"));
    if !path.exists() {
        let url = format!(
            "{}/versions/loader/{}/{}/profile/json",
            loader.kind.meta(),
            vanilla.id,
            loader.version
        );
        Download {
            url,
            path: path.clone(),
            sha1: None,
            size: None,
            executable: false,
        }
        .fetch()?;
    }
    let data = std::fs::read(&path).map_err(|e| format!("Could not read the {} profile: {e}", loader.kind.name()))?;
    let profile: Profile = serde_json::from_slice(&data).map_err(|e| {
        let _ = std::fs::remove_file(&path);
        format!("The {} profile looks damaged: {e}", loader.kind.name())
    })?;
    Ok(merge(vanilla, profile))
}

fn merge(mut version: Version, profile: Profile) -> Version {
    let Some(arguments) = version.arguments.as_mut() else {
        // Pre-1.13 versions have no argument lists; the loaders don't support them anyway
        version.main_class = profile.main_class;
        version.libraries.splice(0..0, profile.libraries);
        return version;
    };
    if let Some(extra) = profile.arguments {
        arguments.jvm.extend(extra.jvm);
        arguments.game.extend(extra.game);
    }
    // The loader's copy of a library (e.g. ASM) replaces the game's
    let key = |name: &str| name.split(':').take(2).collect::<Vec<_>>().join(":");
    let ours: std::collections::HashSet<String> = profile.libraries.iter().map(|l| key(&l.name)).collect();
    version
        .libraries
        .retain(|l| l.is_native_jar() || !ours.contains(&key(&l.name)));
    version.libraries.splice(0..0, profile.libraries);
    version.main_class = profile.main_class;
    version
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_a_fabric_profile() {
        let vanilla: Version = serde_json::from_str(
            r#"{"id":"1.21.4","type":"release","mainClass":"net.minecraft.client.main.Main",
                "arguments":{"game":["--username"],"jvm":["-cp","${classpath}"]},
                "assetIndex":{"id":"19","url":"u","sha1":"s"},
                "downloads":{"client":{"url":"u","sha1":"s"}},
                "libraries":[{"name":"org.ow2.asm:asm:9.0"},{"name":"com.mojang:brigadier:1.0"}]}"#,
        )
        .unwrap();
        let profile: Profile = serde_json::from_str(
            r#"{"mainClass":"net.fabricmc.loader.impl.launch.knot.KnotClient",
                "arguments":{"game":[],"jvm":["-DFabricMcEmu= net.minecraft.client.main.Main "]},
                "libraries":[{"name":"org.ow2.asm:asm:9.7","url":"https://maven.fabricmc.net/"},
                             {"name":"net.fabricmc:fabric-loader:0.16.9","url":"https://maven.fabricmc.net/"}]}"#,
        )
        .unwrap();
        let merged = merge(vanilla, profile);
        assert_eq!(merged.main_class, "net.fabricmc.loader.impl.launch.knot.KnotClient");
        let names: Vec<&str> = merged.libraries.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "org.ow2.asm:asm:9.7",
                "net.fabricmc:fabric-loader:0.16.9",
                "com.mojang:brigadier:1.0"
            ]
        );
        assert_eq!(merged.arguments.unwrap().jvm.len(), 3);
        let fabric = &merged.libraries[1].artifact().unwrap();
        assert_eq!(
            fabric.url,
            "https://maven.fabricmc.net/net/fabricmc/fabric-loader/0.16.9/fabric-loader-0.16.9.jar"
        );
    }
}
