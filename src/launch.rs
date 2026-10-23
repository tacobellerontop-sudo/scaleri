//! Getting an instance ready and starting Minecraft.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;

use serde_json::Value;

use crate::accounts::{Account, AccountKind};
use crate::instances::Instance;
use crate::net::{self, Progress, Result};
use crate::settings::Settings;
use crate::versions::{self, AssetIndex, Version};
use crate::{java, loaders, paths};

/// Loads the version JSON, downloading it (and the version list) only if it's missing.
pub fn resolve_version(id: &str) -> Result<Version> {
    if let Ok(version) = Version::load(id) {
        return Ok(version);
    }
    let manifest = versions::load_manifest()?;
    let entry = manifest
        .versions
        .iter()
        .find(|v| v.id == id)
        .ok_or_else(|| format!("Mojang doesn't list a Minecraft version called {id}"))?;
    Version::fetch(entry)
}

/// Picks the latest stable loader version for an instance that hasn't got one yet.
/// Returns true when the instance changed and should be saved.
pub fn pin_loader_version(instance: &mut Instance, progress: &Progress) -> Result<bool> {
    let Some(loader) = instance.loader.as_mut() else {
        return Ok(false);
    };
    if !loader.version.is_empty() {
        return Ok(false);
    }
    progress.set_stage(format!("Finding the latest {}", loader.kind.name()), 0);
    loader.version = loaders::latest_version(loader.kind, &instance.version)?;
    Ok(true)
}

/// Downloads everything the instance needs and starts the game. Returns the running game process.
pub fn launch(instance: &Instance, account: &Account, settings: &Settings, progress: &Arc<Progress>) -> Result<Child> {
    progress.set_stage(format!("Loading Minecraft {}", instance.version), 0);
    let mut version = resolve_version(&instance.version)?;
    if let Some(loader) = &instance.loader {
        progress.set_stage(format!("Loading {} {}", loader.kind.name(), loader.version), 0);
        version = loaders::apply(version, loader)?;
    }
    progress.cancelled()?;

    // Java
    let java = if settings.java_path.trim().is_empty() {
        let (component, _) = version.java_component();
        java::ensure_runtime(&component, progress)?
    } else {
        PathBuf::from(settings.java_path.trim())
    };
    progress.cancelled()?;

    // Game files and libraries
    let mut files = vec![version.client_download()?];
    let mut natives = Vec::new();
    for lib in version.active_libraries() {
        if let Some(artifact) = lib.artifact() {
            if lib.is_native_jar() {
                natives.push(artifact.path.clone());
            }
            files.push(artifact);
        }
        if let Some(native) = lib.native() {
            natives.push(native.path.clone());
            files.push(native);
        }
    }
    let logging = version.logging_download();
    if let Some((_, file)) = &logging {
        files.push(file.clone());
    }
    net::download_all("Downloading game files", files, true, progress)?;

    // Assets: sounds, languages and textures shared by all instances
    progress.set_stage("Checking assets", 0);
    let index = versions::fetch_asset_index(&version)?;
    let assets: Vec<_> = index.objects.values().map(|o| o.download()).collect();
    net::download_all("Downloading assets", assets, false, progress)?;
    let game_assets = prepare_legacy_assets(&version, &index, instance)?;

    // Native libraries (LWJGL and friends) are unpacked fresh for every launch
    progress.set_stage("Preparing native libraries", 0);
    let natives_dir = instance.dir().join("natives");
    let _ = std::fs::remove_dir_all(&natives_dir);
    std::fs::create_dir_all(&natives_dir).map_err(|e| format!("Could not create {}: {e}", natives_dir.display()))?;
    for jar in &natives {
        extract_natives(jar, &natives_dir)?;
    }

    // Classpath
    let mut seen = HashSet::new();
    let mut classpath: Vec<PathBuf> = Vec::new();
    for lib in version.active_libraries() {
        if let Some(artifact) = lib.artifact() {
            if seen.insert(artifact.path.clone()) {
                classpath.push(artifact.path);
            }
        }
    }
    classpath.push(version.client_jar_path());
    let separator = if cfg!(windows) { ";" } else { ":" };
    let classpath = classpath
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(separator);

    let game_dir = instance.game_dir();
    std::fs::create_dir_all(&game_dir).map_err(|e| format!("Could not create {}: {e}", game_dir.display()))?;

    let mut vars: HashMap<&str, String> = HashMap::new();
    vars.insert("auth_player_name", account.name.clone());
    vars.insert("auth_uuid", account.uuid.clone());
    vars.insert("auth_access_token", account.access_token.clone());
    vars.insert(
        "auth_session",
        format!("token:{}:{}", account.access_token, account.uuid),
    );
    vars.insert("auth_xuid", "0".into());
    vars.insert("clientid", settings.client_id());
    vars.insert(
        "user_type",
        if account.kind == AccountKind::Microsoft {
            "msa"
        } else {
            "legacy"
        }
        .into(),
    );
    vars.insert("user_properties", "{}".into());
    vars.insert("version_name", version.id.clone());
    vars.insert(
        "version_type",
        if version.kind.is_empty() {
            "release".into()
        } else {
            version.kind.clone()
        },
    );
    vars.insert("game_directory", game_dir.to_string_lossy().into_owned());
    vars.insert("assets_root", paths::assets_dir().to_string_lossy().into_owned());
    vars.insert("game_assets", game_assets.to_string_lossy().into_owned());
    vars.insert("assets_index_name", version.asset_index.id.clone());
    vars.insert("natives_directory", natives_dir.to_string_lossy().into_owned());
    vars.insert(
        "library_directory",
        paths::libraries_dir().to_string_lossy().into_owned(),
    );
    vars.insert("classpath_separator", separator.into());
    vars.insert("classpath", classpath);
    vars.insert("launcher_name", "scaleri".into());
    vars.insert("launcher_version", env!("CARGO_PKG_VERSION").into());

    let memory = instance.memory_mb.unwrap_or(settings.memory_mb).max(512);
    let mut args: Vec<String> = vec![format!("-Xmx{memory}M"), "-Xms512M".into()];
    args.extend(settings.java_args.split_whitespace().map(String::from));

    match &version.arguments {
        Some(arguments) => args.extend(expand(&arguments.jvm, &vars)),
        None => {
            args.push(substitute("-Djava.library.path=${natives_directory}", &vars));
            args.push("-cp".into());
            args.push(vars["classpath"].clone());
        }
    }
    if let Some((argument, file)) = &logging {
        args.push(argument.replace("${path}", &file.path.to_string_lossy()));
    }
    args.push(version.main_class.clone());
    match (&version.arguments, &version.minecraft_arguments) {
        (Some(arguments), _) => args.extend(expand(&arguments.game, &vars)),
        (None, Some(legacy)) => args.extend(legacy.split_whitespace().map(|a| substitute(a, &vars))),
        (None, None) => {}
    }
    args.extend([
        "--width".into(),
        settings.game_width.to_string(),
        "--height".into(),
        settings.game_height.to_string(),
    ]);
    args.extend(join_server_args(&version, &instance.server));

    progress.set_stage("Starting Minecraft", 0);
    let log = std::fs::File::create(instance.log_path()).map_err(|e| format!("Could not create the game log: {e}"))?;
    let log_err = log
        .try_clone()
        .map_err(|e| format!("Could not create the game log: {e}"))?;
    let mut command = Command::new(&java);
    command
        .args(&args)
        .current_dir(&game_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_err));
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Keep the game running on its own when Scaleri closes
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    command
        .spawn()
        .map_err(|e| format!("Could not start Java ({}): {e}", java.display()))
}

