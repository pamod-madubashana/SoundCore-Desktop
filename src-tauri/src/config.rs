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
}

impl Default for Config {
    fn default() -> Self {
        Self { autostart: true, devices: Vec::new() }
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingEntry {
    pub id: String,
    pub value: String,
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
        toml::from_str(&text).map_err(Into::into)
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
