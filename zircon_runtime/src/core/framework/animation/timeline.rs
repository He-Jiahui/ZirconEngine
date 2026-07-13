use serde::{Deserialize, Serialize};

use crate::core::math::Real;

use super::{avatar_mask::animation_target_id_matches, AnimationAvatarMask, AnimationTrackPath};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationTimelineTrackKind {
    Property,
    BoneTransform,
    Event,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationTimelineTrackDescriptor {
    pub kind: AnimationTimelineTrackKind,
    pub path: Option<AnimationTrackPath>,
    pub target_id: Option<String>,
    pub key_count: u32,
    pub muted: bool,
    pub avatar_mask: Option<AnimationAvatarMask>,
}

impl AnimationTimelineTrackDescriptor {
    pub fn property(path: AnimationTrackPath, key_count: u32) -> Self {
        Self {
            kind: AnimationTimelineTrackKind::Property,
            path: Some(path),
            target_id: None,
            key_count,
            muted: false,
            avatar_mask: None,
        }
    }

    pub fn bone_transform(target_id: impl Into<String>, key_count: u32) -> Self {
        Self {
            kind: AnimationTimelineTrackKind::BoneTransform,
            path: None,
            target_id: Some(target_id.into()),
            key_count,
            muted: false,
            avatar_mask: None,
        }
    }

    pub fn event(target_id: Option<String>) -> Self {
        Self {
            kind: AnimationTimelineTrackKind::Event,
            path: None,
            target_id,
            key_count: 1,
            muted: false,
            avatar_mask: None,
        }
    }

    pub fn with_avatar_mask(mut self, avatar_mask: AnimationAvatarMask) -> Self {
        self.avatar_mask = Some(avatar_mask);
        self
    }

    pub fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }

    pub fn allows_target(&self, target_id: &str) -> bool {
        let target_id = target_id.trim();
        if target_id.is_empty() || self.muted {
            return false;
        }

        let direct_match = self
            .target_id
            .as_deref()
            .map(|candidate| animation_target_id_matches(candidate, target_id))
            .unwrap_or(true);
        let mask_match = self
            .avatar_mask
            .as_ref()
            .map(|mask| mask.allows_target(target_id))
            .unwrap_or(true);

        direct_match && mask_match
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationTimelineClipDescriptor {
    pub clip_id: Option<String>,
    pub start_seconds: Real,
    pub duration_seconds: Real,
    pub playback_speed: Real,
    pub looping: bool,
    pub weight: Real,
}

impl Default for AnimationTimelineClipDescriptor {
    fn default() -> Self {
        Self {
            clip_id: None,
            start_seconds: 0.0,
            duration_seconds: 0.0,
            playback_speed: 1.0,
            looping: false,
            weight: 1.0,
        }
    }
}

impl AnimationTimelineClipDescriptor {
    pub fn sanitized_start_seconds(&self) -> Real {
        sanitize_non_negative_real(self.start_seconds)
    }

    pub fn sanitized_duration_seconds(&self) -> Real {
        sanitize_non_negative_real(self.duration_seconds)
    }

    pub fn sanitized_playback_speed(&self) -> Real {
        if self.playback_speed.is_finite() {
            self.playback_speed.max(0.0)
        } else {
            0.0
        }
    }

    pub fn normalized_weight(&self) -> Real {
        if self.weight.is_finite() {
            self.weight.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationTimelineEventDescriptor {
    pub target_id: Option<String>,
    pub name: String,
    pub payload: Option<String>,
    pub time_seconds: Real,
}

impl AnimationTimelineEventDescriptor {
    pub fn sanitized_time_seconds(&self) -> Real {
        sanitize_non_negative_real(self.time_seconds)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationTimelineDescriptor {
    pub id: Option<String>,
    pub duration_seconds: Real,
    pub frames_per_second: Real,
    pub clips: Vec<AnimationTimelineClipDescriptor>,
    pub tracks: Vec<AnimationTimelineTrackDescriptor>,
    pub events: Vec<AnimationTimelineEventDescriptor>,
}

impl Default for AnimationTimelineDescriptor {
    fn default() -> Self {
        Self {
            id: None,
            duration_seconds: 0.0,
            frames_per_second: 0.0,
            clips: Vec::new(),
            tracks: Vec::new(),
            events: Vec::new(),
        }
    }
}

impl AnimationTimelineDescriptor {
    pub fn sanitized_duration_seconds(&self) -> Real {
        sanitize_non_negative_real(self.duration_seconds)
    }

    pub fn sanitized_frames_per_second(&self) -> Real {
        sanitize_non_negative_real(self.frames_per_second)
    }

    pub fn track_count_by_kind(&self, kind: AnimationTimelineTrackKind) -> usize {
        self.tracks
            .iter()
            .filter(|track| track.kind == kind)
            .count()
    }
}

fn sanitize_non_negative_real(value: Real) -> Real {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
