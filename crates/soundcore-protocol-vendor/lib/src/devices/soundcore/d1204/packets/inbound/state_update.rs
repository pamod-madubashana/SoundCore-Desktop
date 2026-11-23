use nom::{
    IResult, Parser,
    bytes::complete::take,
    error::{ContextError, ParseError, context},
    number::complete::le_u8,
};

use crate::devices::soundcore::{
    common::{
        macros::state_update_packet_module,
        packet::{self, Command, inbound::FromPacketBody, outbound::ToPacket},
        structures::{
            BatteryLevel, CaseBatteryLevel, DualBattery, EqualizerConfiguration, IsBatteryCharging,
            SingleBattery, TwsStatus, VolumeAdjustments,
        },
    },
    d1204::{self, state::D1204State, structures::AmbientSoundMode},
};

pub const CUSTOM_EQ_ID: u16 = 0xFF1E;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tlv<'a> {
    pub tag: u8,
    pub value: &'a [u8],
}

pub fn take_tlv<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
    input: &'a [u8],
) -> IResult<&'a [u8], Tlv<'a>, E> {
    let (input, (tag, length)) = context("tlv header", (le_u8, le_u8)).parse_complete(input)?;
    let (input, value) = context("tlv value", take(length)).parse_complete(input)?;
    Ok((input, Tlv { tag, value }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct D1204StateUpdatePacket {
    pub tws_status: TwsStatus,
    pub battery: DualBattery,
    pub case_battery_level: CaseBatteryLevel,
    pub sound_modes: d1204::structures::SoundModes,
    pub spatial_audio: d1204::structures::SpatialAudio,
    pub equalizer_configuration: EqualizerConfiguration<1, 8, -120, 134, 1>,
}

impl Default for D1204StateUpdatePacket {
    fn default() -> Self {
        Self {
            tws_status: TwsStatus::default(),
            battery: DualBattery::default(),
            case_battery_level: CaseBatteryLevel::default(),
            sound_modes: Default::default(),
            spatial_audio: Default::default(),
            equalizer_configuration: Default::default(),
        }
    }
}

fn scaled_battery(raw: u8) -> u8 {
    if raw == 0xFF { 0 } else { raw.min(100) }
}

fn apply_battery_side(side: &mut SingleBattery, value: &[u8]) {
    if let Some(&charging) = value.first() {
        side.is_charging = if charging == 1 {
            IsBatteryCharging::Yes
        } else {
            IsBatteryCharging::No
        };
    }
    if let Some(&level) = value.get(1) {
        side.level = BatteryLevel(scaled_battery(level));
    }
}

fn apply_tlv<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
    packet: &mut D1204StateUpdatePacket,
    eq_preset_id: &mut u16,
    tlv: Tlv<'a>,
) {
    match tlv.tag {
        2 => {
            if let Some(&value) = tlv.value.first() {
                packet.tws_status.is_connected = value == 1;
            }
        }
        3 => apply_battery_side(&mut packet.battery.left, tlv.value),
        4 => apply_battery_side(&mut packet.battery.right, tlv.value),
        8 => {
            if let Some(&value) = tlv.value.last() {
                packet.case_battery_level = CaseBatteryLevel(BatteryLevel(value.min(100)));
            }
        }
        11 => {
            if tlv.value.len() >= 2 {
                let id = u16::from_le_bytes([tlv.value[0], tlv.value[1]]);
                *eq_preset_id =
                    if tlv.value[1] == 0xFE || tlv.value[1] == 0xFF || id == CUSTOM_EQ_ID {
                        CUSTOM_EQ_ID
                    } else {
                        id
                    };
            }
        }
        12 => {
            if tlv.value.len() >= 32 {
                let mut gains = [0i16; 8];
                for (index, gain) in gains.iter_mut().enumerate() {
                    let start = index * 4;
                    let bytes: [u8; 4] = tlv.value[start..start + 4].try_into().unwrap();
                    let value = f32::from_le_bytes(bytes);
                    *gain = (value.clamp(-120.0, 134.0) * 10.0).round() as i16;
                }
                packet.equalizer_configuration = EqualizerConfiguration::new_all_bands_present(
                    *eq_preset_id,
                    [VolumeAdjustments::new(gains)],
                );
            }
        }
        27 => {
            let second = tlv.value.get(1).copied().unwrap_or(0);
            let third = tlv.value.get(2).copied().unwrap_or(0);
            packet.sound_modes.wind_noise.is_suppression_enabled = second & 0x08 != 0;
            packet.sound_modes.large_noise_suppression = third & 0x04 != 0;
            packet.spatial_audio.is_enabled = third & 0x02 != 0;
        }
        36 => {
            let m_option = tlv.value.first().copied().unwrap_or(0);
            packet.sound_modes.ambient_sound_mode = match m_option {
                1 => AmbientSoundMode::NoiseCanceling,
                2 => AmbientSoundMode::Transparency,
                _ => AmbientSoundMode::Normal,
            };
            packet.sound_modes.anc_automation_mode = tlv.value.get(1).copied().unwrap_or(0);
            packet.sound_modes.m_trans_option = tlv.value.get(2).copied().unwrap_or(0);
        }
        37 => {
            let anc = tlv.value.first().copied().unwrap_or(0);
            packet.sound_modes.anc_option_auto = (anc >> 4).clamp(1, 5);
            packet.sound_modes.anc_option_manual = (anc & 0x0F).clamp(1, 5);
            let trans = tlv.value.get(1).copied().unwrap_or(0);
            packet.sound_modes.trans_option_auto = (trans >> 4).clamp(1, 5);
            packet.sound_modes.trans_option_manual = (trans & 0x0F).clamp(1, 5);
        }
        38 => {
            packet.sound_modes.anc_multi_scene_model = tlv.value.first().copied().unwrap_or(0);
            packet.sound_modes.trans_multi_scene_model = tlv.value.get(1).copied().unwrap_or(0);
        }
        44 => {
            if let Ok((_, spatial)) = d1204::structures::SpatialAudio::take::<E>(tlv.value) {
                packet.spatial_audio = spatial;
            }
        }
        46 => {
            if let Some(&mask) = tlv.value.first() {
                packet.sound_modes.is_adaptive_selected = mask & 0x08 != 0;
                packet.sound_modes.ambient_sound_mode = if mask & 0x01 != 0 {
                    AmbientSoundMode::NoiseCanceling
                } else if mask & 0x02 != 0 {
                    AmbientSoundMode::Transparency
                } else if mask & 0x08 != 0 {
                    AmbientSoundMode::Adaptive
                } else if mask & 0x04 != 0 {
                    AmbientSoundMode::Normal
                } else {
                    packet.sound_modes.ambient_sound_mode
                };
            }
        }
        _ => {}
    }
}

impl FromPacketBody for D1204StateUpdatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        let mut remaining = input;
        let mut packet = Self::default();
        let mut eq_preset_id = 0u16;
        while !remaining.is_empty() {
            match take_tlv::<E>(remaining) {
                Ok((rest, tlv)) => {
                    apply_tlv::<E>(&mut packet, &mut eq_preset_id, tlv);
                    remaining = rest;
                }
                Err(_) => break,
            }
        }
        Ok((remaining, packet))
    }
}

