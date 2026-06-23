use serde::{Deserialize, Serialize};

use crate::ui::workbench::layout::ActivityDrawerSlot;

use super::super::ShellRegionId;
use super::{EditorRegion, EditorRegionRole, RegionBindingError, WorkbenchConstraintTokenName};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionBinding {
    pub region: EditorRegion,
    pub role: EditorRegionRole,
    pub panel_asset: String,
    pub size_token: Option<WorkbenchConstraintTokenName>,
}

impl RegionBinding {
    pub fn new(
        region: EditorRegion,
        role: EditorRegionRole,
        panel_asset: impl Into<String>,
        size_token: Option<WorkbenchConstraintTokenName>,
    ) -> Result<Self, RegionBindingError> {
        let expected_role = region.expected_role();
        if role != expected_role {
            return Err(RegionBindingError::role_mismatch(
                region,
                expected_role,
                role,
            ));
        }

        Ok(Self {
            region,
            role,
            panel_asset: panel_asset.into(),
            size_token,
        })
    }

    pub(crate) fn from_trusted_parts(
        region: EditorRegion,
        role: EditorRegionRole,
        panel_asset: impl Into<String>,
        size_token: Option<WorkbenchConstraintTokenName>,
    ) -> Self {
        debug_assert_eq!(role, region.expected_role());
        Self {
            region,
            role,
            panel_asset: panel_asset.into(),
            size_token,
        }
    }

    pub fn drawer_slot(&self) -> Option<ActivityDrawerSlot> {
        self.region.drawer_slot()
    }

    pub fn shell_region(&self) -> ShellRegionId {
        self.region.shell_region()
    }
}
