use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundConvolutionReverbEffect, SoundImpulseResponseId,
};

use crate::engine::dsp_state::SoundEffectRuntimeState;

use super::super::super::reverb::{convolve_block, reverb_block};

pub(super) fn apply_convolution_reverb_effect(
    buffer: &mut [f32],
    channels: usize,
    convolution: &SoundConvolutionReverbEffect,
    impulse_responses: &HashMap<SoundImpulseResponseId, Vec<f32>>,
    state: &mut SoundEffectRuntimeState,
) {
    if let Some(ir) = impulse_responses.get(&convolution.impulse_response) {
        convolve_block(buffer, channels, ir, &mut state.convolution_history);
    } else if convolution.fallback_to_algorithmic {
        reverb_block(
            buffer,
            channels,
            convolution.latency_frames,
            convolution.latency_frames.max(8),
            0.35,
            &mut state.reverb_history,
        );
    }
}