fn push_tlv(output: &mut Vec<u8>, tag: u8, value: &[u8]) {
    output.push(tag);
    output.push(value.len() as u8);
    output.extend_from_slice(value);
}

impl ToPacket for D1204StateUpdatePacket {
    type DirectionMarker = packet::InboundMarker;

    fn command(&self) -> Command {
        packet::inbound::STATE_COMMAND
    }

    fn body(&self) -> Vec<u8> {
        let mut output = Vec::new();
        push_tlv(&mut output, 2, &[u8::from(self.tws_status.is_connected)]);
        push_tlv(
            &mut output,
            3,
            &[
                self.battery.left.is_charging as u8,
                self.battery.left.level.0,
            ],
        );
        push_tlv(
            &mut output,
            4,
            &[
                self.battery.right.is_charging as u8,
                self.battery.right.level.0,
            ],
        );
        push_tlv(&mut output, 8, &[self.case_battery_level.0.0]);
        let preset_id = self.equalizer_configuration.preset_id();
        push_tlv(&mut output, 11, &preset_id.to_le_bytes());
        let gains = self
            .equalizer_configuration
            .volume_adjustments_channel_1()
            .map(|adjustments| adjustments.adjustments())
            .copied()
            .unwrap_or_default();
        let mut eq_values = Vec::with_capacity(32);
        for gain in gains {
            eq_values.extend_from_slice(&((gain as f32) / 10.0).to_le_bytes());
        }
        push_tlv(&mut output, 12, &eq_values);
        push_tlv(
            &mut output,
            27,
            &[
                0,
                u8::from(self.sound_modes.wind_noise.is_suppression_enabled) << 3,
                u8::from(self.sound_modes.large_noise_suppression) << 2
                    | u8::from(self.spatial_audio.is_enabled) << 1,
            ],
        );
        push_tlv(
            &mut output,
            36,
            &[
                self.sound_modes.m_option(),
                self.sound_modes.anc_automation_mode,
                self.sound_modes.m_trans_option,
            ],
        );
        push_tlv(
            &mut output,
            37,
            &[
                (self.sound_modes.anc_option_auto.clamp(1, 5) << 4)
                    | self.sound_modes.anc_option_manual.clamp(1, 5),
                (self.sound_modes.trans_option_auto.clamp(1, 5) << 4)
                    | self.sound_modes.trans_option_manual.clamp(1, 5),
            ],
        );
        push_tlv(
            &mut output,
            38,
            &[
                self.sound_modes.anc_multi_scene_model,
                self.sound_modes.trans_multi_scene_model,
            ],
        );
        push_tlv(&mut output, 44, &self.spatial_audio.bytes());
        let mode_bit = match self.sound_modes.ambient_sound_mode {
            AmbientSoundMode::NoiseCanceling => 0x01,
            AmbientSoundMode::Transparency => 0x02,
            AmbientSoundMode::Normal => 0x04,
            AmbientSoundMode::Adaptive => 0x08,
        };
        push_tlv(&mut output, 46, &[mode_bit]);
        push_tlv(&mut output, 200, &[1, 2, 3]);
        output
    }
}

