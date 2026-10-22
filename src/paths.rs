//! Where Scaleri keeps its files.
//!
//! Everything lives in `%APPDATA%\Scaleri` unless `SCALERI_DATA_DIR` points somewhere else:
//!
//! ```text
//! Scaleri/
//!   settings.json, accounts.json
//!   instances/<id>/instance.json, instances/<id>/minecraft/
//!   libraries/, assets/, versions/, runtimes/
//! ```

use std::path::PathBuf;
use std::sync::OnceLock;

pub fn data_dir() -> &'static PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = std::env::var_os("SCALERI_DATA_DIR")
            .map(PathBuf::from)
            .or_else(|| dirs::data_dir().map(|d| d.join("Scaleri")))
            .unwrap_or_else(|| PathBuf::from("Scaleri"));
        let _ = std::fs::create_dir_all(&dir);
        dir
    })
}

pub fn instances_dir() -> PathBuf {
    data_dir().join("instances")
}

pub fn libraries_dir() -> PathBuf {
    data_dir().join("libraries")
}

pub fn assets_dir() -> PathBuf {
    data_dir().join("assets")
}

pub fn versions_dir() -> PathBuf {
    data_dir().join("versions")
}

pub fn runtimes_dir() -> PathBuf {
    data_dir().join("runtimes")
}

/// Writes a file atomically, so a crash never leaves half a settings file behind.
pub fn write_atomic(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, path)
}

/// Opens a folder in Explorer (or the platform's file manager while developing on another OS).
pub fn open_in_explorer(path: &std::path::Path) {
    let _ = std::fs::create_dir_all(path);
    #[cfg(windows)]
    let program = "explorer";
    #[cfg(target_os = "macos")]
    let program = "open";
    #[cfg(all(not(windows), not(target_os = "macos")))]
    let program = "xdg-open";
    let _ = std::process::Command::new(program).arg(path).spawn();
}

/// Opens a file with its default app, e.g. a log in Notepad.
pub fn open_file(path: &std::path::Path) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let _ = std::process::Command::new("rundll32")
            .arg("url.dll,FileProtocolHandler")
            .arg(path)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
    #[cfg(not(windows))]
    open_in_explorer(path);
}

/// Opens a web page in the default browser.
pub fn open_url(url: &str) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let _ = std::process::Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(url).spawn();
    #[cfg(all(not(windows), not(target_os = "macos")))]
    let _ = std::process::Command::new("xdg-open").arg(url).spawn();
}
