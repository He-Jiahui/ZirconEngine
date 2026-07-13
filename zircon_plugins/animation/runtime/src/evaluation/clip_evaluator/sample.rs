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
    AnimationAssetRevision, AnimationClipEvaluator, AnimationEvaluationError,
    AnimationTransformChannel,
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
        self.stats.pose_pool_miss_count = self
            .skeletons
            .values()
            .map(|cached| cached.pose_pool.miss_count())
            .sum();
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
