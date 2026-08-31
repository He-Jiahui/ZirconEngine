use serde::{Deserialize, Serialize};

use crate::core::math::Real;

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
mod tests {
    use super::*;

    #[test]
    fn optimization_batch_20260830cc_avatar_mask_rejects_include_miss_before_scanning_exclusions() {
        let mask = AnimationAvatarMask {
            id: "upper_body".to_string(),
            included_target_ids: vec!["Rig/Spine/Chest".to_string()],
            excluded_target_ids: vec!["Rig/Face/Jaw".to_string()],
            weight: 1.0,
        };

        assert!(!mask.allows_target("Rig/Hands/Left"));

        let source = include_str!("avatar_mask.rs");
        let allows_target = source
            .split("pub fn allows_target")
            .nth(1)
            .and_then(|source| source.split("pub fn normalized_weight").next())
            .expect("read avatar-mask target filtering");
        let include_guard = allows_target
            .find("if !self.included_target_ids.is_empty()")
            .expect("an include miss must have an explicit early-return guard");
        let exclusion_scan = allows_target
            .find("self.excluded_target_ids")
            .expect("avatar-mask exclusions must remain enforced");

        assert!(
            include_guard < exclusion_scan,
            "include rejection must happen before the exclusion list is scanned"
        );
        assert!(
            source.contains("PreparedAnimationTargetId"),
            "target path normalization must be prepared once and shared by all list probes"
        );
    }

    #[test]
    fn optimization_batch_20260830cc_avatar_mask_leaf_matching_semantics_remain_symmetric() {
        assert!(animation_target_id_matches("Rig/Spine/Chest", "Chest"));
        assert!(animation_target_id_matches("Chest", "Rig/Spine/Chest"));
        assert!(!animation_target_id_matches(
            "Rig/Spine/Chest",
            "Rig/Face/Chest"
        ));
    }
}
