use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::UiLinearSlotSizing;
use super::{Anchor, Pivot, Position};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiMargin {
    #[serde(default)]
    pub left: f32,
    #[serde(default)]
    pub top: f32,
    #[serde(default)]
    pub right: f32,
    #[serde(default)]
    pub bottom: f32,
}

impl UiMargin {
    pub const fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn horizontal(self) -> f32 {
        self.left + self.right
    }

    pub fn vertical(self) -> f32 {
        self.top + self.bottom
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiAlignment {
    #[default]
    Start,
    Center,
    End,
    Fill,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAlignment2D {
    #[serde(default)]
    pub horizontal: UiAlignment,
    #[serde(default)]
    pub vertical: UiAlignment,
}

impl Default for UiAlignment2D {
    fn default() -> Self {
        Self {
            horizontal: UiAlignment::Start,
            vertical: UiAlignment::Start,
        }
    }
}

impl UiAlignment2D {
    pub const fn new(horizontal: UiAlignment, vertical: UiAlignment) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiSlotKind {
    #[default]
    Free,
    Container,
    Overlay,
    Linear,
    Grid,
    Flow,
    Canvas,
    Scrollable,
    Splitter,
    Scale,
}

/// Parent-owned anchor placement payload for Free/Canvas-like panels. Runtime
/// arrange still decides when this preserved contract starts replacing node defaults.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiCanvasSlotPlacement {
    #[serde(default)]
    pub anchor: Anchor,
    #[serde(default)]
    pub pivot: Pivot,
    #[serde(default)]
    pub position: Position,
    #[serde(default)]
    pub offset: UiMargin,
    #[serde(default)]
    pub auto_size: bool,
}

impl UiCanvasSlotPlacement {
    pub const fn new(anchor: Anchor, pivot: Pivot, position: Position) -> Self {
        Self {
            anchor,
            pivot,
            position,
            offset: UiMargin::new(0.0, 0.0, 0.0, 0.0),
            auto_size: false,
        }
    }

    pub fn with_offset(mut self, offset: UiMargin) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_auto_size(mut self, auto_size: bool) -> Self {
        self.auto_size = auto_size;
        self
    }
}

/// Parent-owned cell placement for GridBox children.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiGridSlotPlacement {
    #[serde(default)]
    pub column: usize,
    #[serde(default)]
    pub row: usize,
    #[serde(default = "default_grid_span")]
    pub column_span: usize,
    #[serde(default = "default_grid_span")]
    pub row_span: usize,
}

impl Default for UiGridSlotPlacement {
    fn default() -> Self {
        Self {
            column: 0,
            row: 0,
            column_span: 1,
            row_span: 1,
        }
    }
}

impl UiGridSlotPlacement {
    pub const fn new(column: usize, row: usize) -> Self {
        Self {
            column,
            row,
            column_span: 1,
            row_span: 1,
        }
    }

    pub fn with_span(mut self, column_span: usize, row_span: usize) -> Self {
        self.column_span = column_span.max(1);
        self.row_span = row_span.max(1);
        self
    }
}

/// Parent-owned placement policy for one child; this preserves slot identity
/// across template compilation, layout invalidation, diagnostics, and runtime mutation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSlot {
    pub parent_id: UiNodeId,
    pub child_id: UiNodeId,
    #[serde(default)]
    pub kind: UiSlotKind,
    #[serde(default)]
    pub padding: UiMargin,
    #[serde(default)]
    pub alignment: UiAlignment2D,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub linear_sizing: Option<UiLinearSlotSizing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canvas_placement: Option<UiCanvasSlotPlacement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_placement: Option<UiGridSlotPlacement>,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub z_order: i32,
    #[serde(default)]
    pub dirty_revision: u64,
}

impl UiSlot {
    pub fn new(parent_id: UiNodeId, child_id: UiNodeId, kind: UiSlotKind) -> Self {
        Self {
            parent_id,
            child_id,
            kind,
            padding: UiMargin::default(),
            alignment: UiAlignment2D::default(),
            linear_sizing: None,
            canvas_placement: None,
            grid_placement: None,
            order: 0,
            z_order: 0,
            dirty_revision: 0,
        }
    }

    pub fn with_padding(mut self, padding: UiMargin) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_alignment(mut self, alignment: UiAlignment2D) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_linear_sizing(mut self, linear_sizing: UiLinearSlotSizing) -> Self {
        self.linear_sizing = Some(linear_sizing);
        self
    }

    pub fn with_canvas_placement(mut self, placement: UiCanvasSlotPlacement) -> Self {
        self.canvas_placement = Some(placement);
        self
    }

    pub fn with_grid_placement(mut self, placement: UiGridSlotPlacement) -> Self {
        self.grid_placement = Some(placement);
        self
    }

    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub fn with_z_order(mut self, z_order: i32) -> Self {
        self.z_order = z_order;
        self
    }

    pub fn with_dirty_revision(mut self, dirty_revision: u64) -> Self {
        self.dirty_revision = dirty_revision;
        self
    }
}

const fn default_grid_span() -> usize {
    1
}
