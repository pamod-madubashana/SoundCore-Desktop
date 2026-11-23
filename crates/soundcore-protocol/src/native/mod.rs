use crate::transport::{self, RX_PREFIX};

pub const STATE_BODY_LEN: usize = 139;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    Transport(transport::DecodeError),
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => write!(f, "empty state packet"),
            Self::Transport(error) => write!(f, "transport decode error: {error:?}"),
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct D1202State {
    body: Vec<u8>,
}

impl D1202State {
    pub fn from_body(body: &[u8]) -> Self {
        Self {
            body: body.to_vec(),
        }
    }

    pub fn body(&self) -> &[u8] {
        &self.body
    }

    fn byte(&self, offset: usize) -> Option<u8> {
        self.body.get(offset).copied()
    }

    fn u16_le(&self, offset: usize) -> Option<u16> {
        Some(u16::from_le_bytes([
            self.byte(offset)?,
            self.byte(offset + 1)?,
        ]))
    }

    fn ascii(&self, start: usize, len: usize) -> String {
        let end = start.saturating_add(len);
        let slice = self.body.get(start..end).unwrap_or(&[]);
        let mut out = Vec::with_capacity(slice.len());
        for &byte in slice {
            if byte == 0 || byte == 0xFF {
                break;
            }
            if byte.is_ascii_graphic() || byte == b' ' {
                out.push(byte);
            }
        }
        String::from_utf8_lossy(&out).into_owned()
    }

    fn battery_level(&self, offset: usize) -> String {
        match self.byte(offset) {
            Some(raw) if raw != 0xFF => format!("{}/10", u16::from(raw).saturating_add(1).min(10)),
            _ => "0/10".to_string(),
        }
    }

    pub fn battery_level_left(&self) -> String {
        self.battery_level(2)
    }

    pub fn battery_level_right(&self) -> String {
        self.battery_level(3)
    }

    pub fn case_battery_level(&self) -> String {
        self.battery_level(37)
    }

    pub fn is_charging_left(&self) -> String {
        yes_no(self.byte(4))
    }

    pub fn is_charging_right(&self) -> String {
        yes_no(self.byte(5))
    }

    pub fn ambient_sound_mode(&self) -> String {
        match self.byte(119) {
            Some(0) => "NoiseCanceling",
            Some(1) => "Transparency",
            Some(2) => "Normal",
            _ => "Normal",
        }
        .to_string()
    }

    pub fn noise_canceling_mode(&self) -> String {
        match self.byte(122) {
            Some(0) => "Manual",
            Some(1) => "Adaptive",
            Some(2) => "MultiScene",
            _ => "Manual",
        }
        .to_string()
    }

    pub fn transparency_mode(&self) -> String {
        match self.byte(121) {
            Some(0) => "FullyTransparent",
            Some(1) => "VocalMode",
            _ => "FullyTransparent",
        }
        .to_string()
    }

    pub fn manual_noise_canceling(&self) -> String {
        self.byte(120)
            .map_or_else(|| "0".to_string(), |b| (b >> 4).to_string())
    }

    pub fn adaptive_noise_canceling(&self) -> String {
        match self.byte(120).map(|b| b & 0x0F) {
            Some(0) => "Weak".to_string(),
            Some(1) => "Strong".to_string(),
            Some(other) => other.to_string(),
            None => "Weak".to_string(),
        }
    }

    pub fn multi_scene_noise_canceling(&self) -> String {
        match self.byte(125) {
            Some(0) => "Transport",
            Some(1) => "Outdoor",
            Some(2) => "Indoor",
            _ => "Transport",
        }
        .to_string()
    }

    pub fn wind_noise_suppression(&self) -> String {
        true_false(self.byte(123).map(|b| b & 0x01 != 0))
    }

    pub fn real_time_adaptive_noise_canceling(&self) -> String {
        true_false(self.byte(126).map(|b| b != 0))
    }

