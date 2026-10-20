//! Instances: separate Minecraft installations, each with its own worlds, settings and mods.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::paths;

/// Block icons Scaleri can draw for an instance.
pub const ICONS: &[&str] = &[
    "grass",
    "stone",
    "planks",
    "sand",
    "diamond",
    "tnt",
    "obsidian",
    "snow",
    "netherrack",
    "glass",
];

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Instance {
    /// Folder name under `instances/`.
    #[serde(skip)]
    pub id: String,
    pub name: String,
    /// Minecraft version ID, e.g. `1.21.4`.
    pub version: String,
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default)]
    pub created: u64,
    #[serde(default)]
    pub last_played: Option<u64>,
    #[serde(default)]
    pub play_seconds: u64,
    /// Overrides the global memory setting when set.
    #[serde(default)]
    pub memory_mb: Option<u32>,
}

fn default_icon() -> String {
    "grass".into()
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl Instance {
    pub fn dir(&self) -> PathBuf {
        paths::instances_dir().join(&self.id)
    }

    /// The game directory: worlds, options, mods, resource packs and screenshots.
    pub fn game_dir(&self) -> PathBuf {
        self.dir().join("minecraft")
    }

    pub fn log_path(&self) -> PathBuf {
        self.dir().join("latest-launch.log")
    }

    pub fn save(&self) -> std::io::Result<()> {
        let data = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
        paths::write_atomic(&self.dir().join("instance.json"), &data)
    }

    pub fn create(name: &str, version: &str, icon: &str) -> std::io::Result<Instance> {
        let base: String = name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let base = if base.trim_matches('_').is_empty() {
            "instance".to_string()
        } else {
            base
        };
        let mut id = base.clone();
        let mut n = 2;
        while paths::instances_dir().join(&id).exists() {
            id = format!("{base}_{n}");
            n += 1;
        }
        let instance = Instance {
            id,
            name: name.trim().to_string(),
            version: version.to_string(),
            icon: icon.to_string(),
            created: now(),
            last_played: None,
            play_seconds: 0,
            memory_mb: None,
        };
        std::fs::create_dir_all(instance.game_dir())?;
        instance.save()?;
        Ok(instance)
    }

    pub fn delete(&self) -> std::io::Result<()> {
        std::fs::remove_dir_all(self.dir())
    }

    /// "Played 3h 20m" style summary.
    pub fn playtime_text(&self) -> String {
        if self.last_played.is_none() {
            return "Not played yet".into();
        }
        let minutes = self.play_seconds / 60;
        if minutes < 1 {
            "Played less than a minute".into()
        } else if minutes < 60 {
            format!("Played {minutes}m")
        } else {
            format!("Played {}h {}m", minutes / 60, minutes % 60)
        }
    }
}

/// Every instance on disk, most recently played first.
pub fn load_all() -> Vec<Instance> {
    let mut list = Vec::new();
    let Ok(entries) = std::fs::read_dir(paths::instances_dir()) else {
        return list;
    };
    for entry in entries.flatten() {
        let path = entry.path().join("instance.json");
        let Ok(data) = std::fs::read(&path) else {
            continue;
        };
        let Ok(mut instance) = serde_json::from_slice::<Instance>(&data) else {
            continue;
        };
        instance.id = entry.file_name().to_string_lossy().into_owned();
        list.push(instance);
    }
    sort(&mut list);
    list
}

pub fn sort(list: &mut [Instance]) {
    list.sort_by(|a, b| {
        b.last_played
            .unwrap_or(b.created)
            .cmp(&a.last_played.unwrap_or(a.created))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
}
