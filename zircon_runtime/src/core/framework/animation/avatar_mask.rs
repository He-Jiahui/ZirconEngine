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

        let included = self.included_target_ids.is_empty()
            || self
                .included_target_ids
                .iter()
                .any(|candidate| animation_target_id_matches(candidate, target_id));
        let excluded = self
            .excluded_target_ids
            .iter()
            .any(|candidate| animation_target_id_matches(candidate, target_id));

        included && !excluded
    }

    pub fn normalized_weight(&self) -> Real {
        if self.weight.is_finite() {
            self.weight.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

pub(crate) fn animation_target_id_matches(candidate: &str, target_id: &str) -> bool {
    let candidate = candidate.trim();
    candidate == target_id
        || candidate
            .rsplit('/')
            .next()
            .is_some_and(|leaf| leaf == target_id)
        || target_id
            .rsplit('/')
            .next()
            .is_some_and(|leaf| leaf == candidate)
}
