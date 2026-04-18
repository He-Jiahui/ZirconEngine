use serde::{Deserialize, Serialize};

use super::axis_constraint_override::AxisConstraintOverride;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct PaneConstraintOverride {
    #[serde(default)]
    pub width: AxisConstraintOverride,
    #[serde(default)]
    pub height: AxisConstraintOverride,
}

impl PaneConstraintOverride {
    pub fn is_empty(&self) -> bool {
        self.width.is_empty() && self.height.is_empty()
    }
}