    pub fn volume_adjustments(&self) -> String {
        let mut values = Vec::with_capacity(8);
        for index in 0..8 {
            let adjustment = match self.byte(40 + index) {
                Some(raw) if raw != 0xFF => i16::from(raw) - 120,
                _ => 0,
            };
            values.push(adjustment.to_string());
        }
        format!("[{}]", values.join(","))
    }

    pub fn preset_equalizer_profile(&self) -> String {
        match self.u16_le(38) {
            Some(0xFFFF) | None => "None".to_string(),
            Some(id) => PRESET_PROFILES
                .get(usize::from(id))
                .copied()
                .unwrap_or("None")
                .to_string(),
        }
    }

    pub fn spatial_audio(&self) -> String {
        true_false(self.byte(137).map(|b| b != 0))
    }

    pub fn spatial_audio_mode(&self) -> String {
        match self.byte(138) {
            Some(0) => "Music",
            Some(2) => "Movie",
            Some(3) => "Gaming",
            _ => "Music",
        }
        .to_string()
    }

    pub fn serial_number(&self) -> String {
        self.ascii(16, 16)
    }

    pub fn firmware_version_left(&self) -> String {
        self.ascii(6, 5)
    }

    pub fn firmware_version_right(&self) -> String {
        self.ascii(11, 5)
    }

    pub fn firmware_version(&self) -> String {
        let left = self.firmware_version_left();
        let right = self.firmware_version_right();
        if left >= right { left } else { right }
    }

    pub fn left_single_press(&self) -> String {
        button_action(self.byte(110))
    }

    pub fn right_single_press(&self) -> String {
        button_action(self.byte(111))
    }

    pub fn left_double_press(&self) -> String {
        button_action(self.byte(112))
    }

    pub fn right_double_press(&self) -> String {
        button_action(self.byte(113))
    }

    pub fn left_triple_press(&self) -> String {
        button_action(self.byte(114))
    }

    pub fn right_triple_press(&self) -> String {
        button_action(self.byte(115))
    }

    pub fn left_long_press(&self) -> String {
        button_action(self.byte(116))
    }

    pub fn right_long_press(&self) -> String {
        button_action(self.byte(117))
    }

    pub fn limit_high_volume(&self) -> String {
        true_false(self.byte(134).map(|b| b != 0))
    }

    pub fn limit_high_volume_db_limit(&self) -> String {
        self.byte(135)
            .map_or_else(|| "0".to_string(), |b| b.to_string())
    }

    pub fn limit_high_volume_refresh_rate(&self) -> String {
        match self.byte(136) {
            Some(0) => "RealTime",
            Some(1) => "10s",
            Some(2) => "1m",
            _ => "RealTime",
        }
        .to_string()
    }

    pub fn dual_connections(&self) -> String {
        true_false(self.byte(131).map(|b| b != 0))
    }

    pub fn touch_tone(&self) -> String {
        true_false(self.byte(128).map(|b| b != 0))
    }

    pub fn low_battery_prompt(&self) -> String {
        true_false(self.byte(129).map(|b| b != 0))
    }

    pub fn auto_power_off(&self) -> String {
        match self.byte(133) {
            Some(0) => "5m",
            Some(1) => "10m",
            Some(2) => "30m",
            Some(3) => "60m",
            Some(4) => "300m",
            _ => "30m",
        }
        .to_string()
    }

    pub fn ldac(&self) -> String {
        true_false(self.byte(130).map(|b| b != 0))
    }

