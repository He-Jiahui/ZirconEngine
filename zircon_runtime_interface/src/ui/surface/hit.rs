use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiTreeId};
use crate::ui::layout::{UiFrame, UiPoint};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiVirtualPointerPosition {
    pub current: UiPoint,
    pub previous: UiPoint,
}

impl UiVirtualPointerPosition {
    pub const fn new(current: UiPoint, previous: UiPoint) -> Self {
        Self { current, previous }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiHitTestQuery {
    pub point: UiPoint,
    pub cursor_radius: f32,
    pub virtual_pointer: Option<UiVirtualPointerPosition>,
}

impl Default for UiHitTestQuery {
    fn default() -> Self {
        Self {
            point: UiPoint::default(),
            cursor_radius: 0.0,
            virtual_pointer: None,
        }
    }
}

impl UiHitTestQuery {
    pub const fn new(point: UiPoint) -> Self {
        Self {
            point,
            cursor_radius: 0.0,
            virtual_pointer: None,
        }
    }

    pub const fn with_cursor_radius(mut self, cursor_radius: f32) -> Self {
        self.cursor_radius = cursor_radius;
        self
    }

    pub const fn with_virtual_pointer(mut self, virtual_pointer: UiVirtualPointerPosition) -> Self {
        self.virtual_pointer = Some(virtual_pointer);
        self
    }

    pub fn hit_point(self) -> UiPoint {
        self.virtual_pointer
            .map(|virtual_pointer| virtual_pointer.current)
            .unwrap_or(self.point)
    }

    pub fn sanitized_cursor_radius(self) -> f32 {
        if self.cursor_radius.is_finite() {
            self.cursor_radius.max(0.0)
        } else {
            0.0
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiHitPath {
    pub target: Option<UiNodeId>,
    pub root_to_leaf: Vec<UiNodeId>,
    pub bubble_route: Vec<UiNodeId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub virtual_pointer: Option<UiVirtualPointerPosition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiHitTestEntry {
    pub node_id: UiNodeId,
    pub frame: UiFrame,
    pub clip_frame: UiFrame,
    pub z_index: i32,
    pub paint_order: u64,
    pub control_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiHitTestCell {
    pub entries: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiHitTestGrid {
    pub bounds: UiFrame,
    pub cell_size: f32,
    pub columns: u32,
    pub rows: u32,
    pub entries: Vec<UiHitTestEntry>,
    pub cells: Vec<UiHitTestCell>,
}

impl Default for UiHitTestGrid {
    fn default() -> Self {
        Self {
            bounds: UiFrame::default(),
            cell_size: 64.0,
            columns: 0,
            rows: 0,
            entries: Vec::new(),
            cells: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiHitTestDebugDump {
    pub tree_id: UiTreeId,
    pub point: crate::ui::layout::UiPoint,
    pub hit_stack: Vec<UiNodeId>,
    pub hit_path: UiHitPath,
    pub inspected: usize,
    pub rejected: Vec<UiHitTestReject>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiHitTestReject {
    pub node_id: UiNodeId,
    pub control_id: Option<String>,
    pub reason: UiHitTestRejectReason,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiHitTestRejectReason {
    OutsideFrame,
    OutsideClip,
    VisibilityFiltered,
    Disabled,
    InputPolicyIgnore,
    NotPointerTarget,
    MissingAncestry,
    StaleGridEntry,
    CustomHitPathUnavailable,
}
