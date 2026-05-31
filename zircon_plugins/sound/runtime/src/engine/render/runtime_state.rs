use std::collections::HashSet;

use zircon_runtime::core::framework::sound::{
    SoundEffectDescriptor, SoundEffectKind, SoundMixerGraph, SoundTrackId,
};

use super::super::hrtf::prune_hrtf_render_states;
use super::super::source_environment::active_listener_for;
use super::super::state::SoundEngineState;
use super::super::{SoundEffectStateKey, SoundHrtfRenderStateKey};
use super::source::source_descriptor_with_parameter_bindings;

pub(super) fn sync_runtime_states(state: &mut SoundEngineState) {
    let track_ids = state
        .graph
        .tracks
        .iter()
        .map(|track| track.id)
        .collect::<HashSet<_>>();
    let effect_keys = state
        .graph
        .tracks
        .iter()
        .flat_map(|track| {
            track
                .effects
                .iter()
                .map(move |effect| SoundEffectStateKey::new(track.id, effect.id))
        })
        .collect::<HashSet<_>>();

    state
        .track_states
        .retain(|track, _| track_ids.contains(track));
    for track in &track_ids {
        state.track_states.entry(*track).or_default();
    }
    state
        .effect_states
        .retain(|effect, _| effect_keys.contains(effect));

    let listeners = state.listeners.values().cloned().collect::<Vec<_>>();
    let parameters = state.parameters.clone();
    let active_hrtf_keys = state
        .sources
        .iter()
        .filter_map(|(source_id, voice)| {
            let descriptor =
                source_descriptor_with_parameter_bindings(&voice.descriptor, &parameters);
            if !descriptor.playing && voice.pending_finish.is_none() {
                return None;
            }
            let output_track = if track_ids.contains(&descriptor.output_track) {
                descriptor.output_track
            } else {
                SoundTrackId::master()
            };
            let listener = active_listener_for(&listeners, output_track)?;
            let profile_id = listener.hrtf_profile.as_deref()?;
            state.hrtf_profiles.contains_key(profile_id).then(|| {
                SoundHrtfRenderStateKey::new(*source_id, listener.id, profile_id.to_string())
            })
        })
        .collect::<HashSet<_>>();
    prune_hrtf_render_states(&mut state.hrtf_states, &active_hrtf_keys);
}

pub(super) fn latency_frames_for_graph(graph: &SoundMixerGraph) -> usize {
    graph
        .tracks
        .iter()
        .map(|track| {
            let effect_latency = track
                .effects
                .iter()
                .filter(|effect| effect.enabled && !effect.bypass)
                .map(effect_latency_frames)
                .max()
                .unwrap_or_default();
            track.controls.delay_frames.max(effect_latency)
        })
        .max()
        .unwrap_or_default()
}

fn effect_latency_frames(effect: &SoundEffectDescriptor) -> usize {
    match &effect.kind {
        SoundEffectKind::Delay(delay) => delay.delay_frames,
        SoundEffectKind::Reverb(reverb) => reverb.pre_delay_frames.max(reverb.tail_frames),
        SoundEffectKind::ConvolutionReverb(convolution) => convolution.latency_frames,
        SoundEffectKind::Flanger(flanger) => flanger.delay_frames + flanger.depth_frames,
        SoundEffectKind::Chorus(chorus) => {
            chorus.delay_frames + chorus.depth_frames.saturating_mul(chorus.voices as usize)
        }
        _ => 0,
    }
}
