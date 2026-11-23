use async_trait::async_trait;
use openscq30_lib_has::Has;
use strum::IntoEnumIterator;

use crate::{
    api::settings::{self, Setting, SettingId, Value},
    devices::soundcore::{
        common::settings_manager::{SettingHandler, SettingHandlerError, SettingHandlerResult},
        d1204::structures::SoundModes,
    },
    i18n::fl,
};

use super::SoundModeSetting;

#[derive(Default)]
pub struct SoundModesSettingHandler;

#[async_trait]
impl<T> SettingHandler<T> for SoundModesSettingHandler
where
    T: Has<SoundModes> + Send,
{
    fn settings(&self) -> Vec<SettingId> {
        SoundModeSetting::iter().map(Into::into).collect()
    }

    fn get(&self, state: &T, setting_id: &SettingId) -> Option<Setting> {
        let sound_modes: &SoundModes = state.get();
        let sound_mode_setting: SoundModeSetting = (*setting_id).try_into().ok()?;
        Some(match sound_mode_setting {
            SoundModeSetting::AmbientSoundMode => {
                Setting::select_from_enum_all_variants(sound_modes.ambient_sound_mode)
            }
            SoundModeSetting::ManualNoiseCanceling => Setting::I32Range {
                setting: settings::Range {
                    range: 1..=5,
                    step: 1,
                },
                value: sound_modes.anc_option_manual.clamp(1, 5).into(),
            },
            SoundModeSetting::ManualTransparency => Setting::I32Range {
                setting: settings::Range {
                    range: 1..=5,
                    step: 1,
                },
                value: sound_modes.trans_option_manual.clamp(1, 5).into(),
            },
            SoundModeSetting::WindNoiseSuppression => Setting::Toggle {
                value: sound_modes.wind_noise.is_suppression_enabled,
            },
            SoundModeSetting::AdaptiveNoiseCanceling => Setting::Information {
                value: sound_modes.is_adaptive_selected.to_string(),
                translated_value: if sound_modes.is_adaptive_selected {
                    fl!("yes")
                } else {
                    fl!("no")
                },
            },
        })
    }

    async fn set(
        &self,
        state: &mut T,
        setting_id: &SettingId,
        value: Value,
    ) -> SettingHandlerResult<()> {
        let sound_mode_setting: SoundModeSetting = (*setting_id)
            .try_into()
            .expect("already filtered to valid values only by SettingsManager");
        let sound_modes: &mut SoundModes = state.get_mut();
        match sound_mode_setting {
            SoundModeSetting::AmbientSoundMode => {
                sound_modes.ambient_sound_mode = value.try_as_enum_variant()?;
            }
            SoundModeSetting::ManualNoiseCanceling => {
                sound_modes.anc_option_manual = value.try_as_i32()?.clamp(1, 5) as u8;
            }
            SoundModeSetting::ManualTransparency => {
                sound_modes.trans_option_manual = value.try_as_i32()?.clamp(1, 5) as u8;
            }
            SoundModeSetting::WindNoiseSuppression => {
                sound_modes.wind_noise.is_suppression_enabled = value.try_as_bool()?;
            }
            SoundModeSetting::AdaptiveNoiseCanceling => return Err(SettingHandlerError::ReadOnly),
        }
        Ok(())
    }
}
