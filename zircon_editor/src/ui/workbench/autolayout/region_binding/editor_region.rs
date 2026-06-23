use serde::{Deserialize, Serialize};

use crate::ui::workbench::layout::ActivityDrawerSlot;

use super::super::ShellRegionId;
use super::EditorRegionRole;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditorRegion {
    LeftTop,
    LeftBottom,
    RightTop,
    RightBottom,
    Bottom,
    Center,
}

impl EditorRegion {
    pub const ALL: [Self; 6] = [
        Self::LeftTop,
        Self::LeftBottom,
        Self::RightTop,
        Self::RightBottom,
        Self::Bottom,
        Self::Center,
    ];

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

    pub fn shell_region(self) -> ShellRegionId {
        match self {
            Self::LeftTop | Self::LeftBottom => ShellRegionId::Left,
            Self::RightTop | Self::RightBottom => ShellRegionId::Right,
            Self::Bottom => ShellRegionId::Bottom,
            Self::Center => ShellRegionId::Document,
        }
    }

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
