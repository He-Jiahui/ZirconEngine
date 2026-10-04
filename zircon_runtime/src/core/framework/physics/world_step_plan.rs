use serde::{Deserialize, Serialize};

use crate::core::math::Real;

/// 单个世界的物理时钟计划；步数驱动求解，剩余时间与插值权重供呈现层平滑使用。
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PhysicsWorldStepPlan {
    pub steps: u32,
    pub step_seconds: Real,
    pub remaining_seconds: Real,
    #[serde(default)]
    pub interpolation_alpha: Real,
}