state_update_packet_module!(D1204State, D1204StateUpdatePacket);

#[cfg(test)]
mod tests {
    use nom_language::error::VerboseError;

    use super::*;
    use crate::devices::soundcore::d1204::structures::{SpatialAudioMode, SpatialEffectMode};

    fn parse(body: &[u8]) -> D1204StateUpdatePacket {
        D1204StateUpdatePacket::take::<VerboseError<_>>(body)
            .unwrap()
            .1
    }

    fn tlv(tag: u8, value: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_tlv(&mut bytes, tag, value);
        bytes
    }

    #[test]
    fn inbound_header_and_length_are_parsed() {
        let body = tlv(2, &[1]);
        let packet = packet::Inbound::new(packet::Command([1, 1]), body.clone());
        let bytes = packet.bytes_with_checksum();
        assert_eq!(&bytes[..9], &[0x09, 0xff, 0x00, 0x00, 0x01, 1, 1, 13, 0]);
        let (remainder, parsed) =
            packet::Inbound::take_with_checksum::<VerboseError<_>>(&bytes).unwrap();
        assert!(remainder.is_empty());
        assert_eq!(parsed.command, packet::Command([1, 1]));
        assert_eq!(parsed.body, body);
    }

    #[test]
    fn tlv_records_are_iterated_and_unknown_tags_skipped() {
        let mut body = tlv(99, &[9, 8, 7]);
        body.extend(tlv(2, &[1]));
        body.extend(tlv(250, &[]));
        body.extend(tlv(46, &[0x01]));
        let mut remaining = body.as_slice();
        let mut tags = Vec::new();
        while !remaining.is_empty() {
            let (rest, record) = take_tlv::<VerboseError<_>>(remaining).unwrap();
            tags.push((record.tag, record.value.to_vec()));
            remaining = rest;
        }
        assert_eq!(
            tags,
            vec![
                (99, vec![9, 8, 7]),
                (2, vec![1]),
                (250, vec![]),
                (46, vec![1])
            ]
        );
    }

