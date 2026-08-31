use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::framework::scene::EntityId;

use super::AnimationPoseOutput;

/// Immutable handle for one evaluated entity pose after the animation commit boundary.
pub type AnimationPoseHandle = Arc<AnimationPoseOutput>;

/// Deterministically ordered animation poses sealed for frame consumers.
pub type AnimationPoseMap = BTreeMap<EntityId, AnimationPoseHandle>;

/// Shared animation-pose generation published by a level frame.
pub type AnimationPoseSnapshot = Arc<AnimationPoseMap>;
