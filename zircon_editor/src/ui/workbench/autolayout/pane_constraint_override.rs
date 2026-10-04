use serde::{Deserialize, Serialize};

use super::axis_constraint_override::AxisConstraintOverride;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
/// 区域或view实例的二维布局差量；所属身份由布局映射key提供，空轴不覆盖已有约束。
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
