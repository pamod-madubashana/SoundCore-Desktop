use std::{str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use soundcore_protocol_core::{
    device::OpenSCQ30Device,
    settings::{ModifiableSelectCommand, MultiSelectWithRemoveCommand, Select, Setting, SettingId},
};

use crate::capability::DeviceCapabilities;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingValue {
    pub id: String,
    pub value: JsonValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSnapshot {
    pub model: String,
    pub values: Vec<SettingValue>,
}

pub struct ConnectedDevice {
    inner: Arc<dyn OpenSCQ30Device + Send + Sync>,
}

impl ConnectedDevice {
    pub(crate) fn new(inner: Arc<dyn OpenSCQ30Device + Send + Sync>) -> Self {
        Self { inner }
    }

    pub fn model(&self) -> String {
        self.inner.model().to_string()
    }

    pub fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::from_device(self.inner.as_ref())
    }

    pub fn snapshot(&self) -> DeviceSnapshot {
        let mut values = Vec::new();
        for (_, entries) in self.inner.settings_by_category() {
            for (id, setting) in entries {
                if let Ok(json) = serde_json::to_value(&setting) {
                    let value = json.get("value").cloned().unwrap_or(JsonValue::Null);
                    values.push(SettingValue {
                        id: id.to_string(),
                        value,
                    });
                }
            }
        }
        DeviceSnapshot {
            model: self.model(),
            values,
        }
    }

    pub async fn set_raw(&self, id: &str, raw: &str) -> Result<(), String> {
        let setting_id = SettingId::from_str(id).map_err(|_| format!("unknown setting '{id}'"))?;
        let setting = self
            .inner
            .setting(&setting_id)
            .ok_or_else(|| format!("setting '{id}' is not available"))?;
        let value = parse_value(&setting, raw)?;
        self.inner
            .set_setting_values(vec![(setting_id, value)])
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn set_many(&self, values: Vec<(String, String)>) -> Result<(), String> {
        let mut parsed = Vec::new();
        for (id, raw) in values {
            let setting_id =
                SettingId::from_str(&id).map_err(|_| format!("unknown setting '{id}'"))?;
            let setting = self
                .inner
                .setting(&setting_id)
                .ok_or_else(|| format!("setting '{id}' is not available"))?;
            parsed.push((setting_id, parse_value(&setting, &raw)?));
        }
        self.inner
            .set_setting_values(parsed)
            .await
            .map_err(|e| e.to_string())
    }

    pub fn inner(&self) -> &Arc<dyn OpenSCQ30Device + Send + Sync> {
        &self.inner
    }
}

pub fn parse_value(
    setting: &Setting,
    raw: &str,
) -> Result<soundcore_protocol_core::settings::Value, String> {
    use soundcore_protocol_core::settings::Value;
    match setting {
        Setting::Toggle { .. } => Ok(Value::from(bool::from_str(raw).map_err(|e| e.to_string())?)),
        Setting::I32Range { setting, .. } => {
            let n = i32::from_str(raw).map_err(|e| e.to_string())?;
            if !setting.range.contains(&n) {
                return Err(format!("{n} is out of range {:?}", setting.range));
            }
            if setting.step != 0 && n % setting.step != 0 {
                return Err(format!("{n} does not align with step {}", setting.step));
            }
            Ok(n.into())
        }
        Setting::Select { setting, .. } => Ok(Value::String(one_of(setting, raw)?)),
        Setting::OptionalSelect { setting, .. } => {
            if raw.is_empty() {
                Ok(Value::OptionalString(None))
            } else {
                Ok(Value::OptionalString(Some(one_of(setting, raw)?)))
            }
        }
        Setting::ModifiableSelect { setting, .. } => {
            if let Some(rest) = raw.strip_prefix('+') {
                Ok(Value::ModifiableSelectCommand(
                    ModifiableSelectCommand::Add(rest.to_owned().into()),
                ))
            } else if let Some(rest) = raw.strip_prefix('-') {
                Ok(Value::ModifiableSelectCommand(
                    ModifiableSelectCommand::Remove(rest.to_owned().into()),
                ))
            } else {
                Ok(Value::from(one_of(
                    setting,
                    raw.strip_prefix('\\').unwrap_or(raw),
                )?))
            }
        }
        Setting::MultiSelect { setting, .. } => {
            let selections = raw
                .split(',')
                .map(|item| one_of(setting, item.trim()))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Value::from(selections))
        }
        Setting::MultiSelectWithRemove { setting, .. } => {
            if let Some(rest) = raw.strip_prefix('-') {
                Ok(
                    soundcore_protocol_core::settings::Value::MultiSelectWithRemoveCommand(
                        MultiSelectWithRemoveCommand::Remove(rest.to_owned().into()),
                    ),
                )
            } else {
                Ok(one_of(setting, raw.strip_prefix('\\').unwrap_or(raw))?.into())
            }
        }
        Setting::Equalizer { setting, .. } => {
            let values = raw
                .split(',')
                .map(|s| i16::from_str(s.trim()).map_err(|e| e.to_string()))
                .collect::<Result<Vec<_>, _>>()?;
            if values.len() != setting.band_hz.len() {
                return Err(format!(
                    "expected {} bands, got {}",
                    setting.band_hz.len(),
                    values.len()
                ));
            }
            for v in &values {
                if *v < setting.min || *v > setting.max {
                    return Err(format!(
                        "band {v} outside range {}..={}",
                        setting.min, setting.max
                    ));
                }
            }
            Ok(Value::I16Vec(values))
        }
        Setting::Information { .. } => Err("information settings are read-only".into()),
        Setting::ImportString { .. } => Ok(Value::from(std::borrow::Cow::from(raw.to_owned()))),
        Setting::Action => Ok(Value::Bool(true)),
        Setting::PresetEqualizerProfileSelect { .. } => {
            Ok(Value::String(std::borrow::Cow::from(raw.to_owned())))
        }
        Setting::HueColorPicker { .. } => Ok(Value::String(std::borrow::Cow::from(raw.to_owned()))),
        Setting::TimeOfDay { .. } => {
            let n = i32::from_str(raw.trim()).map_err(|e| e.to_string())?;
            if !(0..1440).contains(&n) {
                return Err(format!("{n} is not a valid time of day"));
            }
            Ok(Value::I32(n))
        }
    }
}

fn one_of(setting: &Select, raw: &str) -> Result<std::borrow::Cow<'static, str>, String> {
    if let Some(exact) = setting.options.iter().find(|o| **o == raw) {
        return Ok(exact.clone());
    }
    let mut matches = setting
        .options
        .iter()
        .filter(|o| o.eq_ignore_ascii_case(raw));
    let first = matches.next();
    if let (Some(a), Some(b)) = (first, matches.next()) {
        return Err(format!("'{raw}' is ambiguous: could be {a} or {b}"));
    }
    first.cloned().ok_or_else(|| {
        format!(
            "'{raw}' is not valid. Expected one of: {:?}",
            setting.options
        )
    })
}
