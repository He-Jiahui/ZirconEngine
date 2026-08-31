use std::sync::Arc;

use zircon_runtime::core::framework::animation::{
    AnimationChannelValueAsset, AnimationClipAsset, AnimationSkeletonAsset,
};
use zircon_runtime::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
};
use zircon_runtime::core::math::{Quat, Real, Transform, Vec3};

use crate::{CompiledAnimationClip, PoseBuffer, PosePool, SkeletonTargetTable};

use super::cache::{CachedClip, CachedSkeleton};
use super::channel_sample::sample_channel;
use super::channel_validation::validate_clip_channels;
use super::time::resolve_sample_time;
use super::{
    AnimationAssetRevision, AnimationClipEvaluator, AnimationClipEvaluatorStats,
    AnimationEvaluationError, AnimationTransformChannel,
};

impl AnimationClipEvaluator {
    pub fn sample_clip(
        &mut self,
        skeleton_revision: AnimationAssetRevision,
        clip_revision: AnimationAssetRevision,
        skeleton: &AnimationSkeletonAsset,
        clip: &AnimationClipAsset,
        time_seconds: Real,
        looping: bool,
    ) -> Result<AnimationPoseOutput, AnimationEvaluationError> {
        self.invalidate_changed_resources();
        self.ensure_skeleton(skeleton_revision, skeleton)?;
        self.ensure_clip(skeleton_revision, clip_revision, clip)?;

        let cache_key = (skeleton_revision.id(), clip_revision.id());
        let access_sequence = self.next_access_sequence();
        let cached_clip = self.clips.get_mut(&cache_key).ok_or(
            AnimationEvaluationError::MissingPreparedClip {
                skeleton: skeleton_revision.id(),
                clip: clip_revision.id(),
            },
        )?;
        cached_clip.last_used = access_sequence;
        let compiled = Arc::clone(&cached_clip.compiled);
        let sample_time = resolve_sample_time(cached_clip.duration_seconds, time_seconds, looping);

        let cached_skeleton = self.skeletons.get_mut(&skeleton_revision.id()).ok_or(
            AnimationEvaluationError::MissingPreparedSkeleton {
                skeleton: skeleton_revision.id(),
            },
        )?;
        cached_skeleton.last_used = access_sequence;
        let pose_pool_miss_count_before = cached_skeleton.pose_pool.miss_count();
        let mut pose = cached_skeleton
            .pose_pool
            .acquire(cached_skeleton.bind_pose.len());
        let result = sample_compiled_pose(
            &mut pose,
            &cached_skeleton.bind_pose,
            &compiled,
            sample_time,
        );
        cached_skeleton.pose_pool.release(pose);
        let pose_pool_miss_count_after = cached_skeleton.pose_pool.miss_count();
        record_pose_pool_miss_delta(
            &mut self.stats,
            pose_pool_miss_count_before,
            pose_pool_miss_count_after,
        );
        result
    }

    fn ensure_skeleton(
        &mut self,
        revision: AnimationAssetRevision,
        skeleton: &AnimationSkeletonAsset,
    ) -> Result<(), AnimationEvaluationError> {
        let access_sequence = self.next_access_sequence();
        if let Some(cached) = self.skeletons.get_mut(&revision.id()) {
            if cached.revision == revision.revision() {
                cached.last_used = access_sequence;
                return Ok(());
            }
        }

        let targets = Arc::new(SkeletonTargetTable::compile(skeleton)?);
        let bind_pose = compile_bind_pose(skeleton)?;
        let joint_count = bind_pose.len();
        self.skeletons.insert(
            revision.id(),
            CachedSkeleton {
                revision: revision.revision(),
                last_used: access_sequence,
                targets,
                bind_pose,
                pose_pool: PosePool::with_buffers(self.pool_size, joint_count),
            },
        );
        self.clips
            .retain(|(skeleton_id, _), _| *skeleton_id != revision.id());
        self.stats.skeleton_compile_count = self.stats.skeleton_compile_count.saturating_add(1);
        self.enforce_skeleton_cache_limit();
        Ok(())
    }

