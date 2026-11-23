use openscq30_lib_macros::Has;

use crate::devices::soundcore::{
    common::{
        state::Update,
        structures::{CaseBatteryLevel, DualBattery, EqualizerConfiguration, TwsStatus},
    },
    d1204::{self, packets::inbound::D1204StateUpdatePacket},
};

#[derive(Debug, Clone, PartialEq, Eq, Has)]
pub struct D1204State {
    tws_status: TwsStatus,
    battery: DualBattery,
    case_battery_level: CaseBatteryLevel,
    equalizer_configuration: EqualizerConfiguration<1, 8, -120, 134, 1>,
    sound_modes: d1204::structures::SoundModes,
    spatial_audio: d1204::structures::SpatialAudio,
}

impl D1204State {
    pub fn new(packet: D1204StateUpdatePacket) -> Self {
        Self {
            tws_status: packet.tws_status,
            battery: packet.battery,
            case_battery_level: packet.case_battery_level,
            equalizer_configuration: packet.equalizer_configuration,
            sound_modes: packet.sound_modes,
            spatial_audio: packet.spatial_audio,
        }
    }
}

impl Update<D1204StateUpdatePacket> for D1204State {
    fn update(&mut self, packet: D1204StateUpdatePacket) {
        *self = Self::new(packet);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_replaces_state_from_packet() {
        let mut state = D1204State::new(D1204StateUpdatePacket::default());
        let packet = D1204StateUpdatePacket {
            tws_status: TwsStatus {
                is_connected: false,
                ..Default::default()
            },
            ..Default::default()
        };
        state.update(packet);
        assert!(!state.tws_status.is_connected);
    }
}
