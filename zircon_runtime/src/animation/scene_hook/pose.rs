use std::collections::BTreeMap;

use crate::asset::ProjectAssetManager;
use crate::core::framework::animation::{AnimationManager, AnimationPoseOutput};
use crate::scene::EntityId;

use super::pending::PendingPoseSample;

pub(super) fn sample_pose_requests(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    pending_samples: Vec<PendingPoseSample>,
) -> BTreeMap<EntityId, AnimationPoseOutput> {
    pending_samples
        .into_iter()
        .filter_map(|pending| sample_pose_request(animation, asset_manager, pending))
        .collect()
}

pub(super) fn sample_pose_request(
    animation: &dyn AnimationManager,
    asset_manager: &ProjectAssetManager,
    pending: PendingPoseSample,
) -> Option<(EntityId, AnimationPoseOutput)> {
    let skeleton = asset_manager
        .load_animation_skeleton_asset(pending.skeleton_id)
        .ok()?;
    let clip = asset_manager
        .load_animation_clip_asset(pending.clip_id)
        .ok()?;
    let mut pose = animation
        .sample_clip_pose(&skeleton, &clip, pending.time_seconds, pending.looping)
        .ok()?;
    pose.source = pending.source;
    pose.active_state = pending.active_state;
    Some((pending.entity, pose))
}
