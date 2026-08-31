use serde::{Deserialize, Serialize};

use crate::core::math::Real;

use super::{avatar_mask::PreparedAnimationTargetId, AnimationAvatarMask, AnimationTrackPath};

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
        let target = PreparedAnimationTargetId::new(target_id);

        if let Some(candidate) = self.target_id.as_deref() {
            if !target.matches(candidate) {
                return false;
            }
        }
        self.avatar_mask
            .as_ref()
            .is_none_or(|mask| mask.allows_prepared_target(target))
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

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;
    use crate::core::framework::animation::avatar_mask::animation_target_id_matches;

    const SAMPLE_COUNT: usize = 17;
    const CHECKS_PER_SAMPLE: usize = 1_200;

    #[test]
    fn optimization_batch_20260830cc_timeline_rejects_direct_target_before_scanning_avatar_mask() {
        let track = AnimationTimelineTrackDescriptor::bone_transform("Rig/Spine/Chest", 1)
            .with_avatar_mask(large_mask(32));

        assert!(!track.allows_target("Rig/Hands/Left"));

        let source = include_str!("timeline.rs");
        let allows_target = source
            .split("pub fn allows_target")
            .nth(1)
            .and_then(|source| source.split("pub fn muted").next())
            .expect("read timeline target filtering");
        let direct_rejection = allows_target
            .find("if !target.matches")
            .expect("a direct target mismatch must return immediately");
        let mask_scan = allows_target
            .find("allows_prepared_target")
            .expect("matching tracks must retain avatar-mask filtering");

        assert!(
            direct_rejection < mask_scan,
            "direct target rejection must happen before avatar-mask evaluation"
        );
    }

    #[test]
    #[ignore = "Windows Release performance qualification"]
    fn optimization_batch_20260830cc_animation_target_rejection_p95() {
        let mask = large_mask(256);
        let track = AnimationTimelineTrackDescriptor::bone_transform(
            "Character/Hero/Rig/Spine/Direct_Other",
            1,
        )
        .with_avatar_mask(mask.clone());
        let target = "Character/Hero/Rig/Hands/Missing_Target";
        let mut avatar_baseline = Vec::with_capacity(SAMPLE_COUNT);
        let mut avatar_optimized = Vec::with_capacity(SAMPLE_COUNT);
        let mut timeline_baseline = Vec::with_capacity(SAMPLE_COUNT);
        let mut timeline_optimized = Vec::with_capacity(SAMPLE_COUNT);
        let mut sink = 0usize;

        for sample_index in 0..SAMPLE_COUNT {
            if sample_index % 2 == 0 {
                sink += sample(&mut avatar_baseline, || legacy_mask_allows(&mask, target));
                sink += sample(&mut avatar_optimized, || mask.allows_target(target));
                sink += sample(&mut timeline_baseline, || {
                    legacy_timeline_allows(&track, target)
                });
                sink += sample(&mut timeline_optimized, || track.allows_target(target));
            } else {
                sink += sample(&mut avatar_optimized, || mask.allows_target(target));
                sink += sample(&mut avatar_baseline, || legacy_mask_allows(&mask, target));
                sink += sample(&mut timeline_optimized, || track.allows_target(target));
                sink += sample(&mut timeline_baseline, || {
                    legacy_timeline_allows(&track, target)
                });
            }
        }

        let avatar_baseline_p50 = percentile(&avatar_baseline, 50);
        let avatar_baseline_p95 = percentile(&avatar_baseline, 95);
        let avatar_optimized_p50 = percentile(&avatar_optimized, 50);
        let avatar_optimized_p95 = percentile(&avatar_optimized, 95);
        let timeline_baseline_p50 = percentile(&timeline_baseline, 50);
        let timeline_baseline_p95 = percentile(&timeline_baseline, 95);
        let timeline_optimized_p50 = percentile(&timeline_optimized, 50);
        let timeline_optimized_p95 = percentile(&timeline_optimized, 95);

        println!(
            "RUNTIME170_AVATAR_MASK_REJECTION_BENCH_V1 baseline_p50_ns={avatar_baseline_p50} optimized_p50_ns={avatar_optimized_p50} baseline_p95_ns={avatar_baseline_p95} optimized_p95_ns={avatar_optimized_p95}"
        );
        println!(
            "RUNTIME170_TIMELINE_DIRECT_REJECTION_BENCH_V1 baseline_p50_ns={timeline_baseline_p50} optimized_p50_ns={timeline_optimized_p50} baseline_p95_ns={timeline_baseline_p95} optimized_p95_ns={timeline_optimized_p95}"
        );

        assert!(
            avatar_optimized_p95 * 100 <= avatar_baseline_p95 * 70,
            "avatar-mask optimized P95 must be at most 70% of baseline"
        );
        assert!(
            timeline_optimized_p95 * 100 <= timeline_baseline_p95 * 10,
            "timeline optimized P95 must be at most 10% of baseline"
        );
        black_box(sink);
    }

    fn large_mask(entry_count: usize) -> AnimationAvatarMask {
        AnimationAvatarMask {
            id: "benchmark".to_string(),
            included_target_ids: (0..entry_count)
                .map(|index| format!("Rig/Spine/Bone_{index:03}"))
                .collect(),
            excluded_target_ids: (0..entry_count)
                .map(|index| format!("Rig/Face/Bone_{index:03}"))
                .collect(),
            weight: 1.0,
        }
    }

    fn legacy_mask_allows(mask: &AnimationAvatarMask, target_id: &str) -> bool {
        let target_id = target_id.trim();
        if target_id.is_empty() {
            return false;
        }
        let included = mask.included_target_ids.is_empty()
            || mask
                .included_target_ids
                .iter()
                .any(|candidate| animation_target_id_matches(candidate, target_id));
        let excluded = mask
            .excluded_target_ids
            .iter()
            .any(|candidate| animation_target_id_matches(candidate, target_id));
        included && !excluded
    }

    fn legacy_timeline_allows(track: &AnimationTimelineTrackDescriptor, target_id: &str) -> bool {
        let target_id = target_id.trim();
        if target_id.is_empty() || track.muted {
            return false;
        }
        let direct_match = track
            .target_id
            .as_deref()
            .map(|candidate| animation_target_id_matches(candidate, target_id))
            .unwrap_or(true);
        let mask_match = track
            .avatar_mask
            .as_ref()
            .map(|mask| legacy_mask_allows(mask, target_id))
            .unwrap_or(true);
        direct_match && mask_match
    }

    fn sample(samples: &mut Vec<u128>, mut operation: impl FnMut() -> bool) -> usize {
        let started = Instant::now();
        let mut hits = 0usize;
        for _ in 0..CHECKS_PER_SAMPLE {
            hits += black_box(operation()) as usize;
        }
        samples.push(started.elapsed().as_nanos());
        hits
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() - 1) * percentile / 100]
    }
}
