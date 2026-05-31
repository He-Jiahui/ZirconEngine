use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundEffectKind, SoundImpulseResponseId, SoundTrackId,
};

use crate::engine::dsp_state::SoundEffectRuntimeState;
use crate::engine::filter::apply_biquad_filter_block;

use super::super::delay::delay_block;
use super::super::dynamics::{compressor_block, limit};
use super::super::gain::multiply;
use super::super::modulation::{modulated_delay, phaser_block};
use super::super::reverb::{convolve_block, reverb_block};
use super::super::shaper::waveshape;
use super::super::stereo::pan_stereo;
use super::sidechain::sidechain_buffer;

pub(super) fn apply_effect_kind(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    kind: &SoundEffectKind,
    pre_effect_sidechain_buffers: &HashMap<SoundTrackId, Vec<f32>>,
    post_effect_sidechain_buffers: &HashMap<SoundTrackId, Vec<f32>>,
    impulse_responses: &HashMap<SoundImpulseResponseId, Vec<f32>>,
    state: &mut SoundEffectRuntimeState,
) {
    match kind {
        SoundEffectKind::Gain(gain) => multiply(buffer, gain.gain),
        SoundEffectKind::Filter(filter) => apply_biquad_filter_block(
            buffer,
            channels,
            sample_rate_hz,
            *filter,
            &mut state.filter_state,
        ),
        SoundEffectKind::Reverb(reverb) => reverb_block(
            buffer,
            channels,
            reverb.pre_delay_frames,
            reverb.tail_frames,
            reverb.damping,
            &mut state.reverb_history,
        ),
        SoundEffectKind::ConvolutionReverb(convolution) => {
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
        SoundEffectKind::Compressor(compressor) => {
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
        SoundEffectKind::WaveShaper(shaper) => waveshape(buffer, shaper.drive),
        SoundEffectKind::Flanger(flanger) => modulated_delay(
            buffer,
            channels,
            sample_rate_hz,
            flanger.delay_frames,
            flanger.depth_frames,
            flanger.rate_hz,
            flanger.feedback,
            &mut state.modulation_history,
            &mut state.modulated_delay_phase,
        ),
        SoundEffectKind::Phaser(phaser) => phaser_block(
            buffer,
            channels,
            sample_rate_hz,
            phaser.rate_hz,
            phaser.depth,
            phaser.phase_offset,
            &mut state.phaser_phase,
        ),
        SoundEffectKind::Chorus(chorus) => modulated_delay(
            buffer,
            channels,
            sample_rate_hz,
            chorus.delay_frames,
            chorus.depth_frames.saturating_mul(chorus.voices as usize),
            chorus.rate_hz,
            0.0,
            &mut state.modulation_history,
            &mut state.modulated_delay_phase,
        ),
        SoundEffectKind::Delay(delay) => delay_block(
            buffer,
            channels,
            delay.delay_frames,
            delay.feedback,
            &mut state.delay_line,
        ),
        SoundEffectKind::PanStereo(pan) => pan_stereo(
            buffer,
            channels,
            pan.pan,
            pan.width,
            pan.left_gain,
            pan.right_gain,
            pan.invert_left_phase,
            pan.invert_right_phase,
        ),
        SoundEffectKind::Limiter(limiter) => limit(buffer, limiter.ceiling),
    }
}
