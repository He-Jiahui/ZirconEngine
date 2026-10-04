use serde::{Deserialize, Serialize};

use crate::ui::workbench::layout::ActivityDrawerSlot;

use super::super::ShellRegionId;
use super::{EditorRegion, EditorRegionRole, RegionBindingError, WorkbenchConstraintTokenName};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 外壳作者绑定：位置和职责须匹配，面板资源由模板系统加载，尺寸token由当前主题解析。
pub struct RegionBinding {
    pub region: EditorRegion,
    pub role: EditorRegionRole,
    pub panel_asset: String,
    pub size_token: Option<WorkbenchConstraintTokenName>,
}

impl RegionBinding {
    /// 外部作者数据的职责验证入口；直接serde或修改公开字段的调用方须自行维护此不变量。
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

    /// 仅内建固定组合使用；release不执行职责校验，不能替代外部资产验证入口。
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
