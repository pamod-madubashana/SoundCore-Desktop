mod equalizer;
mod sound_modes;
mod spatial_audio;

use crate::devices::soundcore::{
    common::modules::{
        case_battery_level::CaseBatteryLevelConfiguration, dual_battery::DualBatteryConfiguration,
    },
    d1204::state::D1204State,
};

use crate::devices::soundcore::common::device::SoundcoreDeviceBuilder;

impl SoundcoreDeviceBuilder<D1204State> {
    pub fn d1204_battery(&mut self) {
        self.dual_battery_custom(DualBatteryConfiguration {
            max_level: 100,
            level_offset: 0,
        });
        self.case_battery_level_custom(CaseBatteryLevelConfiguration {
            max_level: 100,
            level_offset: 0,
        });
        self.tws_status();
    }

    pub fn d1204_sound_modes(&mut self) {
        let packet_io = self.packet_io_controller().clone();
        self.module_collection().add_d1204_sound_modes(packet_io);
    }

    pub fn d1204_spatial_audio(&mut self) {
        let packet_io = self.packet_io_controller().clone();
        self.module_collection().add_d1204_spatial_audio(packet_io);
    }

    pub async fn d1204_equalizer(&mut self) {
        let packet_io = self.packet_io_controller().clone();
        let database = self.database();
        let device_model = self.device_model();
        let change_notify = self.change_notify();
        self.module_collection()
            .add_d1204_equalizer(
                packet_io,
                database,
                device_model,
                change_notify,
                equalizer::eq_settings(),
            )
            .await;
    }
}
