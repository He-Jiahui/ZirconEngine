use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{SoundCompressorEffect, SoundTrackId};

use crate::engine::dsp_state::SoundEffectRuntimeState;

use super::super::super::dynamics::compressor_block;
use super::super::sidechain::sidechain_buffer;

pub(super) fn apply_compressor_effect(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    compressor: &SoundCompressorEffect,
    pre_effect_sidechain_buffers: &HashMap<SoundTrackId, Vec<f32>>,
    post_effect_sidechain_buffers: &HashMap<SoundTrackId, Vec<f32>>,
    state: &mut SoundEffectRuntimeState,
) {
    let sidechain = compressor.sidechain.and_then(|sidechain| {
        sidechain_buffer(
            sidechain,
            pre_effect_sidechain_buffers,
            post_effect_sidechain_buffers,
        )
    });
    compressor_block(
        buffer,
        channels,
        sample_rate_hz,
        compressor.threshold_db,
        compressor.ratio,
        compressor.attack_ms,
        compressor.release_ms,
        compressor.makeup_gain_db,
        sidechain,
        &mut state.compressor_gain,
    );
}
