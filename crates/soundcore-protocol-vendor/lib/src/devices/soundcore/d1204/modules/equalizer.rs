mod state_modifier;

use std::sync::Arc;

use openscq30_lib_has::Has;
use tokio::sync::watch;

use crate::{
    DeviceModel,
    devices::soundcore::{
        common::{
            modules::{
                ModuleCollection,
                equalizer::{
                    EqualizerModuleSettings, EqualizerPreset, common_settings_with_presets,
                },
            },
            structures::{EqualizerConfiguration, TwsStatus, VolumeAdjustments},
        },
        d1204::packets::outbound::CUSTOM_EQ_ID,
    },
    i18n::fl,
    storage::OpenSCQ30Database,
};

pub fn eq_settings() -> EqualizerModuleSettings<8, 8, -120, 134, 1> {
    let presets = vec![
        EqualizerPreset {
            name: "SoundcoreSignature",
            localized_name: || fl!("soundcore-signature"),
            id: 0,
            volume_adjustments: VolumeAdjustments::new([0, 0, 0, 0, 0, 0, 0, 0]),
        },
        EqualizerPreset {
            name: "Acoustic",
            localized_name: || fl!("acoustic"),
            id: 1,
            volume_adjustments: VolumeAdjustments::new([10, 40, 0, 0, 0, 0, 0, 0]),
        },
        EqualizerPreset {
            name: "BassBooster",
            localized_name: || fl!("bass-booster"),
            id: 2,
            volume_adjustments: VolumeAdjustments::new([25, 20, -9, 24, 0, 0, 0, 0]),
        },
        EqualizerPreset {
            name: "Classical",
            localized_name: || fl!("classical"),
            id: 3,
            volume_adjustments: VolumeAdjustments::new([50, 60, -23, -25, 0, 0, 0, 0]),
        },
    ];
    let mut settings = common_settings_with_presets(presets);
    settings.custom_preset_id = Some(CUSTOM_EQ_ID);
    settings
}

impl<T> ModuleCollection<T>
where
    T: Has<EqualizerConfiguration<1, 8, -120, 134, 1>>
        + Has<TwsStatus>
        + Clone
        + Send
        + Sync
        + 'static,
{
    pub async fn add_d1204_equalizer(
        &mut self,
        packet_io: Arc<crate::devices::soundcore::common::packet::PacketIOController>,
        database: Arc<OpenSCQ30Database>,
        device_model: DeviceModel,
        change_notify: watch::Sender<()>,
        settings: EqualizerModuleSettings<8, 8, -120, 134, 1>,
    ) {
        self.add_equalizer_with_custom_state_modifier_tws(
            database,
            device_model,
            change_notify,
            Box::new(state_modifier::EqualizerStateModifier::new(packet_io)),
            settings,
        )
        .await;
    }
}
