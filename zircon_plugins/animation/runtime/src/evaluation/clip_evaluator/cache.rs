//! 每个骨架修订持有目标表、绑定姿态和姿态池；每个剪辑修订持有该骨架上的稠密轨道。
//! 剪辑缓存的目标槽不得跨骨架或骨架修订复用。
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
