use nom::{
    IResult, Parser,
    combinator::map,
    error::{ContextError, ParseError, context},
    number::complete::le_u8,
};

use crate::devices::soundcore::common::{
    macros::sound_mode_enum,
    modules::sound_modes_v2::ToPacketBody,
    packet::{self, inbound::FromPacketBody, parsing::take_bool},
    structures::WindNoise,
};

sound_mode_enum!(
    pub enum AmbientSoundMode {
        NoiseCanceling = 0,
        Transparency = 1,
        Normal = 2,
        Adaptive = 3,
    }
);

sound_mode_enum!(
    pub enum SpatialAudioMode {
        Music = 0,
        Movie = 1,
        Gaming = 2,
        Theater = 3,
    }
);

sound_mode_enum!(
    pub enum SpatialEffectMode {
        Fixed = 1,
        HeadTracking = 2,
    }
);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SoundModes {
    pub ambient_sound_mode: AmbientSoundMode,
    pub is_adaptive_selected: bool,
    pub anc_option_manual: u8,
    pub anc_option_auto: u8,
    pub trans_option_manual: u8,
    pub trans_option_auto: u8,
    pub m_trans_option: u8,
    pub anc_automation_mode: u8,
    pub anc_multi_scene_model: u8,
    pub trans_multi_scene_model: u8,
    pub wind_noise: WindNoise,
    pub large_noise_suppression: bool,
}

impl SoundModes {
    pub fn m_option(&self) -> u8 {
        match self.ambient_sound_mode {
            AmbientSoundMode::NoiseCanceling | AmbientSoundMode::Adaptive => 1,
            AmbientSoundMode::Transparency => 2,
            AmbientSoundMode::Normal => 0,
        }
    }

    pub fn bytes(&self) -> [u8; 7] {
        let manual = self.anc_option_manual.clamp(1, 5);
        let trans_manual = self.trans_option_manual.clamp(1, 5);
        let trans_auto = self.trans_option_auto.clamp(1, 5);
        let automation = if self.is_adaptive_selected {
            1
        } else {
            self.anc_automation_mode
        };
        [
            self.m_option(),
            automation,
            self.m_trans_option.clamp(0, 2),
            manual,
            (trans_auto << 4) | trans_manual,
            self.anc_multi_scene_model,
            self.trans_multi_scene_model,
        ]
    }
}

impl ToPacketBody for SoundModes {
    fn bytes(&self) -> Vec<u8> {
        self.bytes().to_vec()
    }
}

impl FromPacketBody for SoundModes {
    type DirectionMarker = packet::InboundMarker;

    fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "d1204 sound modes",
            map(
                (le_u8, le_u8, le_u8, le_u8, le_u8, le_u8),
                |(m_option, automation, m_trans_option, anc_nibbles, _, anc_scene)| Self {
                    ambient_sound_mode: match m_option {
                        1 => AmbientSoundMode::NoiseCanceling,
                        2 => AmbientSoundMode::Transparency,
                        _ => AmbientSoundMode::Normal,
                    },
                    anc_option_manual: (anc_nibbles & 0x0F).clamp(1, 5),
                    anc_option_auto: (anc_nibbles >> 4).clamp(1, 5),
                    trans_option_manual: 1,
                    trans_option_auto: 1,
                    m_trans_option,
                    anc_automation_mode: automation,
                    anc_multi_scene_model: anc_scene,
                    trans_multi_scene_model: 0,
                    wind_noise: WindNoise::default(),
                    large_noise_suppression: false,
                    is_adaptive_selected: false,
                },
            ),
        )
        .parse_complete(input)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SpatialAudio {
    pub is_enabled: bool,
    pub effect_mode: SpatialEffectMode,
    pub sound_mode: SpatialAudioMode,
}

impl SpatialAudio {
    pub fn take<'a, E: ParseError<&'a [u8]> + ContextError<&'a [u8]>>(
        input: &'a [u8],
    ) -> IResult<&'a [u8], Self, E> {
        context(
            "d1204 spatial audio",
            map(
                (take_bool, SpatialEffectMode::take, SpatialAudioMode::take),
                |(is_enabled, effect_mode, sound_mode)| Self {
                    is_enabled,
                    effect_mode,
                    sound_mode,
                },
            ),
        )
        .parse_complete(input)
    }

    pub fn bytes(&self) -> [u8; 3] {
        [
            u8::from(self.is_enabled),
            self.effect_mode as u8,
            self.sound_mode as u8,
        ]
    }
}

#[cfg(test)]
mod tests {
    use nom_language::error::VerboseError;

    use super::*;

    #[test]
    fn anc_payload_matches_a3956_layout() {
        let modes = SoundModes {
            ambient_sound_mode: AmbientSoundMode::NoiseCanceling,
            anc_option_manual: 4,
            anc_option_auto: 3,
            trans_option_manual: 2,
            trans_option_auto: 1,
            m_trans_option: 1,
            anc_automation_mode: 0,
            anc_multi_scene_model: 1,
            trans_multi_scene_model: 2,
            ..Default::default()
        };
        assert_eq!(modes.bytes(), [1, 0, 1, 4, 0x12, 1, 2]);
    }

    #[test]
    fn anc_payload_uses_first_nibble_for_manual_level() {
        let modes = SoundModes {
            ambient_sound_mode: AmbientSoundMode::Adaptive,
            is_adaptive_selected: true,
            anc_option_manual: 5,
            ..Default::default()
        };
        assert_eq!(modes.bytes()[0], 1);
        assert_eq!(modes.bytes()[1], 1);
        assert_eq!(modes.bytes()[3], 5);
    }

    #[test]
    fn anc_take_parses_manual_and_auto_nibbles() {
        let (_, modes) = SoundModes::take::<VerboseError<_>>(&[1, 0, 1, 0x42, 0, 2]).unwrap();
        assert_eq!(modes.ambient_sound_mode, AmbientSoundMode::NoiseCanceling);
        assert_eq!(modes.anc_option_manual, 2);
        assert_eq!(modes.anc_option_auto, 4);
        assert_eq!(modes.anc_multi_scene_model, 2);
    }

    #[test]
    fn spatial_payload_matches_a3956_layout() {
        let spatial = SpatialAudio {
            is_enabled: true,
            effect_mode: SpatialEffectMode::HeadTracking,
            sound_mode: SpatialAudioMode::Gaming,
        };
        assert_eq!(spatial.bytes(), [1, 2, 2]);
        let (_, parsed) = SpatialAudio::take::<VerboseError<_>>(&spatial.bytes()).unwrap();
        assert_eq!(parsed, spatial);
    }
}
