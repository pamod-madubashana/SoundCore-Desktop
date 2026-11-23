//! Configuration: the devices to watch and the per-device profile that mirrors
//! the settings reported by the device on every connect (read-only fetch).
//! Profiles are plain id/value records; no cross-setting rules are invented.

pub mod autostart;

use std::{path::PathBuf, str::FromStr};

use anyhow::{Context, anyhow};
use macaddr::MacAddr6;
use serde::{Deserialize, Serialize};
use soundcore_protocol::vendor::DeviceModel;

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
    /// Windows toast notifications on device connect/disconnect.
    #[serde(default = "default_true")]
    pub notify_connect: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            autostart: true,
            devices: Vec::new(),
            update_restart: false,
            eq_presets: Vec::new(),
            notify_connect: true,
        }
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
    /// Seconds to wait after connect before reading the device's settings.
    #[serde(default = "default_apply_delay")]
    pub apply_delay_seconds: u64,
    /// Settings recorded for this device. On every connect this list is REPLACED
    /// with the device's current settings — a read-only fetch that never writes
    /// anything to the device. Live UI changes write the single setting they
    /// touch and update this list for reference; it is never pushed back.
    #[serde(default)]
    pub profile: Vec<SettingEntry>,
    /// Optional product image URL shown in the popup (otherwise a generic illustration).
    #[serde(default)]
    pub image: Option<String>,
    /// Device color as a hex string (e.g., "#1a1a1a") or a Soundcore color code ("1"-"y").
    /// Used to tint the SVG device template.
    #[serde(default)]
    pub color: Option<String>,
    /// Unix timestamp (seconds) of the last successful connect, used to order
    /// the idle "My Devices" list newest-first. None for never-connected or
    /// pre-upgrade devices (they sort last).
    #[serde(default)]
    pub last_connected: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingEntry {
    pub id: String,
    pub value: String,
}

/// Upserts one device-reported setting by id. No cross-setting rules: the
/// profile only mirrors what the device reports.
pub fn upsert_entry(list: &mut Vec<SettingEntry>, id: String, value: String) {
    match list.iter_mut().find(|e| e.id == id) {
        Some(e) => e.value = value,
        None => list.push(SettingEntry { id, value }),
    }
}

/// Deduplicates a profile by id, keeping the last occurrence of each setting
/// and the relative order of the survivors. Returns true if anything was
/// removed.
pub fn prune_profile(list: &mut Vec<SettingEntry>) -> bool {
    let before = list.len();
    let mut keep = vec![true; list.len()];
    let mut seen: Vec<&str> = Vec::new();
    for (i, entry) in list.iter().enumerate().rev() {
        if !seen.contains(&entry.id.as_str()) {
            seen.push(entry.id.as_str());
        } else {
            keep[i] = false;
        }
    }
    let mut i = 0;
    list.retain(|_| {
        let retained = keep[i];
        i += 1;
        retained
    });
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
            last_connected: None,
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

/// Orders devices newest-connect first for the idle "My Devices" list.
/// Never-connected devices (`last_connected: None`) sort last, preserving
/// config order among themselves. The sort is stable, so equal timestamps
/// also keep config order.
pub fn sort_devices_by_recency(devices: &mut [DeviceConfig]) {
    devices.sort_by_key(|d| std::cmp::Reverse(d.last_connected));
}

impl Config {
    pub fn load(path: &PathBuf) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let mut config: Self = toml::from_str(&text)?;
        for dev in &mut config.devices {
            if prune_profile(&mut dev.profile) {
                tracing::info!("removed duplicate settings from '{}' profile", dev.label());
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

pub fn load() -> Config {
    let path = resolve_path(None);
    Config::load(&path).unwrap_or_default()
}

pub fn save(cfg: &Config) -> Result<(), String> {
    let path = resolve_path(None);
    cfg.save(&path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev(mac: &str, last_connected: Option<u64>) -> DeviceConfig {
        DeviceConfig {
            mac_address: mac.to_owned(),
            last_connected,
            ..DeviceConfig::default()
        }
    }

    #[test]
    fn last_connected_missing_from_toml_defaults_to_none() {
        let cfg: Config = toml::from_str(
            r#"
            [[devices]]
            mac_address = "34:09:C9:B9:EC:30"
            model = "SoundcoreA3959"
            "#,
        )
        .expect("config without last_connected should parse");
        assert_eq!(cfg.devices.len(), 1);
        assert_eq!(cfg.devices[0].last_connected, None);
    }

    #[test]
    fn sort_devices_newest_first_none_last() {
        let mut devices = vec![
            dev("aa", None),
            dev("bb", Some(100)),
            dev("cc", None),
            dev("dd", Some(300)),
            dev("ee", Some(200)),
        ];
        sort_devices_by_recency(&mut devices);
        let order: Vec<&str> = devices.iter().map(|d| d.mac_address.as_str()).collect();
        assert_eq!(order, vec!["dd", "ee", "bb", "aa", "cc"]);
    }

    #[test]
    fn sort_devices_is_stable_for_ties_and_never_connected() {
        let mut devices = vec![
            dev("first", Some(100)),
            dev("second", Some(100)),
            dev("third", None),
            dev("fourth", None),
        ];
        sort_devices_by_recency(&mut devices);
        let order: Vec<&str> = devices.iter().map(|d| d.mac_address.as_str()).collect();
        assert_eq!(order, vec!["first", "second", "third", "fourth"]);
    }

    #[test]
    fn upsert_entry_keeps_spatial_and_eq_entries_together() {
        let mut profile = Vec::new();
        upsert_entry(&mut profile, "spatialAudio".to_owned(), "true".to_owned());
        upsert_entry(
            &mut profile,
            "volumeAdjustments".to_owned(),
            "6,4,1,0,0".to_owned(),
        );
        upsert_entry(&mut profile, "spatialAudio".to_owned(), "false".to_owned());

        assert_eq!(profile.len(), 2);
        assert_eq!(profile[0].id, "spatialAudio");
        assert_eq!(profile[0].value, "false");
        assert_eq!(profile[1].id, "volumeAdjustments");
        assert_eq!(profile[1].value, "6,4,1,0,0");
    }

    #[test]
    fn prune_profile_keeps_last_duplicate_and_reports_removal() {
        let mut profile = vec![
            SettingEntry {
                id: "a".to_owned(),
                value: "1".to_owned(),
            },
            SettingEntry {
                id: "b".to_owned(),
                value: "2".to_owned(),
            },
            SettingEntry {
                id: "a".to_owned(),
                value: "3".to_owned(),
            },
            SettingEntry {
                id: "c".to_owned(),
                value: "4".to_owned(),
            },
            SettingEntry {
                id: "b".to_owned(),
                value: "5".to_owned(),
            },
        ];
        assert!(prune_profile(&mut profile));
        let pairs: Vec<(&str, &str)> = profile
            .iter()
            .map(|e| (e.id.as_str(), e.value.as_str()))
            .collect();
        assert_eq!(pairs, vec![("a", "3"), ("c", "4"), ("b", "5")]);
    }

    #[test]
    fn prune_profile_returns_false_for_clean_profile() {
        let mut profile = vec![
            SettingEntry {
                id: "a".to_owned(),
                value: "1".to_owned(),
            },
            SettingEntry {
                id: "b".to_owned(),
                value: "2".to_owned(),
            },
        ];
        assert!(!prune_profile(&mut profile));
        assert_eq!(profile.len(), 2);
    }
}
