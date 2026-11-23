//! Settings helpers and device data shaping.
//!
//! Settings arrive as `SettingDto { id, #[serde(flatten)] Setting }`. We flatten
//! each into a `serde_json::Value` so the UI can do the same dynamic access as
//! the React code (`s.spatialAudio?.value === true`).

use std::collections::HashMap;

use serde_json::Value;

use crate::{DeviceStateDto, EqPresetDto};

pub type SettingsMap = HashMap<String, Value>;

pub const SECTION_SPATIAL: &str = "spatial";
pub const SECTION_DEFAULT: &str = "default";
pub const SECTION_CUSTOM: &str = "custom";

/// Build `id -> setting object` from a device snapshot.
/// Each entry serializes to `{ id, type, value, ... }` (serde tag + flatten).
pub fn settings_map(d: &DeviceStateDto) -> SettingsMap {
    let mut m = SettingsMap::new();
    for c in &d.categories {
        for s in &c.settings {
            let v = serde_json::to_value(s).unwrap_or(Value::Null);
            m.insert(s.id.clone(), v);
        }
    }
    m
}

fn setting<'a>(s: &'a SettingsMap, id: &str) -> Option<&'a Value> {
    s.get(id).filter(|v| !v.is_null())
}

fn setting_value<'a>(s: &'a SettingsMap, id: &str) -> Option<&'a Value> {
    setting(s, id)?.get("value")
}

fn as_str_opt(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::String(s) => Some(s.clone()),
        Value::Null => None,
        other => Some(other.to_string()),
    }
}

/// True when the device reports spatial audio as enabled. `value` is a real boolean.
pub fn is_spatial_on(s: &SettingsMap) -> bool {
    setting_value(s, "spatialAudio").and_then(Value::as_bool) == Some(true)
}

/// The active built-in preset id, or None when a custom curve is loaded.
pub fn device_preset_id(s: &SettingsMap) -> Option<String> {
    as_str_opt(setting_value(s, "presetEqualizerProfile"))
}

pub fn has_spatial(s: &SettingsMap) -> bool {
    setting(s, "spatialAudioMode").is_some()
}

pub fn has_default(s: &SettingsMap) -> bool {
    setting(s, "presetEqualizerProfile").is_some()
}

pub fn has_custom(s: &SettingsMap) -> bool {
    setting(s, "volumeAdjustments").is_some()
}

/// Only devices with a writable custom EQ expose the editor.
pub fn has_writable_custom(s: &SettingsMap) -> bool {
    let Some(vol) = setting(s, "volumeAdjustments") else {
        return false;
    };
    vol.get("readOnly").and_then(Value::as_bool) != Some(true)
}

/// Which card should read as selected. Order matters: spatial audio wins because
/// the device keeps its last preset id around while spatial audio is engaged.
pub fn derive_active_section(s: &SettingsMap) -> &'static str {
    if has_spatial(s) && is_spatial_on(s) {
        return SECTION_SPATIAL;
    }
    if has_default(s) && device_preset_id(s).is_some() {
        return SECTION_DEFAULT;
    }
    if has_custom(s) {
        return SECTION_CUSTOM;
    }
    ""
}

/// Fallback when localizedOptions is missing: "BassBooster" -> "Bass Booster".
pub fn pretty_preset_id(id: &str) -> String {
    // Device reports the factory EQ as id "default" — brand name, not "default".
    if id == "default" {
        return "Soundcore Signature".into();
    }
    let mut out = String::with_capacity(id.len() + 4);
    let bytes: Vec<char> = id.chars().collect();
    for (i, &c) in bytes.iter().enumerate() {
        if i > 0 {
            let prev = bytes[i - 1];
            if prev.is_ascii_lowercase() || prev.is_ascii_digit() {
                if c.is_uppercase() {
                    out.push(' ');
                }
            }
        }
        out.push(c);
    }
    out
}

/// Same as pretty_preset_id but for ambient/NC mode ids.
pub fn pretty_select_id(id: &str) -> String {
    pretty_preset_id(id)
}

