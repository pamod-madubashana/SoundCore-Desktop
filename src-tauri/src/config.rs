//! Configuration: the devices to watch and the per-device profile to apply on connect.

use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, anyhow};
use macaddr::MacAddr6;
use openscq30_lib::DeviceModel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Start automatically when the user logs in.
    #[serde(default = "default_true")]
    pub autostart: bool,
    #[serde(default)]
    pub devices: Vec<DeviceConfig>,
    /// Set to true when restarting after an update (cleared on next startup).
    #[serde(default)]
    pub update_restart: bool,
    /// Custom equalizer presets (global + per-device).
    #[serde(default)]
    pub eq_presets: Vec<EqPresetEntry>,
}

impl Default for Config {
    fn default() -> Self {
        Self { autostart: true, devices: Vec::new(), update_restart: false, eq_presets: Vec::new() }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// Friendly label shown in the UI.
    #[serde(default)]
    pub name: String,
    pub mac_address: String,
    pub model: String,
    #[serde(default = "default_poll_seconds")]
    pub poll_seconds: u64,
    #[serde(default = "default_apply_delay")]
    pub apply_delay_seconds: u64,
    /// Settings to push on every connect, in order.
    #[serde(default)]
    pub profile: Vec<SettingEntry>,
    /// Optional product image URL shown in the popup (otherwise a generic illustration).
    #[serde(default)]
    pub image: Option<String>,
    /// Device color as a hex string (e.g., "#1a1a1a") or a Soundcore color code ("1"-"y").
    /// Used to tint the SVG device template.
    #[serde(default)]
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingEntry {
    pub id: String,
    pub value: String,
}

/// Settings that each select the *source* of the equalizer curve. Only one can be
/// in effect at a time.
const EQ_SOURCE_IDS: [&str; 3] = [
    "volumeAdjustments",
    "presetEqualizerProfile",
    "customEqualizerProfile",
];

fn is_eq_source(id: &str) -> bool {
    EQ_SOURCE_IDS.contains(&id)
}

fn is_enabling_spatial(id: &str, value: &str) -> bool {
    id == "spatialAudio" && value.trim().eq_ignore_ascii_case("true")
}

/// Drops entries that cannot coexist with `id`.
///
/// The whole profile is replayed in a single `set_setting_values` call on connect,
/// and the device folds it into one target state before sending anything. Writing
/// any equalizer value clears spatial audio device-side, and two equalizer sources
/// simply overwrite each other — so a profile holding e.g. both `volumeAdjustments`
/// and `spatialAudio = true` silently loses spatial audio on every reconnect.
/// Keeping the profile internally consistent is what makes the mode stick.
fn prune_conflicts(list: &mut Vec<SettingEntry>, id: &str, value: &str) {
    let eq_source = is_eq_source(id);
    if !eq_source && !is_enabling_spatial(id, value) {
        return;
    }
    list.retain(|e| {
        if e.id == id {
            return true;
        }
        // At most one equalizer source survives, and enabling spatial audio
        // removes all of them.
        if is_eq_source(&e.id) {
            return false;
        }
        // Picking an equalizer source turns spatial audio off. `spatialAudioMode`
        // is kept so the choice is remembered for next time.
        if e.id == "spatialAudio" && eq_source {
            return false;
        }
        true
    });
}

/// Upserts one entry into a profile, dropping anything it contradicts.
pub fn upsert_entry(list: &mut Vec<SettingEntry>, id: String, value: String) {
    prune_conflicts(list, &id, &value);
    match list.iter_mut().find(|e| e.id == id) {
        Some(e) => e.value = value,
        None => list.push(SettingEntry { id, value }),
    }
}

/// Resolves contradictions in a profile that was written before the rules above
/// existed. The last conflicting entry wins, matching "most recently changed".
/// Returns true if anything was removed.
pub fn prune_profile(list: &mut Vec<SettingEntry>) -> bool {
    let winner = list
        .iter()
        .rposition(|e| is_eq_source(&e.id) || is_enabling_spatial(&e.id, &e.value))
        .map(|i| (list[i].id.clone(), list[i].value.clone()));
    let Some((id, value)) = winner else {
        return false;
    };
    let before = list.len();
    prune_conflicts(list, &id, &value);
    list.len() != before
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqPresetEntry {
    /// Auto-incrementing ID.
    pub id: i64,
    /// User-facing name.
    pub name: String,
    /// Comma-separated band values (e.g. "6,4,1,0,0").
    pub bands: String,
    /// Device model this preset is for, or empty/null for global presets.
    #[serde(default)]
    pub model: String,
}

fn default_poll_seconds() -> u64 {
    5
}
fn default_apply_delay() -> u64 {
    2
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            name: "New device".to_owned(),
            mac_address: String::new(),
            model: "SoundcoreA3959".to_owned(),
            poll_seconds: 5,
            apply_delay_seconds: 2,
            profile: Vec::new(),
            image: None,
            color: None,
        }
    }
}

impl DeviceConfig {
    /// Validates and parses the mac address and model.
    pub fn parse(&self) -> anyhow::Result<(MacAddr6, DeviceModel)> {
        let mac = MacAddr6::from_str(self.mac_address.trim())
            .with_context(|| format!("invalid mac_address '{}'", self.mac_address))?;
        let model = DeviceModel::from_str(self.model.trim())
            .map_err(|_| anyhow!("invalid model '{}'", self.model))?;
        Ok((mac, model))
    }

    /// A label for the UI, falling back to the mac if no name is set.
    pub fn label(&self) -> String {
        if self.name.trim().is_empty() {
            if self.mac_address.trim().is_empty() {
                "New device".to_owned()
            } else {
                self.mac_address.clone()
            }
        } else {
            self.name.clone()
        }
    }
}

impl Config {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let mut config: Self = toml::from_str(&text)?;
        for dev in &mut config.devices {
            if prune_profile(&mut dev.profile) {
                tracing::info!(
                    "dropped conflicting sound-effect entries from '{}' profile",
                    dev.label()
                );
            }
        }
        Ok(config)
    }

    pub fn save(&self, path: &PathBuf) -> anyhow::Result<()> {
        let text = toml::to_string_pretty(self)?;
        std::fs::write(path, text).with_context(|| format!("failed to write {}", path.display()))
    }
}

/// config.toml resolution: explicit path, else next to the exe, else CWD.
pub fn resolve_path(explicit: Option<String>) -> PathBuf {
    if let Some(arg) = explicit {
        return PathBuf::from(arg);
    }
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join("config.toml");
        if candidate.exists() {
            return candidate;
        }
    }
    PathBuf::from("config.toml")
}

/// Load config using the path stored in the Tauri managed state.
pub fn load(_app: &tauri::AppHandle) -> Config {
    let path = resolve_path(None);
    Config::load(&path).unwrap_or_default()
}

/// Save config using the path stored in the Tauri managed state.
pub fn save(_app: &tauri::AppHandle, cfg: &Config) -> Result<(), String> {
    let path = resolve_path(None);
    cfg.save(&path).map_err(|e| e.to_string())
}
