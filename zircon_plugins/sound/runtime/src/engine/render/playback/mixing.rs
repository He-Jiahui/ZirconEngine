use std::collections::{HashMap, HashSet};

use zircon_runtime::core::framework::sound::{SoundPlaybackFinishReason, SoundTrackId};

use crate::SoundConfig;

use super::super::super::state::SoundEngineState;
use super::super::routing::accepts_direct_input;
use super::clip::mix_clip_playback;
use super::finish::finish_playbacks;

impl SoundEngineState {
    pub(in crate::engine::render) fn mix_playbacks(
        &mut self,
        config: &SoundConfig,
        frames: usize,
        channels: usize,
        solo_tracks: &HashSet<SoundTrackId>,
        track_buffers: &mut HashMap<SoundTrackId, Vec<f32>>,
    ) {
        let clips = self.clips.clone();
        let mut finished = Vec::new();
        for (playback_id, playback) in self.playbacks.iter_mut() {
            let Some(clip) = clips.get(&playback.clip) else {
                finished.push((*playback_id, SoundPlaybackFinishReason::MissingClip));
                continue;
            };
            let output_track = if track_buffers.contains_key(&playback.output_track) {
                playback.output_track
            } else {
                SoundTrackId::master()
            };
            if !accepts_direct_input(output_track, solo_tracks) {
                let mut scratch = vec![0.0; frames.saturating_mul(channels)];
                if mix_clip_playback(
                    &mut scratch,
                    channels,
                    frames,
                    &clip.asset,
                    playback,
                    config,
                ) {
                    finished.push((*playback_id, SoundPlaybackFinishReason::Completed));
                }
                continue;
            }
            let finished_playback = if let Some(destination) = track_buffers.get_mut(&output_track)
            {
                mix_clip_playback(destination, channels, frames, &clip.asset, playback, config)
            } else {
                false
            };
            if finished_playback {
                finished.push((*playback_id, SoundPlaybackFinishReason::Completed));
            }
        }
        finish_playbacks(self, finished);
    }
}
