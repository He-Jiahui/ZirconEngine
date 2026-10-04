use serde::{Deserialize, Serialize};

use crate::core::math::Real;

/// 目标筛选与混合权重的作者数据；时间线先按显式目标筛选，再应用此遮罩。
/// 候选既可使用完整路径，也可使用末段名称，调用方需理解后者可能匹配多个目标。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnimationAvatarMask {
    pub id: String,
    pub included_target_ids: Vec<String>,
    pub excluded_target_ids: Vec<String>,
    pub weight: Real,
}

impl Default for AnimationAvatarMask {
    fn default() -> Self {
        Self {
            id: "full_body".to_string(),
            included_target_ids: Vec::new(),
            excluded_target_ids: Vec::new(),
            weight: 1.0,
        }
    }
}

impl AnimationAvatarMask {
    pub fn allows_target(&self, target_id: &str) -> bool {
        let target_id = target_id.trim();
        if target_id.is_empty() {
            return false;
        }
        self.allows_prepared_target(PreparedAnimationTargetId::new(target_id))
    }

    pub(super) fn allows_prepared_target(&self, target: PreparedAnimationTargetId<'_>) -> bool {
        if !self.included_target_ids.is_empty()
            && !self
                .included_target_ids
                .iter()
                .any(|candidate| target.matches(candidate))
        {
            return false;
        }
        !self
            .excluded_target_ids
            .iter()
            .any(|candidate| target.matches(candidate))
    }

    pub fn normalized_weight(&self) -> Real {
        if self.weight.is_finite() {
            self.weight.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Reuses the normalized target leaf across direct, include, and exclude probes.
#[derive(Clone, Copy)]
pub(super) struct PreparedAnimationTargetId<'a> {
    full: &'a str,
    leaf: &'a str,
}

impl<'a> PreparedAnimationTargetId<'a> {
    pub(super) fn new(target_id: &'a str) -> Self {
        Self {
            full: target_id,
            leaf: target_id.rsplit('/').next().unwrap_or(target_id),
        }
    }

    pub(super) fn matches(self, candidate: &str) -> bool {
        let candidate = candidate.trim();
        candidate == self.full
            || candidate
                .rsplit('/')
                .next()
                .is_some_and(|leaf| leaf == self.full)
            || self.leaf == candidate
    }
}

pub(crate) fn animation_target_id_matches(candidate: &str, target_id: &str) -> bool {
    PreparedAnimationTargetId::new(target_id).matches(candidate)
}

#[cfg(test)]
#[path = "tests/avatar_mask.rs"]
mod tests;
