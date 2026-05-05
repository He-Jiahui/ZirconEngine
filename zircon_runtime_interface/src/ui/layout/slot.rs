use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiMargin {
    pub left: f32,
    pub top: f32,
    pub right: f32,
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
    pub horizontal: UiAlignment,
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

/// Parent-owned placement policy for one child; this preserves slot identity
/// across template compilation, layout invalidation, diagnostics, and runtime mutation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSlot {
    pub parent_id: UiNodeId,
    pub child_id: UiNodeId,
    pub kind: UiSlotKind,
    pub padding: UiMargin,
    pub alignment: UiAlignment2D,
    pub order: i32,
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
            order: 0,
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

    pub fn with_order(mut self, order: i32) -> Self {
        self.order = order;
        self
    }

    pub fn with_dirty_revision(mut self, dirty_revision: u64) -> Self {
        self.dirty_revision = dirty_revision;
        self
    }
}
