use std::sync::Arc;

use async_trait::async_trait;
use openscq30_lib_has::Has;
use tokio::sync::watch;

use crate::{
    api::device,
    devices::soundcore::{
        common::{packet::PacketIOController, state_modifier::StateModifier},
        d1204::{packets::outbound::set_spatial_audio, structures::SpatialAudio},
    },
};

pub struct SpatialAudioStateModifier {
    packet_io: Arc<PacketIOController>,
}

impl SpatialAudioStateModifier {
    pub fn new(packet_io: Arc<PacketIOController>) -> Self {
        Self { packet_io }
    }
}

#[async_trait]
impl<T> StateModifier<T> for SpatialAudioStateModifier
where
    T: Has<SpatialAudio> + Clone + Send + Sync,
{
    async fn move_to_state(
        &self,
        state_sender: &watch::Sender<T>,
        target_state: &T,
    ) -> device::Result<()> {
        let target: SpatialAudio = *target_state.get();
        {
            let state = state_sender.borrow();
            let current: &SpatialAudio = state.get();
            if *current == target {
                return Ok(());
            }
        }
        self.packet_io
            .send_with_response(&set_spatial_audio(&target))
            .await?;
        state_sender.send_modify(|state| {
            let value: &mut SpatialAudio = state.get_mut();
            *value = target;
        });
        Ok(())
    }
}
