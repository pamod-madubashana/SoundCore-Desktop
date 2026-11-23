use crate::devices::soundcore::{
    common::{packet, structures::EqualizerConfiguration},
    d1204::structures::SpatialAudio,
};

pub const CUSTOM_EQ_ID: u16 = 0xFF1E;

const EQ_COMMAND: packet::Command = packet::Command([3, 0x8A]);
const SPATIAL_COMMAND: packet::Command = packet::Command([16, 0x81]);

pub fn set_equalizer_configuration(
    configuration: &EqualizerConfiguration<1, 8, -120, 134, 1>,
) -> packet::Outbound {
    const FREQUENCIES: [f32; 8] = [100.0, 200.0, 400.0, 800.0, 1600.0, 3200.0, 6400.0, 12800.0];
    const Q: f32 = 0.7;
    const POINT_COUNT: usize = 8;

    let gains = configuration
        .volume_adjustments_channel_1()
        .map(|adjustments| adjustments.adjustments())
        .copied()
        .unwrap_or_default();

    let mut body = Vec::with_capacity(9 + POINT_COUNT * 13);
    body.extend_from_slice(&configuration.preset_id().to_le_bytes());
    body.push(0);
    body.push(0);
    body.extend_from_slice(&0u32.to_be_bytes());
    body.push(POINT_COUNT as u8);

    for (index, gain) in gains.into_iter().enumerate() {
        body.push(1);
        body.extend_from_slice(&((gain as f32) / 10.0).to_le_bytes());
        body.extend_from_slice(&FREQUENCIES[index].to_le_bytes());
        body.extend_from_slice(&Q.to_le_bytes());
    }

    packet::Outbound::new(EQ_COMMAND, body)
}

pub fn set_spatial_audio(spatial_audio: &SpatialAudio) -> packet::Outbound {
    packet::Outbound::new(SPATIAL_COMMAND, spatial_audio.bytes().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_write_matches_a3956_payload_layout() {
        let configuration = EqualizerConfiguration::<1, 8, -120, 134, 1>::new_all_bands_present(
            CUSTOM_EQ_ID,
            [
                crate::devices::soundcore::common::structures::VolumeAdjustments::new([
                    10, 40, -9, 24, 0, -15, 32, -20,
                ]),
            ],
        );
        let packet = set_equalizer_configuration(&configuration);
        assert_eq!(packet.command, EQ_COMMAND);
        assert_eq!(packet.body.len(), 9 + 8 * 13);
        assert_eq!(&packet.body[..2], &CUSTOM_EQ_ID.to_le_bytes());
        assert_eq!(packet.body[8], 8);
        assert_eq!(packet.body[9], 1);
        assert_eq!(&packet.body[10..14], &1.0f32.to_le_bytes());
        assert_eq!(&packet.body[14..18], &100.0f32.to_le_bytes());
        assert_eq!(&packet.body[18..22], &0.7f32.to_le_bytes());
    }

    #[test]
    fn spatial_write_uses_three_byte_payload() {
        let spatial = SpatialAudio::default();
        let packet = set_spatial_audio(&spatial);
        assert_eq!(packet.command, SPATIAL_COMMAND);
        assert_eq!(packet.body, spatial.bytes());
    }
}