    pub fn value_for_key(&self, key: &str) -> Option<String> {
        let value = match key {
            "BatteryLevelLeft" => self.battery_level_left(),
            "BatteryLevelRight" => self.battery_level_right(),
            "CaseBatteryLevel" => self.case_battery_level(),
            "IsChargingLeft" => self.is_charging_left(),
            "IsChargingRight" => self.is_charging_right(),
            "AmbientSoundMode" => self.ambient_sound_mode(),
            "NoiseCancelingMode" => self.noise_canceling_mode(),
            "TransparencyMode" => self.transparency_mode(),
            "AdaptiveNoiseCanceling" => self.adaptive_noise_canceling(),
            "ManualNoiseCanceling" => self.manual_noise_canceling(),
            "MultiSceneNoiseCanceling" => self.multi_scene_noise_canceling(),
            "WindNoiseSuppression" => self.wind_noise_suppression(),
            "RealTimeAdaptiveNoiseCanceling" => self.real_time_adaptive_noise_canceling(),
            "VolumeAdjustments" => self.volume_adjustments(),
            "PresetEqualizerProfile" => self.preset_equalizer_profile(),
            "LeftSinglePress" => self.left_single_press(),
            "LeftDoublePress" => self.left_double_press(),
            "LeftTriplePress" => self.left_triple_press(),
            "LeftLongPress" => self.left_long_press(),
            "RightSinglePress" => self.right_single_press(),
            "RightDoublePress" => self.right_double_press(),
            "RightTriplePress" => self.right_triple_press(),
            "RightLongPress" => self.right_long_press(),
            "LimitHighVolume" => self.limit_high_volume(),
            "LimitHighVolumeDbLimit" => self.limit_high_volume_db_limit(),
            "LimitHighVolumeRefreshRate" => self.limit_high_volume_refresh_rate(),
            "DualConnections" => self.dual_connections(),
            "TouchTone" => self.touch_tone(),
            "LowBatteryPrompt" => self.low_battery_prompt(),
            "AutoPowerOff" => self.auto_power_off(),
            "Ldac" => self.ldac(),
            "FirmwareVersionLeft" => self.firmware_version_left(),
            "FirmwareVersionRight" => self.firmware_version_right(),
            "FirmwareVersion" => self.firmware_version(),
            "SerialNumber" => self.serial_number(),
            "SpatialAudio" => self.spatial_audio(),
            "SpatialAudioMode" => self.spatial_audio_mode(),
            _ => return None,
        };
        Some(value)
    }
}

pub fn parse_state_packet(input: &[u8]) -> Result<D1202State, ParseError> {
    if input.is_empty() {
        return Err(ParseError::Empty);
    }
    if input.starts_with(&RX_PREFIX) {
        let (frame, _) = transport::decode(input).map_err(ParseError::Transport)?;
        Ok(D1202State::from_body(&frame.body))
    } else {
        Ok(D1202State::from_body(input))
    }
}

