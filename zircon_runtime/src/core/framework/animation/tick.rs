use serde::{Deserialize, Serialize};

use crate::core::framework::scene::WorldHandle;
use crate::core::math::Real;

use super::{AnimationEventRecord, AnimationPlaybackSettings};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationTickRequest {
    pub world: WorldHandle,
    pub delta_seconds: Real,
    pub frame_index: u64,
    pub playback_settings: AnimationPlaybackSettings,
}

impl AnimationTickRequest {
    pub fn new(world: WorldHandle, delta_seconds: Real) -> Self {
        Self {
            world,
            delta_seconds,
            frame_index: 0,
            playback_settings: AnimationPlaybackSettings::default(),
        }
    }

    pub fn with_frame_index(mut self, frame_index: u64) -> Self {
        self.frame_index = frame_index;
        self
    }

    pub fn with_playback_settings(mut self, playback_settings: AnimationPlaybackSettings) -> Self {
        self.playback_settings = playback_settings;
        self
    }

    pub fn sanitized_delta_seconds(&self) -> Real {
        if self.delta_seconds.is_finite() {
            self.delta_seconds.max(0.0)
        } else {
            0.0
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AnimationTickReport {
    pub world: WorldHandle,
    pub advanced_players: u32,
    pub sampled_clips: u32,
    pub evaluated_graphs: u32,
    pub evaluated_state_machines: u32,
    pub applied_sequences: u32,
    pub posed_entities: u32,
    pub missing_tracks: u32,
    pub emitted_events: Vec<AnimationEventRecord>,
    pub diagnostics: Vec<String>,
}

impl AnimationTickReport {
    pub fn new(world: WorldHandle) -> Self {
        Self {
            world,
            ..Self::default()
        }
    }

    pub fn has_runtime_work(&self) -> bool {
        self.advanced_players > 0
            || self.sampled_clips > 0
            || self.evaluated_graphs > 0
            || self.evaluated_state_machines > 0
            || self.applied_sequences > 0
            || self.posed_entities > 0
            || !self.emitted_events.is_empty()
    }

    pub fn with_event(mut self, event: AnimationEventRecord) -> Self {
        self.emitted_events.push(event);
        self
    }

    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostics.push(diagnostic.into());
        self
    }
}
