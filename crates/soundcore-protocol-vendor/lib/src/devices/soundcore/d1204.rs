use std::collections::HashMap;

use crate::devices::soundcore::{
    common::{
        macros::soundcore_device,
        packet::{
            inbound::TryToPacket,
            outbound::{RequestState, ToPacket},
        },
    },
    d1204::{packets::inbound::D1204StateUpdatePacket, state::D1204State},
};

mod modules;
mod packets;
mod state;
mod structures;

soundcore_device!(
    D1204State,
    async |packet_io| {
        let packet: D1204StateUpdatePacket = packet_io
            .send_with_response(&RequestState.to_packet())
            .await?
            .try_to_packet()?;
        Ok(D1204State::new(packet))
    },
    async |builder| {
        builder.module_collection().add_state_update();

        builder.d1204_battery();
        builder.d1204_sound_modes();
        builder.d1204_spatial_audio();
        builder.d1204_equalizer().await;
    },
    {
        HashMap::from([(
            RequestState::COMMAND,
            D1204StateUpdatePacket::default().to_packet(),
        )])
    },
);

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        DeviceModel,
        devices::soundcore::common::{
            device::{SoundcoreDeviceConfig, test_utils::TestSoundcoreDevice},
            packet,
        },
        settings::SettingId,
    };

    #[tokio::test(start_paused = true)]
    async fn d1204_exposes_parsed_state() {
        let device = TestSoundcoreDevice::new(
            super::device_registry,
            DeviceModel::SoundcoreD1204,
            HashMap::from([(
                packet::Command([1, 1]),
                D1204StateUpdatePacketFixture::state_update(),
            )]),
            SoundcoreDeviceConfig::default(),
        )
        .await;

        device.assert_setting_values([
            (SettingId::BatteryLevelLeft, "87/100".into()),
            (SettingId::BatteryLevelRight, "92/100".into()),
            (SettingId::CaseBatteryLevel, "64/100".into()),
            (SettingId::AmbientSoundMode, "NoiseCanceling".into()),
            (SettingId::ManualNoiseCanceling, 3.into()),
            (SettingId::ManualTransparency, 2.into()),
            (SettingId::SpatialAudio, true.into()),
        ]);
    }

    struct D1204StateUpdatePacketFixture;

    impl D1204StateUpdatePacketFixture {
        fn state_update() -> packet::Inbound {
            let mut body = Vec::new();
            push(&mut body, 2, &[1]);
            push(&mut body, 3, &[0, 87]);
            push(&mut body, 4, &[0, 92]);
            push(&mut body, 8, &[64]);
            push(&mut body, 11, &[0x1E, 0xFF]);
            let mut eq = Vec::new();
            for gain in [1.0f32, 4.0, -0.9, 2.4, 0.0, -1.5, 3.25, -2.0] {
                eq.extend_from_slice(&gain.to_le_bytes());
            }
            push(&mut body, 12, &eq);
            push(&mut body, 36, &[1, 0, 1]);
            push(&mut body, 37, &[0x03, 0x02]);
            push(&mut body, 44, &[1, 2, 2]);
            push(&mut body, 46, &[0x01]);
            push(&mut body, 250, &[1, 2, 3]);
            packet::Inbound::new(packet::Command([1, 1]), body)
        }
    }

    fn push(output: &mut Vec<u8>, tag: u8, value: &[u8]) {
        output.push(tag);
        output.push(value.len() as u8);
        output.extend_from_slice(value);
    }

    #[tokio::test(start_paused = true)]
    async fn d1204_sets_sound_mode_and_spatial_audio() {
        let mut device = TestSoundcoreDevice::new(
            super::device_registry,
            DeviceModel::SoundcoreD1204,
            HashMap::from([(
                packet::Command([1, 1]),
                D1204StateUpdatePacketFixture::state_update(),
            )]),
            SoundcoreDeviceConfig::default(),
        )
        .await;

        device
            .assert_set_settings_response(
                vec![
                    (SettingId::AmbientSoundMode, "Transparency".into()),
                    (SettingId::SpatialAudio, false.into()),
                ],
                vec![
                    packet::Outbound::new(packet::Command([6, 0x81]), vec![2, 0, 1, 3, 0x12, 0, 0]),
                    packet::Outbound::new(packet::Command([16, 0x81]), vec![0, 2, 2]),
                ],
            )
            .await;
    }
}
