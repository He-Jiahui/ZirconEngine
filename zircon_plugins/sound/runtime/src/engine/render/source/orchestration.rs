use std::collections::{HashMap, HashSet};

use zircon_runtime::core::framework::sound::{SoundSourceFinished, SoundSourceInput, SoundTrackId};

use crate::SoundConfig;

use super::super::super::source_environment::{
    active_listener_for, apply_source_environment, hrtf_tail_pending_for_source,
};
use super::super::super::state::SoundEngineState;
use super::super::routing::{accepts_direct_input, add_scaled};
use super::input::mix_source_voice;
use super::parameters::source_descriptor_with_parameter_bindings;

impl SoundEngineState {
    pub(in crate::engine::render) fn mix_sources(
        &mut self,
        config: &SoundConfig,
        frames: usize,
        channels: usize,
        solo_tracks: &HashSet<SoundTrackId>,
        track_buffers: &mut HashMap<SoundTrackId, Vec<f32>>,
    ) {
        let clips = self.clips.clone();
        let external_sources = self.external_sources.clone();
        let parameters = self.parameters.clone();
        let listeners = self.listeners.values().cloned().collect::<Vec<_>>();
        let volumes = self.volumes.values().cloned().collect::<Vec<_>>();
        let impulse_responses = self.impulse_responses.clone();
        let ray_traced_impulse_responses = self.ray_traced_impulse_responses.clone();
        let hrtf_profiles = self.hrtf_profiles.clone();
        let ray_tracing = self.ray_tracing.clone();
        let mut finished = Vec::new();
        for (source_id, voice) in self.sources.iter_mut() {
            let source_descriptor =
                source_descriptor_with_parameter_bindings(&voice.descriptor, &parameters);
            if !source_descriptor.playing {
                continue;
            }
            let output_track = if track_buffers.contains_key(&source_descriptor.output_track) {
                source_descriptor.output_track
            } else {
                SoundTrackId::master()
            };
            let mut source_buffer = vec![0.0; frames.saturating_mul(channels)];
            if voice.pending_finish.is_none() {
                voice.pending_finish = mix_source_voice(
                    &mut source_buffer,
                    channels,
                    frames,
                    voice,
                    &source_descriptor,
                    &clips,
                    &external_sources,
                    &parameters,
                    config,
                );
            }
            if !accepts_direct_input(output_track, solo_tracks) {
                continue;
            }
            let dry_source_buffer = source_buffer.clone();
            apply_source_environment(
                &mut source_buffer,
                channels,
                config.sample_rate_hz,
                *source_id,
                &source_descriptor,
                active_listener_for(&listeners, output_track),
                config.default_spatial_scale,
                &volumes,
                &impulse_responses,
                &ray_traced_impulse_responses,
                &hrtf_profiles,
                &mut self.hrtf_states,
                &ray_tracing,
            );
            if let Some(reason) = voice.pending_finish {
                let has_tail = hrtf_tail_pending_for_source(
                    &self.hrtf_states,
                    *source_id,
                    active_listener_for(&listeners, output_track),
                    &hrtf_profiles,
                );
                if !has_tail {
                    finished.push((*source_id, source_descriptor.clone(), reason));
                }
            }
            if let Some(destination) = track_buffers.get_mut(&output_track) {
                add_scaled(destination, &source_buffer, 1.0);
            }
            for send in &source_descriptor.sends {
                if let Some(destination) = track_buffers.get_mut(&send.target) {
                    let source = if send.pre_spatial {
                        &dry_source_buffer
                    } else {
                        &source_buffer
                    };
                    add_scaled(destination, source, send.gain);
                }
            }
        }
        for (source_id, descriptor, reason) in finished {
            if self.sources.remove(&source_id).is_some() {
                let input = descriptor.input;
                let clip = match input {
                    SoundSourceInput::Clip(clip) => Some(clip),
                    SoundSourceInput::External(_)
                    | SoundSourceInput::SynthParameter { .. }
                    | SoundSourceInput::Silence => None,
                };
                self.finished_sources.push(SoundSourceFinished {
                    source: source_id,
                    input,
                    clip,
                    reason,
                    completion_action: descriptor.completion_action,
                    output_track: descriptor.output_track,
                });
            }
        }
    }
}
