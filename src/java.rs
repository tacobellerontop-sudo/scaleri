//! The Java runtime each Minecraft version needs, downloaded from Mojang the same way the
//! official launcher does, so players never have to install Java themselves.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Deserialize;
use serde_json::Value;

use crate::net::{self, Download, Progress, Result};
use crate::paths;

const RUNTIMES_URL: &str =
    "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

fn platform() -> &'static str {
    match (cfg!(windows), std::env::consts::ARCH) {
        (true, "aarch64") => "windows-arm64",
        (true, "x86") => "windows-x86",
        (true, _) => "windows-x64",
        (false, "aarch64") if cfg!(target_os = "linux") => "linux-arm64",
        (false, _) if cfg!(target_os = "macos") => "mac-os",
        _ => "linux",
    }
}

#[derive(Deserialize)]
struct RuntimeManifest {
    files: HashMap<String, RuntimeFile>,
}

#[derive(Deserialize)]
struct RuntimeFile {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    executable: bool,
    #[serde(default)]
    downloads: Option<RuntimeDownloads>,
}

#[derive(Deserialize)]
struct RuntimeDownloads {
    raw: RawFile,
}

#[derive(Deserialize)]
struct RawFile {
    url: String,
    sha1: String,
    size: u64,
}

fn runtime_dir(component: &str) -> PathBuf {
    paths::runtimes_dir().join(component)
}

/// The Java executable inside a runtime folder. On Windows this is `javaw.exe`, which runs
/// without opening a console window.
pub fn java_binary(home: &Path) -> PathBuf {
    if cfg!(windows) {
        home.join("bin").join("javaw.exe")
    } else if cfg!(target_os = "macos") {
        home.join("jre.bundle/Contents/Home/bin/java")
    } else {
        home.join("bin").join("java")
    }
}

/// Makes sure the runtime `component` (e.g. `java-runtime-delta`) is installed and returns its
/// Java executable.
pub fn ensure_runtime(component: &str, progress: &Arc<Progress>) -> Result<PathBuf> {
    let home = runtime_dir(component);
    let marker = home.join(".scaleri-complete");
    let java = java_binary(&home);
    if marker.exists() && java.exists() {
        return Ok(java);
    }

    progress.set_stage("Finding Java", 0);
    let all: Value = net::get_json(RUNTIMES_URL)?;
    let entry = &all[platform()][component][0];
    let url = entry["manifest"]["url"]
        .as_str()
        .ok_or_else(|| format!("Mojang has no {component} Java runtime for {}", platform()))?;
    let manifest: RuntimeManifest = net::get_json(url)?;

    let mut items = Vec::new();
    for (path, file) in &manifest.files {
        let target = home.join(path);
        match file.kind.as_str() {
            "directory" => {
                std::fs::create_dir_all(&target).map_err(|e| format!("Could not create {}: {e}", target.display()))?;
            }
            "file" => {
                if let Some(downloads) = &file.downloads {
                    items.push(Download {
                        url: downloads.raw.url.clone(),
                        path: target,
                        sha1: Some(downloads.raw.sha1.clone()),
                        size: Some(downloads.raw.size),
                        executable: file.executable,
                    });
                }
            }
            // Symlinks only appear in the macOS runtimes
            _ => {}
        }
    }
    net::download_all("Downloading Java", items, false, progress)?;
    let _ = std::fs::write(&marker, component);
    if !java.exists() {
        return Err(format!("The downloaded Java runtime has no {}", java.display()));
    }
    Ok(java)
}
