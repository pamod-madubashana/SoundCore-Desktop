use std::{marker::PhantomData, sync::Arc};

use async_trait::async_trait;
use openscq30_lib_has::Has;
use tokio::sync::watch;

use crate::{
    api::device,
    devices::soundcore::{
        common::{
            packet::PacketIOController, state_modifier::StateModifier,
            structures::EqualizerConfiguration,
        },
        d1204::packets::outbound::set_equalizer_configuration,
    },
};

pub struct EqualizerStateModifier<T> {
    _state: PhantomData<T>,
    packet_io: Arc<PacketIOController>,
}

impl<T> EqualizerStateModifier<T> {
    pub fn new(packet_io: Arc<PacketIOController>) -> Self {
        Self {
            packet_io,
            _state: PhantomData,
        }
    }
}

#[async_trait]
impl<T> StateModifier<T> for EqualizerStateModifier<T>
where
    T: Has<EqualizerConfiguration<1, 8, -120, 134, 1>> + Clone + Send + Sync,
{
    async fn move_to_state(
        &self,
        state_sender: &watch::Sender<T>,
        target_state: &T,
    ) -> device::Result<()> {
        let target: &EqualizerConfiguration<1, 8, -120, 134, 1> = target_state.get();
        {
            let state = state_sender.borrow();
            let current: &EqualizerConfiguration<1, 8, -120, 134, 1> = state.get();
            if current == target {
                return Ok(());
            }
        }
        self.packet_io
            .send_with_response(&set_equalizer_configuration(target))
            .await?;
        state_sender.send_modify(|state| {
            let value: &mut EqualizerConfiguration<1, 8, -120, 134, 1> = state.get_mut();
            *value = *target;
        });
        Ok(())
    }
}
