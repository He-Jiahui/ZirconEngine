mod environment;
mod finish;
mod routing;
mod snapshot;

use std::collections::{HashMap, HashSet};

use zircon_runtime::core::framework::sound::SoundTrackId;

use crate::SoundConfig;

use super::super::super::state::SoundEngineState;
use super::super::routing::accepts_direct_input;
use super::input::mix_source_voice;
use super::parameters::source_descriptor_with_parameter_bindings;
use environment::{apply_source_environment_block, source_has_pending_tail};
use finish::PendingSourceFinish;
use routing::route_source_block;
use snapshot::SourceRenderSnapshot;

impl SoundEngineState {
    pub(in crate::engine::render) fn mix_sources(
        &mut self,
        config: &SoundConfig,
        frames: usize,
        channels: usize,
        solo_tracks: &HashSet<SoundTrackId>,
        track_buffers: &mut HashMap<SoundTrackId, Vec<f32>>,
    ) {
        let snapshot = SourceRenderSnapshot::new(self);
        let mut finished = Vec::new();
        for (source_id, voice) in self.sources.iter_mut() {
            let source_descriptor =
                source_descriptor_with_parameter_bindings(&voice.descriptor, &snapshot.parameters);
            if !source_descriptor.playing {
                continue;
            }
            let output_track = snapshot.output_track_for(&source_descriptor, track_buffers);
            let mut source_buffer = vec![0.0; frames.saturating_mul(channels)];
            if voice.pending_finish.is_none() {
                voice.pending_finish = mix_source_voice(
                    &mut source_buffer,
                    channels,
                    frames,
                    voice,
                    &source_descriptor,
                    &snapshot.clips,
                    &snapshot.external_sources,
                    &snapshot.parameters,
                    config,
                );
            }
            if !accepts_direct_input(output_track, solo_tracks) {
                continue;
            }
            let dry_source_buffer = source_buffer.clone();
            apply_source_environment_block(
                &mut source_buffer,
                channels,
                config,
                *source_id,
                &source_descriptor,
                output_track,
                &snapshot,
                &mut self.hrtf_states,
            );
            if let Some(reason) = voice.pending_finish {
                let has_tail =
                    source_has_pending_tail(*source_id, output_track, &snapshot, &self.hrtf_states);
                if !has_tail {
                    finished.push(PendingSourceFinish::new(
                        *source_id,
                        source_descriptor.clone(),
                        reason,
                    ));
                }
            }
            route_source_block(
                track_buffers,
                output_track,
                &source_descriptor,
                &source_buffer,
                &dry_source_buffer,
            );
        }
        for finish in finished {
            if self.sources.remove(&finish.source_id).is_some() {
                self.finished_sources.push(finish.into_event());
            }
        }
    }
}