    #[test]
    fn unknown_tags_do_not_fail_whole_packet() {
        let mut body = tlv(250, &[1, 2, 3]);
        body.extend(tlv(2, &[1]));
        body.extend(tlv(99, &[4, 5]));
        body.extend(tlv(46, &[0x04]));
        let packet = parse(&body);
        assert!(packet.tws_status.is_connected);
        assert_eq!(
            packet.sound_modes.ambient_sound_mode,
            AmbientSoundMode::Normal
        );
    }

    #[test]
    fn sound_mode_mask_and_anc_nibbles_are_extracted() {
        let mut body = tlv(36, &[1, 0, 2]);
        body.extend(tlv(37, &[0x53, 0x21]));
        body.extend(tlv(38, &[2, 1]));
        body.extend(tlv(46, &[0x01]));
        let packet = parse(&body);
        assert_eq!(
            packet.sound_modes.ambient_sound_mode,
            AmbientSoundMode::NoiseCanceling
        );
        assert!(!packet.sound_modes.is_adaptive_selected);
        assert_eq!(packet.sound_modes.anc_option_manual, 3);
        assert_eq!(packet.sound_modes.anc_option_auto, 5);
        assert_eq!(packet.sound_modes.trans_option_manual, 1);
        assert_eq!(packet.sound_modes.trans_option_auto, 2);
        assert_eq!(packet.sound_modes.anc_multi_scene_model, 2);
        assert_eq!(packet.sound_modes.trans_multi_scene_model, 1);
    }

    #[test]
    fn adaptive_sound_mode_bit_is_extracted() {
        let packet = parse(&tlv(46, &[0x08]));
        assert_eq!(
            packet.sound_modes.ambient_sound_mode,
            AmbientSoundMode::Adaptive
        );
        assert!(packet.sound_modes.is_adaptive_selected);
    }

    #[test]
    fn battery_values_are_extracted_from_tags() {
        let mut body = tlv(3, &[0, 87]);
        body.extend(tlv(4, &[1, 92]));
        body.extend(tlv(8, &[64]));
        let packet = parse(&body);
        assert_eq!(packet.battery.left.level.0, 87);
        assert_eq!(packet.battery.right.level.0, 92);
        assert!(bool::from(packet.battery.right.is_charging));
        assert_eq!(packet.case_battery_level.0.0, 64);
    }

    #[test]
    fn battery_sentinel_is_normalized_to_zero() {
        let packet = parse(&tlv(3, &[0, 0xFF]));
        assert_eq!(packet.battery.left.level.0, 0);
    }

    #[test]
    fn custom_eq_floats_are_extracted() {
        let gains = [1.0f32, 4.0, -0.9, 2.4, 0.0, -1.5, 3.25, -2.0];
        let mut body = tlv(11, &[0x1E, 0xFF]);
        let mut values = Vec::new();
        for gain in gains {
            values.extend_from_slice(&gain.to_le_bytes());
        }
        body.extend(tlv(12, &values));
        let packet = parse(&body);
        assert_eq!(packet.equalizer_configuration.preset_id(), CUSTOM_EQ_ID);
        let adjustments = packet
            .equalizer_configuration
            .volume_adjustments_channel_1()
            .unwrap()
            .adjustments();
        let expected = gains.map(|gain| (gain * 10.0).round() as i16);
        assert_eq!(*adjustments, expected);
    }

    #[test]
    fn spatial_tag_is_extracted() {
        let packet = parse(&tlv(44, &[1, 2, 3]));
        assert!(packet.spatial_audio.is_enabled);
        assert_eq!(
            packet.spatial_audio.effect_mode,
            SpatialEffectMode::HeadTracking
        );
        assert_eq!(packet.spatial_audio.sound_mode, SpatialAudioMode::Theater);
    }

    #[test]
    fn truncated_tail_is_ignored() {
        let mut body = tlv(2, &[1]);
        body.extend_from_slice(&[46, 4, 1]);
        let packet = parse(&body);
        assert!(packet.tws_status.is_connected);
    }
}
