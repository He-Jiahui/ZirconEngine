//! 帧采样请求以资源快照取得骨架和剪辑修订，再交给剪辑评估缓存；失败转成对应实体的诊断。
use std::collections::BTreeMap;

use zircon_runtime::asset::ProjectAssetManager;
use zircon_runtime::core::framework::animation::AnimationPoseOutput;
use zircon_runtime::core::framework::animation::{AnimationClipAsset, AnimationSkeletonAsset};
use zircon_runtime::core::resource::{
    AnimationClipMarker, AnimationSkeletonMarker, ResourceHandle, ResourceSnapshot,
};
use zircon_runtime::scene::EntityId;

use super::requests::{playback_requires_continuous_frame, PendingPoseSample};
use crate::{AnimationAssetRevision, AnimationClipEvaluator};

pub(super) fn sample_pose_requests(
    evaluator: &mut AnimationClipEvaluator,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingPoseSample>,
) -> BTreeMap<EntityId, AnimationPoseOutput> {
    sample_pose_requests_with_demand(evaluator, asset_manager, pending_samples).0
}

pub(super) fn sample_pose_requests_with_demand(
    evaluator: &mut AnimationClipEvaluator,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingPoseSample>,
) -> (BTreeMap<EntityId, AnimationPoseOutput>, bool) {
    let mut poses = BTreeMap::new();
    let mut requires_continuous_frame = false;
    for pending in pending_samples {
        if let Some((entity, pose, requires_frame)) =
            sample_pose_request_with_demand(evaluator, asset_manager, pending)
        {
            poses.insert(entity, pose);
            requires_continuous_frame |= requires_frame;
        }
    }
    (poses, requires_continuous_frame)
}

/// 通过资源快照形成修订键，保证该次剪辑评估与缓存失效观察同一载荷。
pub(super) fn sample_pose_request(
    evaluator: &mut AnimationClipEvaluator,
    asset_manager: &ProjectAssetManager,
    pending: PendingPoseSample,
) -> Option<(EntityId, AnimationPoseOutput)> {
    sample_pose_request_with_demand(evaluator, asset_manager, pending)
        .map(|(entity, pose, _)| (entity, pose))
}

pub(super) fn sample_pose_request_with_demand(
    evaluator: &mut AnimationClipEvaluator,
    asset_manager: &ProjectAssetManager,
    pending: PendingPoseSample,
) -> Option<(EntityId, AnimationPoseOutput, bool)> {
    let skeleton = load_skeleton_snapshot(asset_manager, pending.skeleton_id)?;
    let clip = load_clip_snapshot(asset_manager, pending.clip_id)?;
    let skeleton_revision = AnimationAssetRevision::new(pending.skeleton_id, skeleton.revision());
    let clip_revision = AnimationAssetRevision::new(pending.clip_id, clip.revision());
    let mut pose = match evaluator.sample_clip(
        skeleton_revision,
        clip_revision,
        &skeleton,
        &clip,
        pending.time_seconds,
        pending.looping,
    ) {
        Ok(pose) => pose,
        Err(error) => {
            evaluator.record_diagnostic(pending.entity, skeleton_revision, clip_revision, error);
            return None;
        }
    };
    // The duration and revisions belong to the exact snapshots sampled above.
    let requires_continuous_frame = !pose.bones.is_empty()
        && playback_requires_continuous_frame(
            pending.playing,
            pending.playback_speed,
            pending.time_seconds,
            clip.duration_seconds,
            pending.looping,
        );
    pose.source = pending.source;
    pose.active_state = pending.active_state;
    Some((pending.entity, pose, requires_continuous_frame))
}

fn load_skeleton_snapshot(
    asset_manager: &ProjectAssetManager,
    asset_id: zircon_runtime::asset::AssetId,
) -> Option<ResourceSnapshot<AnimationSkeletonAsset>> {
    let resources = asset_manager.resource_manager();
    let handle = ResourceHandle::<AnimationSkeletonMarker>::new(asset_id);
    resources.snapshot(handle).or_else(|| {
        asset_manager.load_animation_skeleton_asset(asset_id).ok()?;
        resources.snapshot(handle)
    })
}

fn load_clip_snapshot(
    asset_manager: &ProjectAssetManager,
    asset_id: zircon_runtime::asset::AssetId,
) -> Option<ResourceSnapshot<AnimationClipAsset>> {
    let resources = asset_manager.resource_manager();
    let handle = ResourceHandle::<AnimationClipMarker>::new(asset_id);
    resources.snapshot(handle).or_else(|| {
        asset_manager.load_animation_clip_asset(asset_id).ok()?;
        resources.snapshot(handle)
    })
}

#[cfg(test)]
#[path = "../tests/clip_sample_perf.rs"]
mod optimization_tests;
