//! HTTP helpers: JSON requests, verified downloads and a small parallel download queue.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use serde::de::DeserializeOwned;

pub type Result<T> = std::result::Result<T, String>;

pub fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_connect(Some(Duration::from_secs(15)))
            .timeout_recv_body(Some(Duration::from_secs(120)))
            .user_agent(concat!("Scaleri/", env!("CARGO_PKG_VERSION")))
            .build();
        config.into()
    })
}

/// Turns a response into its JSON body, or an error naming the status and the server's message.
pub fn json_body<T: DeserializeOwned>(what: &str, mut res: ureq::http::Response<ureq::Body>) -> Result<T> {
    let status = res.status();
    if !status.is_success() {
        let body = res.body_mut().read_to_string().unwrap_or_default();
        let body: String = body.chars().take(300).collect();
        return Err(format!("{what} failed ({status}): {body}"));
    }
    res.body_mut()
        .with_config()
        .limit(64 * 1024 * 1024)
        .read_json()
        .map_err(|e| format!("{what}: could not read the response ({e})"))
}

pub fn get_json<T: DeserializeOwned>(url: &str) -> Result<T> {
    let res = agent()
        .get(url)
        .call()
        .map_err(|e| format!("Could not reach {}: {e}", host(url)))?;
    json_body(url, res)
}

fn host(url: &str) -> &str {
    url.split("://").nth(1).and_then(|r| r.split('/').next()).unwrap_or(url)
}

pub fn sha1_file(path: &Path) -> Option<String> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut hasher = sha1_smol::Sha1::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Some(hasher.digest().to_string())
}

/// One file to fetch. `sha1` and `size` are checked when the source provides them.
#[derive(Clone, Debug)]
pub struct Download {
    pub url: String,
    pub path: PathBuf,
    pub sha1: Option<String>,
    pub size: Option<u64>,
    /// Marks downloaded runtime files executable (only matters off Windows).
    #[cfg_attr(windows, allow(dead_code))]
    pub executable: bool,
}

impl Download {
    /// Whether the file on disk is already the right one.
    pub fn is_current(&self, verify_hash: bool) -> bool {
        let Ok(meta) = std::fs::metadata(&self.path) else {
            return false;
        };
        if let Some(size) = self.size {
            if meta.len() != size {
                return false;
            }
        }
        match (&self.sha1, verify_hash) {
            (Some(expected), true) => sha1_file(&self.path).as_deref() == Some(expected.as_str()),
            _ => true,
        }
    }

    pub fn fetch(&self) -> Result<()> {
        let mut last_err = String::new();
        for attempt in 0..3 {
            if attempt > 0 {
                std::thread::sleep(Duration::from_millis(500 * attempt));
            }
            match self.try_fetch() {
                Ok(()) => return Ok(()),
                Err(e) => last_err = e,
            }
        }
        Err(last_err)
    }

    fn try_fetch(&self) -> Result<()> {
        let res = agent()
            .get(&self.url)
            .call()
            .map_err(|e| format!("Could not download {}: {e}", self.url))?;
        if !res.status().is_success() {
            return Err(format!("Could not download {} ({})", self.url, res.status()));
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Could not create {}: {e}", parent.display()))?;
        }
        let tmp = self.path.with_extension("part");
        let mut reader = res.into_body().into_reader();
        let mut file = std::fs::File::create(&tmp).map_err(|e| format!("Could not write {}: {e}", tmp.display()))?;
        let mut hasher = sha1_smol::Sha1::new();
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let n = reader
                .read(&mut buf)
                .map_err(|e| format!("Download of {} was cut off: {e}", self.url))?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            file.write_all(&buf[..n])
                .map_err(|e| format!("Could not write {}: {e}", tmp.display()))?;
        }
        drop(file);
        if let Some(expected) = &self.sha1 {
            let actual = hasher.digest().to_string();
            if &actual != expected {
                let _ = std::fs::remove_file(&tmp);
                return Err(format!(
                    "{} was corrupted while downloading (checksum mismatch)",
                    self.url
                ));
            }
        }
        std::fs::rename(&tmp, &self.path).map_err(|e| format!("Could not save {}: {e}", self.path.display()))?;
        #[cfg(unix)]
        if self.executable {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&self.path, std::fs::Permissions::from_mode(0o755));
        }
        Ok(())
    }
}

/// Progress of a running task, shared between the worker thread and the UI.
#[derive(Default)]
pub struct Progress {
    pub label: Mutex<String>,
    pub done: AtomicUsize,
    pub total: AtomicUsize,
    pub bytes: AtomicU64,
    pub cancel: AtomicBool,
}

impl Progress {
    pub fn set_stage(&self, label: impl Into<String>, total: usize) {
        *self.label.lock().unwrap() = label.into();
        self.done.store(0, Ordering::Relaxed);
        self.total.store(total, Ordering::Relaxed);
    }

    pub fn fraction(&self) -> Option<f32> {
        let total = self.total.load(Ordering::Relaxed);
        (total > 0).then(|| self.done.load(Ordering::Relaxed) as f32 / total as f32)
    }

    pub fn cancelled(&self) -> Result<()> {
        if self.cancel.load(Ordering::Relaxed) {
            Err("Cancelled".into())
        } else {
            Ok(())
        }
    }
}

/// Downloads every missing or outdated file, eight at a time.
pub fn download_all(label: &str, items: Vec<Download>, verify_hash: bool, progress: &Arc<Progress>) -> Result<()> {
    let pending: Vec<Download> = items.into_iter().filter(|d| !d.is_current(verify_hash)).collect();
    progress.set_stage(label, pending.len());
    if pending.is_empty() {
        return Ok(());
    }
    let queue = Arc::new(Mutex::new(pending));
    let error: Arc<Mutex<Option<String>>> = Arc::default();
    let workers: Vec<_> = (0..8)
        .map(|_| {
            let queue = queue.clone();
            let error = error.clone();
            let progress = progress.clone();
            std::thread::spawn(move || loop {
                if progress.cancel.load(Ordering::Relaxed) || error.lock().unwrap().is_some() {
                    return;
                }
                let Some(item) = queue.lock().unwrap().pop() else {
                    return;
                };
                match item.fetch() {
                    Ok(()) => {
                        progress.done.fetch_add(1, Ordering::Relaxed);
                        progress.bytes.fetch_add(item.size.unwrap_or(0), Ordering::Relaxed);
                    }
                    Err(e) => {
                        error.lock().unwrap().get_or_insert(e);
                    }
                }
            })
        })
        .collect();
    for worker in workers {
        let _ = worker.join();
    }
    progress.cancelled()?;
    let first_error = error.lock().unwrap().take();
    match first_error {
        Some(e) => Err(e),
        None => Ok(()),
    }
}
