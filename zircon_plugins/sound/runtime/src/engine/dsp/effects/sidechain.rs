use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{SoundSidechainInput, SoundTrackId};

pub(super) fn sidechain_buffer<'a>(
    sidechain: SoundSidechainInput,
    pre_effect_sidechain_buffers: &'a HashMap<SoundTrackId, Vec<f32>>,
    post_effect_sidechain_buffers: &'a HashMap<SoundTrackId, Vec<f32>>,
) -> Option<&'a [f32]> {
    if sidechain.pre_effects {
        pre_effect_sidechain_buffers
            .get(&sidechain.track)
            .map(Vec::as_slice)
    } else {
        post_effect_sidechain_buffers
            .get(&sidechain.track)
            .map(Vec::as_slice)
    }
}
