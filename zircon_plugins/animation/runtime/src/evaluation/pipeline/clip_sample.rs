//! 帧采样请求以资源快照取得骨架和剪辑修订，再交给剪辑评估缓存；失败转成对应实体的诊断。
use std::collections::BTreeMap;

use zircon_runtime::asset::ProjectAssetManager;
use zircon_runtime::core::framework::animation::AnimationPoseOutput;
use zircon_runtime::core::framework::animation::{AnimationClipAsset, AnimationSkeletonAsset};
use zircon_runtime::core::resource::{
    AnimationClipMarker, AnimationSkeletonMarker, ResourceHandle, ResourceSnapshot,
};
use zircon_runtime::scene::EntityId;

use super::requests::PendingPoseSample;
use crate::{AnimationAssetRevision, AnimationClipEvaluator};

pub(super) fn sample_pose_requests(
    evaluator: &mut AnimationClipEvaluator,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingPoseSample>,
) -> BTreeMap<EntityId, AnimationPoseOutput> {
    pending_samples
        .into_iter()
        .filter_map(|pending| sample_pose_request(evaluator, asset_manager, pending))
        .collect()
}

/// 通过资源快照形成修订键，保证该次剪辑评估与缓存失效观察同一载荷。
pub(super) fn sample_pose_request(
    evaluator: &mut AnimationClipEvaluator,
    asset_manager: &ProjectAssetManager,
    pending: PendingPoseSample,
) -> Option<(EntityId, AnimationPoseOutput)> {
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
    pose.source = pending.source;
    pose.active_state = pending.active_state;
    Some((pending.entity, pose))
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
#[path = "tests/clip_sample_optimization_tests.rs"]
mod optimization_tests;