fn true_false(value: Option<bool>) -> String {
    if value.unwrap_or(false) {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

fn yes_no(value: Option<u8>) -> String {
    if value == Some(1) {
        "Yes".to_string()
    } else {
        "No".to_string()
    }
}

fn button_action(byte: Option<u8>) -> String {
    match byte.map(|b| b & 0x0F) {
        Some(0x03) => "NextSong",
        Some(0x04) => "AmbientSoundMode",
        Some(0x06) => "PlayPause",
        Some(0x0F) | None => "None",
        Some(_) => "Unknown",
    }
    .to_string()
}

const PRESET_PROFILES: &[&str] = &[
    "SoundcoreSignature",
    "Acoustic",
    "BassBoost",
    "Classical",
    "Country",
    "Daily",
    "Dance",
    "Electronic",
    "Flat",
    "HipHop",
    "Jazz",
    "Latin",
    "Lounge",
    "Metal",
    "Movie",
    "Podcast",
    "Pop",
    "Reggae",
    "RnB",
    "Rock",
    "TrebleBoost",
    "Vocal",
];

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Fixture {
        state_vectors: Vec<StateVector>,
    }

    #[derive(Deserialize)]
    struct StateVector {
        packet_hex: String,
        expected: serde_json::Map<String, serde_json::Value>,
    }

    fn unhex(text: &str) -> Option<Vec<u8>> {
        if text.len() % 2 != 0 || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
            .collect()
    }

    fn expected_string(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(text) => text.clone(),
            other => other.to_string(),
        }
    }

    #[test]
    fn fixture_expectations_match_accessors() {
        let json = include_str!("../../fixtures/d1202.sanitized.json");
        let fixture: Fixture = serde_json::from_str(json).expect("fixture JSON");
        assert!(
            !fixture.state_vectors.is_empty(),
            "fixture has no state vectors"
        );
        for vector in &fixture.state_vectors {
            let bytes = unhex(&vector.packet_hex).expect("valid hex");
            let state = parse_state_packet(&bytes).expect("parse state packet");
            for (key, expected) in &vector.expected {
                let actual = state
                    .value_for_key(key)
                    .unwrap_or_else(|| panic!("no accessor for fixture key {key}"));
                assert_eq!(actual, expected_string(expected), "mismatch for {key}");
            }
        }
    }

    #[test]
    fn tolerates_trailing_and_short_bodies() {
        let json = include_str!("../../fixtures/d1202.sanitized.json");
        let fixture: Fixture = serde_json::from_str(json).expect("fixture JSON");
        let bytes = unhex(&fixture.state_vectors[0].packet_hex).expect("valid hex");
        assert!(bytes.len() > STATE_BODY_LEN);
        let full = parse_state_packet(&bytes).expect("full body");
        let short = parse_state_packet(&bytes[..STATE_BODY_LEN]).expect("short body");
        let tiny = parse_state_packet(&bytes[..2]).expect("tiny body");
        assert_eq!(full.serial_number(), short.serial_number());
        assert_eq!(full.spatial_audio_mode(), short.spatial_audio_mode());
        assert_eq!(tiny.serial_number(), "");
        assert_eq!(tiny.battery_level_left(), "0/10");
        assert_eq!(parse_state_packet(&[]), Err(ParseError::Empty));
    }

    #[derive(Deserialize)]
    struct CaptureFixture {
        rx_frames: Vec<String>,
        tx_frames: Vec<String>,
    }

    #[test]
    fn parses_real_r60i_capture_frames() {
        let json = include_str!("../../fixtures/r60i_capture.sanitized.json");
        let fixture: CaptureFixture = serde_json::from_str(json).expect("capture JSON");
        assert!(!fixture.rx_frames.is_empty(), "no RX frames captured");
        assert!(!fixture.tx_frames.is_empty(), "no TX frames captured");
        let mut states = 0;
        for frame_hex in &fixture.rx_frames {
            let bytes = unhex(frame_hex).expect("valid capture hex");
            let mut offset = 0;
            while offset < bytes.len() {
                let (frame, consumed) =
                    transport::decode(&bytes[offset..]).expect("real frame must pass codec");
                let raw = &bytes[offset..offset + consumed];
                offset += consumed;
                if raw.len() < 20 || frame.group != 1 || frame.command != 1 {
                    continue;
                }
                let state = parse_state_packet(raw).expect("real state frame must parse");
                assert!(
                    state.battery_level_left().ends_with("/10"),
                    "unexpected battery value"
                );
                states += 1;
            }
        }
        assert!(states > 0, "no real state frames were parsed");
    }

    #[test]
    fn parses_framed_state_packet() {
        let json = include_str!("../../fixtures/d1202.sanitized.json");
        let fixture: Fixture = serde_json::from_str(json).expect("fixture JSON");
        let body = unhex(&fixture.state_vectors[0].packet_hex).expect("valid hex");
        let mut raw = vec![0x09, 0xFF, 0x00, 0x00, 0x01, 0x01, 0x01];
        let total = (raw.len() + 2 + body.len() + 1) as u16;
        raw.extend_from_slice(&total.to_le_bytes());
        raw.extend_from_slice(&body);
        raw.push(transport::checksum(&raw));
        let state = parse_state_packet(&raw).expect("framed packet");
        assert_eq!(state.serial_number(), "REDACTED00000000");
        assert_eq!(state.battery_level_left(), "10/10");
    }
}
