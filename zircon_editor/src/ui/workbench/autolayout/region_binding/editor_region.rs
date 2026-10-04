use serde::{Deserialize, Serialize};

use crate::ui::workbench::layout::ActivityDrawerSlot;

use super::super::ShellRegionId;
use super::EditorRegionRole;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 作者资产中的六个语义位置；左右上下slot分别保持身份，但共享同侧壳几何。
pub enum EditorRegion {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
    Bottom,
    Center,
}

impl EditorRegion {
    /// 完整性校验和默认drawer设置的闭合集合；新增位置须同步职责映射及资产校验。
    pub const ALL: [Self; 6] = [
        Self::LeftTop,
        Self::LeftBottom,
        Self::RightTop,
        Self::RightBottom,
        Self::Bottom,
        Self::Center,
    ];

    /// 仅工具位置映射抽屉slot，Center由文档系统持有。
    pub fn drawer_slot(self) -> Option<ActivityDrawerSlot> {
        match self {
            Self::LeftTop => Some(ActivityDrawerSlot::LeftTop),
            Self::LeftBottom => Some(ActivityDrawerSlot::LeftBottom),
            Self::RightTop => Some(ActivityDrawerSlot::RightTop),
            Self::RightBottom => Some(ActivityDrawerSlot::RightBottom),
            Self::Bottom => Some(ActivityDrawerSlot::Bottom),
            Self::Center => None,
        }
    }

    /// 把语义slot归并到四个几何区域，用于共同尺寸偏好和约束。
    pub fn shell_region(self) -> ShellRegionId {
        match self {
            Self::LeftTop | Self::LeftBottom => ShellRegionId::Left,
            Self::RightTop | Self::RightBottom => ShellRegionId::Right,
            Self::Bottom => ShellRegionId::Bottom,
            Self::Center => ShellRegionId::Document,
        }
    }

    /// 资产绑定的固定职责约束；用于拒绝位置和内容职责误配。
    pub fn expected_role(self) -> EditorRegionRole {
        match self {
            Self::LeftTop => EditorRegionRole::PlacementTools,
            Self::LeftBottom => EditorRegionRole::ProjectTree,
            Self::RightTop => EditorRegionRole::HierarchyStructure,
            Self::RightBottom => EditorRegionRole::DetailInspector,
            Self::Bottom => EditorRegionRole::ConsoleDiagnosticsTimeline,
            Self::Center => EditorRegionRole::CenterDocument,
        }
    }
}
