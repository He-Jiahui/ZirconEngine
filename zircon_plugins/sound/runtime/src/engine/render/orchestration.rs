use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{SoundError, SoundMixBlock, SoundTrackId};

use crate::SoundConfig;

use super::super::dsp::{apply_track_controls, apply_track_effects, meter_for};
use super::super::state::SoundEngineState;
use super::super::validation::{track_render_order, validate_graph};
use super::routing::{add_scaled, solo_tracks};
use super::runtime_state::{latency_frames_for_graph, sync_runtime_states};

impl SoundEngineState {
    pub(crate) fn render_mix(
        &mut self,
        config: &SoundConfig,
        frames: usize,
    ) -> Result<SoundMixBlock, SoundError> {
        if frames == 0 {
            return Err(SoundError::InvalidMixRequest { frames });
        }
        validate_graph(&self.graph)?;
        sync_runtime_states(self);
        self.latency_frames = latency_frames_for_graph(&self.graph);

        let channels = config.channel_count.max(1) as usize;
        let samples_len = frames.saturating_mul(channels);
        let mut track_buffers = self
            .graph
            .tracks
            .iter()
            .map(|track| (track.id, vec![0.0; samples_len]))
            .collect::<HashMap<_, _>>();
        let solo_tracks = solo_tracks(&self.graph);

        self.mix_playbacks(config, frames, channels, &solo_tracks, &mut track_buffers);
        self.mix_sources(config, frames, channels, &solo_tracks, &mut track_buffers);
        let mut pre_effect_sidechain_buffers = track_buffers.clone();
        let mut post_effect_sidechain_buffers = HashMap::new();

        let mut meters = Vec::new();
        let tracks = self.graph.tracks.clone();
        for track_id in track_render_order(&self.graph) {
            let Some(track) = tracks.iter().find(|track| track.id == track_id) else {
                continue;
            };
            let Some(mut buffer) = track_buffers.remove(&track_id) else {
                continue;
            };
            let raw_buffer = buffer.clone();
            pre_effect_sidechain_buffers.insert(track_id, raw_buffer.clone());
            if !track.controls.bypass_effects {
                apply_track_effects(
                    &mut buffer,
                    channels,
                    config.sample_rate_hz,
                    &track.effects,
                    &pre_effect_sidechain_buffers,
                    &post_effect_sidechain_buffers,
                    &self.impulse_responses,
                    track_id,
                    &mut self.effect_states,
                );
            }
            let track_state = self.track_states.entry(track_id).or_default();
            apply_track_controls(&mut buffer, channels, track.controls, track_state);
            post_effect_sidechain_buffers.insert(track_id, buffer.clone());
            meters.push(meter_for(track_id, &buffer, channels));

            if track_id == SoundTrackId::master() {
                track_buffers.insert(track_id, buffer);
                continue;
            }

            if let Some(parent) = track.parent {
                if let Some(parent_buffer) = track_buffers.get_mut(&parent) {
                    add_scaled(parent_buffer, &buffer, 1.0);
                }
            }
            for send in &track.sends {
                if let Some(send_buffer) = track_buffers.get_mut(&send.target) {
                    let source = if send.pre_effects {
                        &raw_buffer
                    } else {
                        &buffer
                    };
                    add_scaled(send_buffer, source, send.gain);
                }
            }
        }

        self.meters = meters;
        let mut mix = SoundMixBlock {
            sample_rate_hz: config.sample_rate_hz,
            channel_count: config.channel_count.max(1),
            samples: track_buffers
                .remove(&SoundTrackId::master())
                .unwrap_or_else(|| vec![0.0; samples_len]),
        };
        for sample in &mut mix.samples {
            *sample = (*sample * config.master_gain).clamp(-1.0, 1.0);
        }
        Ok(mix)
    }
}