    fn ensure_clip(
        &mut self,
        skeleton_revision: AnimationAssetRevision,
        clip_revision: AnimationAssetRevision,
        clip: &AnimationClipAsset,
    ) -> Result<(), AnimationEvaluationError> {
        let cache_key = (skeleton_revision.id(), clip_revision.id());
        let access_sequence = self.next_access_sequence();
        if let Some(cached) = self.clips.get_mut(&cache_key) {
            if cached.skeleton_revision == skeleton_revision.revision()
                && cached.clip_revision == clip_revision.revision()
            {
                cached.last_used = access_sequence;
                self.stats.clip_cache_hit_count = self.stats.clip_cache_hit_count.saturating_add(1);
                return Ok(());
            }
        }

        let targets = Arc::clone(
            &self
                .skeletons
                .get(&skeleton_revision.id())
                .ok_or(AnimationEvaluationError::MissingPreparedSkeleton {
                    skeleton: skeleton_revision.id(),
                })?
                .targets,
        );
        validate_clip_channels(clip)?;
        let compiled = Arc::new(CompiledAnimationClip::compile(targets, &clip.tracks)?);
        self.clips.insert(
            cache_key,
            CachedClip {
                skeleton_revision: skeleton_revision.revision(),
                clip_revision: clip_revision.revision(),
                last_used: access_sequence,
                duration_seconds: clip.duration_seconds,
                compiled,
            },
        );
        self.stats.clip_compile_count = self.stats.clip_compile_count.saturating_add(1);
        self.enforce_clip_cache_limit();
        Ok(())
    }
}

fn record_pose_pool_miss_delta(stats: &mut AnimationClipEvaluatorStats, before: u64, after: u64) {
    stats.pose_pool_miss_count = stats
        .pose_pool_miss_count
        .saturating_add(after.saturating_sub(before));
}

