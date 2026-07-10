use std::sync::Arc;

use zircon_runtime::core::framework::animation::AnimationPoseBone;

use crate::{CompiledAnimationClip, PosePool, SkeletonTargetTable};

#[derive(Debug)]
pub(super) struct CachedSkeleton {
    pub revision: u64,
    pub last_used: u64,
    pub targets: Arc<SkeletonTargetTable>,
    pub bind_pose: Box<[AnimationPoseBone]>,
    pub pose_pool: PosePool,
}

#[derive(Debug)]
pub(super) struct CachedClip {
    pub skeleton_revision: u64,
    pub clip_revision: u64,
    pub last_used: u64,
    pub duration_seconds: f32,
    pub compiled: Arc<CompiledAnimationClip>,
}