/// Arguments that make the game join `server` straight away. 1.20 and newer use Quick Play;
/// older versions take `--server` and `--port`.
fn join_server_args(version: &Version, server: &str) -> Vec<String> {
    let server = server.trim();
    if server.is_empty() {
        return Vec::new();
    }
    let quick_play = version
        .arguments
        .as_ref()
        .is_some_and(|a| a.game.iter().any(|v| v.to_string().contains("quickPlayMultiplayer")));
    if quick_play {
        return vec!["--quickPlayMultiplayer".into(), server.into()];
    }
    let (host, port) = match server.rsplit_once(':') {
        Some((host, port)) if port.parse::<u16>().is_ok() => (host, port),
        _ => (server, "25565"),
    };
    vec!["--server".into(), host.into(), "--port".into(), port.into()]
}

fn substitute(arg: &str, vars: &HashMap<&str, String>) -> String {
    let mut out = String::with_capacity(arg.len());
    let mut rest = arg;
    while let Some(start) = rest.find("${") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        match after.find('}') {
            Some(end) => {
                let key = &after[..end];
                match vars.get(key) {
                    Some(value) => out.push_str(value),
                    None => {
                        out.push_str("${");
                        out.push_str(key);
                        out.push('}');
                    }
                }
                rest = &after[end + 1..];
            }
            None => {
                out.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// Expands the modern `arguments.game` / `arguments.jvm` lists, applying their rules.
fn expand(list: &[Value], vars: &HashMap<&str, String>) -> Vec<String> {
    let mut out = Vec::new();
    for item in list {
        match item {
            Value::String(arg) => out.push(substitute(arg, vars)),
            Value::Object(obj) => {
                let rules: Option<Vec<versions::Rule>> =
                    obj.get("rules").and_then(|r| serde_json::from_value(r.clone()).ok());
                if !versions::rules_allow(rules.as_deref()) {
                    continue;
                }
                match obj.get("value") {
                    Some(Value::String(arg)) => out.push(substitute(arg, vars)),
                    Some(Value::Array(args)) => {
                        out.extend(args.iter().filter_map(Value::as_str).map(|a| substitute(a, vars)))
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    out
}

fn extract_natives(jar: &Path, dest: &Path) -> Result<()> {
    let file = std::fs::File::open(jar).map_err(|e| format!("Could not open {}: {e}", jar.display()))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("{} is damaged: {e}", jar.display()))?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("{} is damaged: {e}", jar.display()))?;
        if entry.is_dir() {
            continue;
        }
        let Some(name) = entry.enclosed_name() else {
            continue;
        };
        if name.starts_with("META-INF") {
            continue;
        }
        let is_native = name
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| matches!(e, "dll" | "so" | "dylib" | "jnilib"));
        if !is_native {
            continue;
        }
        // Flatten: LWJGL 3 jars keep their libraries in per-platform folders
        let Some(file_name) = name.file_name() else {
            continue;
        };
        let target = dest.join(file_name);
        let mut out =
            std::fs::File::create(&target).map_err(|e| format!("Could not write {}: {e}", target.display()))?;
        std::io::copy(&mut entry, &mut out).map_err(|e| format!("Could not unpack {}: {e}", jar.display()))?;
    }
    Ok(())
}

/// Versions before 1.7.10 read assets from a plain folder tree instead of the hashed object store.
fn prepare_legacy_assets(version: &Version, index: &AssetIndex, instance: &Instance) -> Result<PathBuf> {
    let target = if index.map_to_resources {
        instance.game_dir().join("resources")
    } else if index.is_virtual {
        paths::assets_dir().join("virtual").join(&version.asset_index.id)
    } else {
        return Ok(paths::assets_dir());
    };
    for (name, object) in &index.objects {
        let dest = target.join(name);
        if dest.metadata().map(|m| m.len() == object.size).unwrap_or(false) {
            continue;
        }
        if let Some(parent) = dest.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::copy(object.object_path(), &dest).map_err(|e| format!("Could not copy asset {name}: {e}"))?;
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn version(game_args: &str) -> Version {
        serde_json::from_str(&format!(
            r#"{{"id":"x","mainClass":"m","arguments":{{"game":{game_args},"jvm":[]}},
                "assetIndex":{{"id":"1","url":"u","sha1":"s"}},"downloads":{{}}}}"#
        ))
        .unwrap()
    }

    #[test]
    fn join_server_uses_quick_play_when_available() {
        let modern = version(
            r#"[{"rules":[{"action":"allow","features":{"is_quick_play_multiplayer":true}}],"value":["--quickPlayMultiplayer","${quickPlayMultiplayer}"]}]"#,
        );
        assert_eq!(
            join_server_args(&modern, "play.example.net"),
            ["--quickPlayMultiplayer", "play.example.net"]
        );
        let old = version("[]");
        assert_eq!(
            join_server_args(&old, "mc.example.net:25570"),
            ["--server", "mc.example.net", "--port", "25570"]
        );
        assert_eq!(
            join_server_args(&old, "mc.example.net"),
            ["--server", "mc.example.net", "--port", "25565"]
        );
        assert!(join_server_args(&old, " ").is_empty());
    }

    #[test]
    fn substitutes_known_variables_only() {
        let mut vars = HashMap::new();
        vars.insert("auth_player_name", "Steve".to_string());
        assert_eq!(substitute("--username=${auth_player_name}", &vars), "--username=Steve");
        assert_eq!(substitute("${unknown} x", &vars), "${unknown} x");
        assert_eq!(substitute("plain", &vars), "plain");
    }

    #[test]
    fn expands_rules() {
        let vars = HashMap::new();
        let list: Vec<Value> = serde_json::from_str(
            r#"["--a", {"rules":[{"action":"allow","features":{"is_demo_user":true}}],"value":"--demo"},
               {"rules":[{"action":"allow"}],"value":["--b","--c"]}]"#,
        )
        .unwrap();
        assert_eq!(expand(&list, &vars), vec!["--a", "--b", "--c"]);
    }
}
