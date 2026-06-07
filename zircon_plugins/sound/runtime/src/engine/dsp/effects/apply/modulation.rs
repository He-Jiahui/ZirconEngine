use zircon_runtime::core::framework::sound::{
    SoundChorusEffect, SoundFlangerEffect, SoundPhaserEffect,
};

use crate::engine::dsp_state::SoundEffectRuntimeState;

use super::super::super::modulation::{modulated_delay, phaser_block};

pub(super) fn apply_flanger_effect(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    flanger: &SoundFlangerEffect,
    state: &mut SoundEffectRuntimeState,
) {
    modulated_delay(
        buffer,
        channels,
        sample_rate_hz,
        flanger.delay_frames,
        flanger.depth_frames,
        flanger.rate_hz,
        flanger.feedback,
        &mut state.modulation_history,
        &mut state.modulated_delay_phase,
    );
}

pub(super) fn apply_phaser_effect(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    phaser: &SoundPhaserEffect,
    state: &mut SoundEffectRuntimeState,
) {
    phaser_block(
        buffer,
        channels,
        sample_rate_hz,
        phaser.rate_hz,
        phaser.depth,
        phaser.phase_offset,
        &mut state.phaser_phase,
    );
}

pub(super) fn apply_chorus_effect(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    chorus: &SoundChorusEffect,
    state: &mut SoundEffectRuntimeState,
) {
    modulated_delay(
        buffer,
        channels,
        sample_rate_hz,
        chorus.delay_frames,
        chorus.depth_frames.saturating_mul(chorus.voices as usize),
        chorus.rate_hz,
        0.0,
        &mut state.modulation_history,
        &mut state.modulated_delay_phase,
    );
}