/// Toggle-row label: "autoPlayPause" -> "Auto Play Pause" (App.tsx `pretty`).
pub fn pretty_setting_id(id: &str) -> String {
    let mut out = String::with_capacity(id.len() + 4);
    let mut first = true;
    for c in id.chars() {
        if c.is_uppercase() {
            if !first {
                out.push(' ');
            }
            out.push(c);
            first = false;
        } else {
            out.push(c);
            first = false;
        }
    }
    // Capitalize first letter
    let mut chars = out.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn option_id(o: &Value) -> String {
    match o {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn option_parts(container: Option<&Value>) -> (Vec<Value>, Vec<Value>) {
    let opts = container
        .and_then(|x| x.get("options"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let localized = container
        .and_then(|x| x.get("localizedOptions"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    (opts, localized)
}

fn entries_from_parts(
    opts: &[Value],
    localized: &[Value],
    prettify: fn(&str) -> String,
) -> Vec<(String, String)> {
    opts.iter()
        .enumerate()
        .map(|(i, o)| {
            let id = option_id(o);
            let label = localized
                .get(i)
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .unwrap_or_else(|| prettify(&id));
            (id, label)
        })
        .collect()
}

fn select_entries_raw(setting: &Value) -> Vec<(String, String)> {
    let (opts, localized) = option_parts(setting.get("setting"));
    entries_from_parts(&opts, &localized, pretty_select_id)
}

fn preset_entries_raw(setting: &Value) -> Vec<(String, String)> {
    let (opts, localized) = option_parts(setting.get("select"));
    entries_from_parts(&opts, &localized, pretty_preset_id)
}

/// Built-in presets as `[(id, label)]`.
pub fn preset_options(s: &SettingsMap) -> Vec<(String, String)> {
    setting(s, "presetEqualizerProfile")
        .map(preset_entries_raw)
        .unwrap_or_default()
}

/// Localized label for a built-in preset id, falling back to a prettified id.
pub fn preset_label(s: &SettingsMap, id: Option<&str>) -> Option<String> {
    let id = id?;
    preset_options(s)
        .into_iter()
        .find(|(oid, _)| oid == id)
        .map(|(_, label)| label)
        .or_else(|| Some(pretty_preset_id(id)))
}

/// Localized label for the current spatial audio mode (Music / Movie / Gaming).
pub fn spatial_mode_label(s: &SettingsMap) -> Option<String> {
    let setting = setting(s, "spatialAudioMode")?;
    let value = as_str_opt(setting.get("value"))?;
    let entries = select_entries_raw(setting);
    entries
        .into_iter()
        .find(|(id, _)| *id == value)
        .map(|(_, label)| label)
        .or(Some(value))
}

/// Generic select options as `[(id, label)]`.
pub fn select_entries(s: &SettingsMap, id: &str) -> Vec<(String, String)> {
    setting(s, id).map(select_entries_raw).unwrap_or_default()
}

pub fn select_settings_with_prefix(
    s: &SettingsMap,
    prefix: &str,
) -> Vec<(String, Vec<(String, String)>)> {
    const SELECT_TYPES: &[&str] = &[
        "select",
        "optionalSelect",
        "modifiableSelect",
        "multiSelect",
        "multiSelectWithRemove",
        "presetEqualizerProfileSelect",
    ];
    let mut out: Vec<(String, Vec<(String, String)>)> = s
        .iter()
        .filter(|(id, _)| id.starts_with(prefix))
        .filter_map(|(id, v)| {
            let ty = v.get("type").and_then(Value::as_str)?;
            if !SELECT_TYPES.contains(&ty) {
                return None;
            }
            let entries = if ty == "presetEqualizerProfileSelect" {
                preset_entries_raw(v)
            } else {
                select_entries_raw(v)
            };
            Some((id.clone(), entries))
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// Localized label for a select value.
pub fn select_label(s: &SettingsMap, id: &str) -> Option<String> {
    let value = as_str_opt(setting_value(s, id))?;
    select_entries(s, id)
        .into_iter()
        .find(|(oid, _)| *oid == value)
        .map(|(_, label)| label)
        .or(Some(value))
}

pub fn has_option(s: &SettingsMap, id: &str, option: &str) -> bool {
    let Some(v) = setting(s, id) else {
        return false;
    };
    let (opts, _) = option_parts(v.get("setting"));
    let (preset_opts, _) = option_parts(v.get("select"));
    opts.iter()
        .chain(preset_opts.iter())
        .any(|o| option_id(o).eq_ignore_ascii_case(option))
}

pub fn first_option(s: &SettingsMap, id: &str) -> Option<String> {
    let v = setting(s, id)?;
    let (opts, _) = option_parts(v.get("setting"));
    let (preset_opts, _) = option_parts(v.get("select"));
    opts.iter().chain(preset_opts.iter()).next().map(option_id)
}

fn bands_key(bands: Option<&Value>) -> String {
    match bands {
        Some(Value::Array(a)) => a
            .iter()
            .map(|v| match v {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .collect::<Vec<_>>()
            .join(","),
        _ => String::new(),
    }
}

pub struct EqMetadata {
    pub bands: usize,
    pub min: i16,
    pub max: i16,
    pub fraction_digits: i32,
    pub band_hz: Vec<u16>,
}

pub fn eq_metadata(s: &SettingsMap) -> Option<EqMetadata> {
    let vol = setting(s, "volumeAdjustments")?;
    let meta = vol.get("setting")?;
    let min = i16::try_from(meta.get("min")?.as_i64()?).ok()?;
    let max = i16::try_from(meta.get("max")?.as_i64()?).ok()?;
    let fraction_digits = i32::try_from(meta.get("fractionDigits")?.as_i64()?).ok()?;
    let band_hz: Vec<u16> = meta
        .get("bandHz")?
        .as_array()?
        .iter()
        .map(|x| u16::try_from(x.as_u64()?).ok())
        .collect::<Option<Vec<_>>>()?;
    let value_len = vol
        .get("value")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    let bands = if value_len > 0 {
        value_len
    } else {
        band_hz.len()
    };
    Some(EqMetadata {
        bands,
        min,
        max,
        fraction_digits,
        band_hz,
    })
}

/// Name of the saved custom preset whose bands match exactly, or None.
pub fn match_custom_preset(s: &SettingsMap, eq_presets: &[EqPresetDto]) -> Option<String> {
    let value = setting_value(s, "volumeAdjustments");
    let key = bands_key(value);
    if key.is_empty() {
        return None;
    }
    let device_bands = value
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);
    eq_presets
        .iter()
        .filter(|p| p.bands.split(',').count() == device_bands)
        .find(|p| p.bands == key)
        .map(|p| p.name.clone())
}

/// Summary text for the Sound Effects row and the Custom EQ card.
pub fn sound_effect_label(s: &SettingsMap, eq_presets: &[EqPresetDto]) -> String {
    match derive_active_section(s) {
        SECTION_SPATIAL => match spatial_mode_label(s) {
            Some(mode) => format!("Spatial Audio · {mode}"),
            None => "Spatial Audio".into(),
        },
        SECTION_DEFAULT => {
            preset_label(s, device_preset_id(s).as_deref()).unwrap_or_else(|| "Default".into())
        }
        SECTION_CUSTOM => match_custom_preset(s, eq_presets).unwrap_or_else(|| "Custom".into()),
        _ => String::new(),
    }
}

/// Product-kind subtitle for a device card (App.tsx `deviceKindLabel`).
pub fn device_kind_label(name: &str) -> String {
    let n = name.to_lowercase();
    if contains_any(&n, &["motion", "boom", "flare", "select", "rave"]) {
        return "Speaker".into();
    }
    if is_overear_name(&n) {
        return "Over-ear headphones".into();
    }
    let nc = if has_nc_token(&n) {
        " · Noise cancelling"
    } else {
        ""
    };
    format!("Wireless earbuds{nc}")
}

/// App.tsx DeviceArt category: speaker / neckband(q11i→earbuds) / overear / earbuds.
pub fn device_art_category(name: &str) -> &'static str {
    let n = name.to_lowercase();
    if contains_any(&n, &["motion", "boom", "flare", "select", "rave"]) {
        "speaker"
    } else if n.contains("q11i") {
        "earbuds"
    } else if is_overear_name(&n) {
        "overear"
    } else {
        "earbuds"
    }
}

fn contains_any(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|x| hay.contains(x))
}

/// `/space|vortex|life tune|life q|(^|\s)q\d/`
fn is_overear_name(n: &str) -> bool {
    if contains_any(n, &["space", "vortex", "life tune", "life q"]) {
        return true;
    }
    // (^|\s)q\d — word-boundary q followed by a digit
    let bytes: Vec<char> = n.chars().collect();
    for i in 0..bytes.len() {
        let boundary = i == 0 || bytes[i - 1].is_whitespace();
        if boundary && bytes[i] == 'q' {
            if bytes
                .get(i + 1)
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                return true;
            }
        }
    }
    false
}

/// `/(^\W)(nc|anc)(\W|$)/` roughly — nc/anc as a standalone word.
fn has_nc_token(n: &str) -> bool {
    let bytes: Vec<char> = n.chars().collect();
    let is_word = |c: char| c.is_ascii_alphanumeric();
    for i in 0..bytes.len() {
        if i == 0 || !is_word(bytes[i - 1]) {
            let two: String = bytes.iter().skip(i).take(2).collect();
            if two == "nc" || two == "anc" {
                let next = bytes.get(i + two.len()).copied();
                if next.map(|c| !is_word(c)).unwrap_or(true) {
                    return true;
                }
            }
        }
    }
    false
}

/// Battery pill list for the control header: `[(label, pct)]`.
/// Labels: L / R / (none for combined) / Case — nulls dropped.
pub fn header_batteries(d: &DeviceStateDto) -> Vec<(String, i32)> {
    let mut out = Vec::new();
    if let Some(v) = d.battery_left {
        out.push(("L".into(), v.clamp(0, 100)));
    }
    if let Some(v) = d.battery_right {
        out.push(("R".into(), v.clamp(0, 100)));
    }
    // Combined is redundant when both buds report individually.
    if d.battery_left.is_none() || d.battery_right.is_none() {
        if let Some(v) = d.battery_combined {
            out.push((String::new(), v.clamp(0, 100)));
        }
    }
    if let Some(v) = d.battery_case {
        out.push(("Case".into(), v.clamp(0, 100)));
    }
    out
}

/// Strip leading "soundcore " from the display name (App.tsx header).
pub fn display_device_name(d: &DeviceStateDto) -> String {
    let n = d.name.trim();
    let lower = n.to_lowercase();
    if let Some(rest) = lower.strip_prefix("soundcore ") {
        // Preserve original casing after the prefix
        let skip = n.len() - rest.len();
        n[skip..].trim().to_string()
    } else if n.is_empty() {
        d.model.clone()
    } else {
        n.to_string()
    }
}

/// Toggle settings for the quick-toggles list.
/// Game mode first, then the design-target order, then alphabetical (App.tsx).
pub fn quick_toggles(s: &SettingsMap) -> Vec<(String, bool)> {
    // Dropped by request: windNoiseSuppression, realTimeAdaptiveNoiseCanceling.
    const HIDDEN: &[&str] = &["windNoiseSuppression", "realTimeAdaptiveNoiseCanceling"];
    // UI-Target/controls_ui_redesign.svg row order.
    const PREFERRED: &[&str] = &["limitHighVolume", "dualConnections", "ldac", "touchTone"];
    let mut toggles: Vec<(String, bool)> = s
        .iter()
        .filter(|(_, v)| v.get("type").and_then(Value::as_str) == Some("toggle"))
        .filter(|(id, _)| id.as_str() != "spatialAudio")
        .filter(|(id, _)| !HIDDEN.contains(&id.as_str()))
        .map(|(id, v)| {
            let on = v.get("value").and_then(Value::as_bool).unwrap_or(false);
            (id.clone(), on)
        })
        .collect();
    // HashMap iteration order is unstable — pin preferred ids, then alpha.
    toggles.sort_by(|a, b| {
        let rank = |id: &str| {
            if id.to_lowercase().contains("gam") {
                return -1;
            }
            PREFERRED
                .iter()
                .position(|p| *p == id)
                .map(|i| i as i32)
                .unwrap_or(i32::MAX)
        };
        rank(&a.0).cmp(&rank(&b.0)).then_with(|| a.0.cmp(&b.0))
    });
    toggles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn select_entries_keeps_device_order_and_localized_labels() {
        let mut m = SettingsMap::new();
        m.insert(
            "spatialAudioMode".into(),
            serde_json::json!({
                "id": "spatialAudioMode",
                "type": "select",
                "setting": {
                    "options": ["movie", "music", "gaming"],
                    "localizedOptions": ["Movie Mode", "Music Mode", "Gaming Mode"],
                },
                "value": "music",
            }),
        );
        assert_eq!(
            select_entries(&m, "spatialAudioMode"),
            vec![
                ("movie".to_string(), "Movie Mode".to_string()),
                ("music".to_string(), "Music Mode".to_string()),
                ("gaming".to_string(), "Gaming Mode".to_string()),
            ]
        );
    }

    #[test]
    fn select_settings_with_prefix_filters_by_type_and_sorts() {
        let mut m = SettingsMap::new();
        m.insert(
            "spatialAudioMode".into(),
            serde_json::json!({
                "id": "spatialAudioMode",
                "type": "select",
                "setting": {
                    "options": ["music", "movie"],
                    "localizedOptions": ["Music", "Movie"],
                },
                "value": "music",
            }),
        );
        m.insert(
            "spatialAudioMusicMode".into(),
            serde_json::json!({
                "id": "spatialAudioMusicMode",
                "type": "optionalSelect",
                "setting": {
                    "options": ["bass", "treble"],
                    "localizedOptions": ["Bass", "Treble"],
                },
                "value": "bass",
            }),
        );
        m.insert(
            "spatialAudio".into(),
            serde_json::json!({
                "id": "spatialAudio",
                "type": "toggle",
                "value": true,
            }),
        );

        let found = select_settings_with_prefix(&m, "spatial");
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].0, "spatialAudioMode");
        assert_eq!(
            found[0].1,
            vec![
                ("music".to_string(), "Music".to_string()),
                ("movie".to_string(), "Movie".to_string()),
            ]
        );
        assert_eq!(found[1].0, "spatialAudioMusicMode");
        assert_eq!(
            found[1].1,
            vec![
                ("bass".to_string(), "Bass".to_string()),
                ("treble".to_string(), "Treble".to_string()),
            ]
        );
        assert!(found.iter().all(|(id, _)| id.as_str() != "spatialAudio"));

        let mut none = SettingsMap::new();
        none.insert(
            "presetEqualizerProfile".into(),
            serde_json::json!({
                "id": "presetEqualizerProfile",
                "type": "select",
                "setting": { "options": ["signature"], "localizedOptions": ["Signature"] },
                "value": "signature",
            }),
        );
        assert!(select_settings_with_prefix(&none, "spatial").is_empty());
    }

    #[test]
    fn eq_metadata_parses_a3953_shape() {
        let mut m = SettingsMap::new();
        m.insert(
            "volumeAdjustments".into(),
            serde_json::json!({
                "id": "volumeAdjustments",
                "type": "equalizer",
                "setting": {
                    "bandHz": [100, 200, 400, 800, 1600, 3200, 6400, 12800],
                    "fractionDigits": 1,
                    "min": -120,
                    "max": 134,
                },
                "value": [0, 0, 0, 0, 0, 0, 0, 0],
            }),
        );
        let meta = eq_metadata(&m).unwrap();
        assert_eq!(meta.bands, 8);
        assert_eq!(meta.min, -120);
        assert_eq!(meta.max, 134);
        assert_eq!(meta.fraction_digits, 1);
        assert_eq!(
            meta.band_hz,
            vec![100, 200, 400, 800, 1600, 3200, 6400, 12800]
        );
    }

    #[test]
    fn eq_metadata_returns_none_without_metadata() {
        let mut m = SettingsMap::new();
        m.insert(
            "volumeAdjustments".into(),
            serde_json::json!({
                "id": "volumeAdjustments",
                "type": "equalizer",
                "value": [0, 0],
            }),
        );
        assert!(eq_metadata(&m).is_none());
        assert!(eq_metadata(&SettingsMap::new()).is_none());
    }

    #[test]
    fn has_option_and_first_option_use_option_ids() {
        let mut m = SettingsMap::new();
        m.insert(
            "spatialAudioMode".into(),
            serde_json::json!({
                "id": "spatialAudioMode",
                "type": "select",
                "setting": {
                    "options": ["music", "movie"],
                    "localizedOptions": ["Music", "Movie"],
                },
                "value": "music",
            }),
        );
        assert!(has_option(&m, "spatialAudioMode", "MUSIC"));
        assert!(has_option(&m, "spatialAudioMode", "Movie"));
        assert!(!has_option(&m, "spatialAudioMode", "gaming"));
        assert!(!has_option(&m, "missing", "music"));
        assert_eq!(
            first_option(&m, "spatialAudioMode").as_deref(),
            Some("music")
        );
        assert_eq!(first_option(&m, "missing"), None);
    }

    #[test]
    fn match_custom_preset_ignores_different_band_counts() {
        let mut m = SettingsMap::new();
        m.insert(
            "volumeAdjustments".into(),
            serde_json::json!({
                "id": "volumeAdjustments",
                "type": "equalizer",
                "setting": {
                    "bandHz": [100, 200, 400, 800, 1600, 3200, 6400, 12800],
                    "fractionDigits": 1,
                    "min": -120,
                    "max": 134,
                },
                "value": [0, 1, 2, 3, 4, 5, 6, 7],
            }),
        );
        let presets = vec![
            EqPresetDto {
                id: 1,
                name: "Eight Band".into(),
                bands: "0,1,2,3,4,5,6,7".into(),
                model: "A3953".into(),
            },
            EqPresetDto {
                id: 2,
                name: "Ten Band".into(),
                bands: "0,1,2,3,4,5,6,7,8,9".into(),
                model: "R60i".into(),
            },
        ];
        assert_eq!(
            match_custom_preset(&m, &presets).as_deref(),
            Some("Eight Band")
        );

        let mut collision = SettingsMap::new();
        collision.insert(
            "volumeAdjustments".into(),
            serde_json::json!({ "value": ["1,2", "3"] }),
        );
        assert_eq!(match_custom_preset(&collision, &presets), None);
    }
}
