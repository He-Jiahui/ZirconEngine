use serde::{Deserialize, Serialize};

use super::{EditorRegion, EditorRegionRole};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionBindingError {
    region: EditorRegion,
    expected_role: EditorRegionRole,
    actual_role: EditorRegionRole,
}

impl RegionBindingError {
    pub(super) fn role_mismatch(
        region: EditorRegion,
        expected_role: EditorRegionRole,
        actual_role: EditorRegionRole,
    ) -> Self {
        Self {
            region,
            expected_role,
            actual_role,
        }
    }

    pub fn region(&self) -> EditorRegion {
        self.region
    }

    pub fn expected_role(&self) -> EditorRegionRole {
        self.expected_role
    }

    pub fn actual_role(&self) -> EditorRegionRole {
        self.actual_role
    }
}
