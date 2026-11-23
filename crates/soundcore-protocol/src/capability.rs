use serde::{Deserialize, Serialize};
use serde_json::Value;
use soundcore_protocol_core::{
    device::OpenSCQ30Device,
    settings::{Setting, SettingId},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SettingKind {
    Toggle,
    Range,
    Select,
    MultiSelect,
    Equalizer,
    PresetEqualizer,
    Action,
    Information,
    ImportString,
    Color,
    Time,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingCapability {
    pub id: String,
    pub category: String,
    pub kind: SettingKind,
    pub writable: bool,
    pub options: Vec<String>,
    pub localized_options: Vec<String>,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub step: Option<i64>,
    pub band_hz: Vec<u16>,
    pub fraction_digits: Option<i32>,
}

impl SettingCapability {
    fn from_json(id: String, category: String, json: &Value) -> Self {
        let kind = match json.get("type").and_then(Value::as_str).unwrap_or_default() {
            "toggle" => SettingKind::Toggle,
            "i32Range" => SettingKind::Range,
            "select" | "optionalSelect" | "modifiableSelect" => SettingKind::Select,
            "multiSelect" | "multiSelectWithRemove" => SettingKind::MultiSelect,
            "equalizer" => SettingKind::Equalizer,
            "presetEqualizerProfileSelect" => SettingKind::PresetEqualizer,
            "action" => SettingKind::Action,
            "information" => SettingKind::Information,
            "importString" => SettingKind::ImportString,
            "hueColorPicker" => SettingKind::Color,
            "timeOfDay" => SettingKind::Time,
            _ => SettingKind::Other,
        };
        let writable = !matches!(
            kind,
            SettingKind::Information | SettingKind::ImportString | SettingKind::Other
        ) && json.get("readOnly").and_then(Value::as_bool) != Some(true);
        let select = json
            .get("setting")
            .filter(|v| v.is_object())
            .or_else(|| json.get("select").filter(|v| v.is_object()));
        let options = strings(select.and_then(|v| v.get("options")));
        let localized_options = strings(select.and_then(|v| v.get("localizedOptions")));
        let range = json
            .get("setting")
            .and_then(|v| v.get("range"))
            .filter(|v| v.is_object())
            .or(select.filter(|v| v.get("start").is_some()));
        let min = range.and_then(|v| v.get("start")).and_then(Value::as_i64);
        let max = range.and_then(|v| v.get("end")).and_then(Value::as_i64);
        let step = range.and_then(|v| v.get("step")).and_then(Value::as_i64);
        let eq = json.get("setting").filter(|v| v.get("bandHz").is_some());
        let band_hz = eq
            .and_then(|v| v.get("bandHz"))
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_u64().map(|n| n as u16))
                    .collect()
            })
            .unwrap_or_default();
        let fraction_digits = eq
            .and_then(|v| v.get("fractionDigits"))
            .and_then(Value::as_i64)
            .map(|n| n as i32);
        Self {
            id,
            category,
            kind,
            writable,
            options,
            localized_options,
            min,
            max,
            step,
            band_hz,
            fraction_digits,
        }
    }
}

fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCapabilities {
    pub model: String,
    pub settings: Vec<SettingCapability>,
}

impl DeviceCapabilities {
    pub fn from_device<D: OpenSCQ30Device + ?Sized>(device: &D) -> Self {
        let mut settings = Vec::new();
        for (category, entries) in device.settings_by_category() {
            let category = category.to_string();
            for (id, setting) in entries {
                if let Some(capability) = descriptor(id, category.clone(), &setting) {
                    settings.push(capability);
                }
            }
        }
        Self {
            model: device.model().to_string(),
            settings,
        }
    }

    pub fn supports(&self, id: &str) -> bool {
        self.settings.iter().any(|s| s.id == id)
    }

    pub fn setting(&self, id: &str) -> Option<&SettingCapability> {
        self.settings.iter().find(|s| s.id == id)
    }

    pub fn writable_settings(&self) -> impl Iterator<Item = &SettingCapability> {
        self.settings.iter().filter(|s| s.writable)
    }
}

fn descriptor(id: SettingId, category: String, setting: &Setting) -> Option<SettingCapability> {
    let json = serde_json::to_value(setting).ok()?;
    Some(SettingCapability::from_json(
        id.to_string(),
        category,
        &json,
    ))
}