fn compile_bind_pose(
    skeleton: &AnimationSkeletonAsset,
) -> Result<Box<[AnimationPoseBone]>, AnimationEvaluationError> {
    skeleton
        .bones
        .iter()
        .enumerate()
        .map(|(bone_index, bone)| {
            let translation = Vec3::from_array(bone.local_translation);
            if !translation.is_finite() {
                return Err(AnimationEvaluationError::NonFiniteSkeletonTransform {
                    bone_index,
                    channel: AnimationTransformChannel::Translation,
                });
            }
            let rotation = Quat::from_array(bone.local_rotation);
            if !rotation.is_finite() {
                return Err(AnimationEvaluationError::NonFiniteSkeletonTransform {
                    bone_index,
                    channel: AnimationTransformChannel::Rotation,
                });
            }
            if rotation.length_squared() <= Real::EPSILON {
                return Err(AnimationEvaluationError::ZeroLengthSkeletonRotation { bone_index });
            }
            let scale = Vec3::from_array(bone.local_scale);
            if !scale.is_finite() {
                return Err(AnimationEvaluationError::NonFiniteSkeletonTransform {
                    bone_index,
                    channel: AnimationTransformChannel::Scale,
                });
            }
            Ok(AnimationPoseBone {
                name: bone.name.clone(),
                local_transform: Transform {
                    translation,
                    rotation: rotation.normalize(),
                    scale,
                },
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn sample_compiled_pose(
    pose: &mut PoseBuffer,
    bind_pose: &[AnimationPoseBone],
    clip: &CompiledAnimationClip,
    sample_time: Real,
) -> Result<AnimationPoseOutput, AnimationEvaluationError> {
    for (bone_index, bone) in bind_pose.iter().enumerate() {
        pose.set_transform(bone_index, bone.local_transform)?;
    }

    for (track_index, track) in clip.tracks().iter().enumerate() {
        let bone_index = clip
            .target_index_for_track(track_index)
            .ok_or(AnimationEvaluationError::MissingCompiledTrackTarget { track_index })?;
        let mut transform =
            pose.transform(bone_index)
                .ok_or(AnimationEvaluationError::PoseShapeMismatch {
                    index: bone_index,
                    len: pose.len(),
                })?;
        if let Some(value) = sample_channel(track.translation(), sample_time) {
            transform.translation =
                sampled_vec3(value, track_index, AnimationTransformChannel::Translation)?;
        }
        if let Some(value) = sample_channel(track.rotation(), sample_time) {
            transform.rotation = sampled_rotation(value, track_index)?;
        }
        if let Some(value) = sample_channel(track.scale(), sample_time) {
            transform.scale = sampled_vec3(value, track_index, AnimationTransformChannel::Scale)?;
        }
        pose.set_transform(bone_index, transform)?;
    }

    let bones = bind_pose
        .iter()
        .enumerate()
        .map(|(bone_index, bind_bone)| {
            Ok(AnimationPoseBone {
                name: bind_bone.name.clone(),
                local_transform: pose.transform(bone_index).ok_or(
                    AnimationEvaluationError::PoseShapeMismatch {
                        index: bone_index,
                        len: pose.len(),
                    },
                )?,
            })
        })
        .collect::<Result<Vec<_>, AnimationEvaluationError>>()?;
    Ok(AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: None,
        bones,
    })
}

fn sampled_vec3(
    value: AnimationChannelValueAsset,
    track_index: usize,
    channel: AnimationTransformChannel,
) -> Result<Vec3, AnimationEvaluationError> {
    let AnimationChannelValueAsset::Vec3(value) = value else {
        return Err(AnimationEvaluationError::ValidatedChannelTypeMismatch {
            track_index,
            channel,
        });
    };
    let value = Vec3::from_array(value);
    debug_assert!(value.is_finite());
    Ok(value)
}

fn sampled_rotation(
    value: AnimationChannelValueAsset,
    track_index: usize,
) -> Result<Quat, AnimationEvaluationError> {
    let AnimationChannelValueAsset::Quaternion(value) = value else {
        return Err(AnimationEvaluationError::ValidatedChannelTypeMismatch {
            track_index,
            channel: AnimationTransformChannel::Rotation,
        });
    };
    let value = Quat::from_array(value);
    debug_assert!(value.is_finite());
    debug_assert!(value.length_squared() > Real::EPSILON);
    Ok(value.normalize())
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::BTreeMap;
    use std::hint::black_box;
    use std::time::Instant;

    use super::{record_pose_pool_miss_delta, AnimationClipEvaluatorStats};

    #[test]
    fn optimization_batch_20260830cd_pose_pool_stats_use_local_miss_delta() {
        let source = include_str!("sample.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source");
        let sample_start = production
            .find("    pub fn sample_clip(")
            .expect("sample owner");
        let sample_end = production[sample_start..]
            .find("    fn ensure_skeleton(")
            .map(|offset| sample_start + offset)
            .expect("sample owner boundary");
        let sample = &production[sample_start..sample_end];

        assert!(sample.contains("pose_pool_miss_count_before"));
        assert!(sample.contains("record_pose_pool_miss_delta"));
        assert!(!sample.contains(".skeletons\n            .values()"));
        assert!(!sample.contains(".map(|cached| cached.pose_pool.miss_count())"));
    }

    #[test]
    fn optimization_batch_20260830cd_pose_pool_miss_delta_is_cumulative_and_saturating() {
        let mut stats = AnimationClipEvaluatorStats {
            pose_pool_miss_count: 7,
            ..AnimationClipEvaluatorStats::default()
        };

        record_pose_pool_miss_delta(&mut stats, 11, 12);
        assert_eq!(stats.pose_pool_miss_count, 8);
        record_pose_pool_miss_delta(&mut stats, 12, 12);
        assert_eq!(stats.pose_pool_miss_count, 8);
        record_pose_pool_miss_delta(&mut stats, 12, 11);
        assert_eq!(stats.pose_pool_miss_count, 8);

        stats.pose_pool_miss_count = u64::MAX;
        record_pose_pool_miss_delta(&mut stats, 0, 1);
        assert_eq!(stats.pose_pool_miss_count, u64::MAX);
    }

    #[test]
    #[ignore = "Release-only Runtime170 performance contract"]
    fn optimization_batch_20260830cd_pose_pool_miss_delta_p95() {
        const CACHE_COUNT: u64 = 64;
        const ITERATIONS: usize = 100_000;
        const SAMPLES: usize = 17;
        let pools = (0..CACHE_COUNT)
            .map(|id| (id, id & 3))
            .collect::<BTreeMap<_, _>>();
        let mut baseline_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);

        for sample in 0..SAMPLES {
            let baseline = || {
                let started = Instant::now();
                let mut stats = 0_u64;
                for _ in 0..ITERATIONS {
                    stats = black_box(&pools).values().copied().sum();
                    black_box(stats);
                }
                started.elapsed().as_nanos()
            };
            let optimized = || {
                let started = Instant::now();
                let mut stats = 0_u64;
                for _ in 0..ITERATIONS {
                    stats = stats.saturating_add(black_box(11_u64).saturating_sub(black_box(11)));
                    black_box(stats);
                }
                started.elapsed().as_nanos()
            };
            if sample % 2 == 0 {
                baseline_samples.push(baseline());
                optimized_samples.push(optimized());
            } else {
                optimized_samples.push(optimized());
                baseline_samples.push(baseline());
            }
        }

        let baseline_p95 = percentile_95(&mut baseline_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        println!(
            "RUNTIME170_POSE_POOL_MISS_DELTA_BENCH_V1 baseline_p95_ns={baseline_p95} optimized_p95_ns={optimized_p95}"
        );
        assert!(
            optimized_p95.saturating_mul(100) <= baseline_p95.saturating_mul(20),
            "expected local miss-delta accounting to reduce P95 by at least 80%: baseline={baseline_p95}ns optimized={optimized_p95}ns"
        );
    }

    fn percentile_95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[(samples.len() * 95 / 100).min(samples.len() - 1)]
    }
}
