use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundImpulseResponseId, SoundTrackId,
};

use crate::engine::dsp_state::{SoundEffectRuntimeState, SoundEffectStateKey};

use super::super::gain::wet_mix;
use super::apply::apply_effect_kind;

pub(crate) fn apply_track_effects(
    buffer: &mut [f32],
    channels: usize,
    sample_rate_hz: u32,
    effects: &[SoundEffectDescriptor],
    pre_effect_sidechain_buffers: &HashMap<SoundTrackId, Vec<f32>>,
    post_effect_sidechain_buffers: &HashMap<SoundTrackId, Vec<f32>>,
    impulse_responses: &HashMap<SoundImpulseResponseId, Vec<f32>>,
    track: SoundTrackId,
    effect_states: &mut HashMap<SoundEffectStateKey, SoundEffectRuntimeState>,
) {
    for effect in effects {
        if !effect.enabled || effect.bypass {
            continue;
        }
        let dry = buffer.to_vec();
        let state = effect_states
            .entry(SoundEffectStateKey::new(track, effect.id))
            .or_default();
        apply_effect_kind(
            buffer,
            channels,
            sample_rate_hz,
            &effect.kind,
            pre_effect_sidechain_buffers,
            post_effect_sidechain_buffers,
            impulse_responses,
            state,
        );
        wet_mix(buffer, &dry, effect.wet);
    }
}
