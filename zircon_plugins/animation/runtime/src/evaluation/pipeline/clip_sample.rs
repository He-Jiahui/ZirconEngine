use std::collections::BTreeMap;

use zircon_runtime::asset::{AnimationClipAsset, AnimationSkeletonAsset, ProjectAssetManager};
use zircon_runtime::core::framework::animation::AnimationPoseOutput;
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
    asset_manager.load_animation_skeleton_asset(asset_id).ok()?;
    let resources = asset_manager.resource_manager();
    resources
        .snapshot::<AnimationSkeletonMarker, AnimationSkeletonAsset>(ResourceHandle::new(asset_id))
}

fn load_clip_snapshot(
    asset_manager: &ProjectAssetManager,
    asset_id: zircon_runtime::asset::AssetId,
) -> Option<ResourceSnapshot<AnimationClipAsset>> {
    asset_manager.load_animation_clip_asset(asset_id).ok()?;
    let resources = asset_manager.resource_manager();
    resources.snapshot::<AnimationClipMarker, AnimationClipAsset>(ResourceHandle::new(asset_id))
}
