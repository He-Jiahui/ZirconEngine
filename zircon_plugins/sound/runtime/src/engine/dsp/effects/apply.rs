use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundEffectKind, SoundImpulseResponseId, SoundTrackId,
};

use crate::engine::dsp_state::SoundEffectRuntimeState;
use crate::engine::filter::apply_biquad_filter_block;

mod compressor;
mod convolution;
mod modulation;

use super::super::delay::delay_block;
use super::super::dynamics::limit;
use super::super::gain::multiply;
use super::super::reverb::reverb_block;
use super::super::shaper::waveshape;
use super::super::stereo::pan_stereo;
use compressor::apply_compressor_effect;
use convolution::apply_convolution_reverb_effect;
use modulation::{apply_chorus_effect, apply_flanger_effect, apply_phaser_effect};

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
            apply_convolution_reverb_effect(buffer, channels, convolution, impulse_responses, state)
        }
        SoundEffectKind::Compressor(compressor) => apply_compressor_effect(
            buffer,
            channels,
            sample_rate_hz,
            compressor,
            pre_effect_sidechain_buffers,
            post_effect_sidechain_buffers,
            state,
        ),
        SoundEffectKind::WaveShaper(shaper) => waveshape(buffer, shaper.drive),
        SoundEffectKind::Flanger(flanger) => {
            apply_flanger_effect(buffer, channels, sample_rate_hz, flanger, state)
        }
        SoundEffectKind::Phaser(phaser) => {
            apply_phaser_effect(buffer, channels, sample_rate_hz, phaser, state)
        }
        SoundEffectKind::Chorus(chorus) => {
            apply_chorus_effect(buffer, channels, sample_rate_hz, chorus, state)
        }
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
