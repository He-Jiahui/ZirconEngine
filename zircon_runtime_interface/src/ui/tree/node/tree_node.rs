use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiNodePath, UiStateFlags};
use crate::ui::layout::{
    Anchor, BoxConstraints, LayoutBoundary, Pivot, Position, UiContainerKind, UiFrame,
    UiScrollState,
};

use super::{UiDirtyFlags, UiInputPolicy, UiLayoutCache, UiTemplateNodeMetadata, UiVisibility};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiTreeNode {
    pub node_id: UiNodeId,
    pub node_path: UiNodePath,
    pub parent: Option<UiNodeId>,
    pub children: Vec<UiNodeId>,
    pub state_flags: UiStateFlags,
    #[serde(default)]
    pub visibility: UiVisibility,
    #[serde(default)]
    pub constraints: BoxConstraints,
    #[serde(default)]
    pub anchor: Anchor,
    #[serde(default)]
    pub pivot: Pivot,
    #[serde(default)]
    pub position: Position,
    #[serde(default)]
    pub container: UiContainerKind,
    /// Preserves an explicitly-authored stretch axis in linear layout instead of treating
    /// a default-looking stretch constraint as a content-driven fixed fallback.
    #[serde(default)]
    pub layout_stretch_width: bool,
    #[serde(default)]
    pub layout_stretch_height: bool,
    #[serde(default)]
    pub scroll_state: Option<UiScrollState>,
    pub input_policy: UiInputPolicy,
    pub clip_to_bounds: bool,
    pub layout_boundary: LayoutBoundary,
    pub z_index: i32,
    pub paint_order: u64,
    pub dirty: UiDirtyFlags,
    pub layout_cache: UiLayoutCache,
    #[serde(default)]
    pub template_metadata: Option<UiTemplateNodeMetadata>,
}

impl UiTreeNode {
    pub fn new(node_id: UiNodeId, node_path: UiNodePath) -> Self {
        Self {
            node_id,
            node_path,
            parent: None,
            children: Vec::new(),
            state_flags: UiStateFlags {
                visible: true,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            },
            visibility: UiVisibility::Visible,
            constraints: BoxConstraints::default(),
            anchor: Anchor::default(),
            pivot: Pivot::default(),
            position: Position::default(),
            container: UiContainerKind::default(),
            layout_stretch_width: false,
            layout_stretch_height: false,
            scroll_state: None,
            input_policy: UiInputPolicy::Inherit,
            clip_to_bounds: false,
            layout_boundary: LayoutBoundary::ContentDriven,
            z_index: 0,
            paint_order: 0,
            dirty: UiDirtyFlags::default(),
            layout_cache: UiLayoutCache::default(),
            template_metadata: None,
        }
    }

    pub fn with_frame(mut self, frame: UiFrame) -> Self {
        self.layout_cache.frame = frame;
        self
    }

    pub fn with_state_flags(mut self, state_flags: UiStateFlags) -> Self {
        self.state_flags = state_flags;
        self
    }

    pub fn with_visibility(mut self, visibility: UiVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn effective_visibility(&self) -> UiVisibility {
        self.visibility.effective(self.state_flags.visible)
    }

    pub fn is_render_visible(&self) -> bool {
        self.effective_visibility().is_render_visible()
    }

    pub fn is_self_hit_test_visible(&self) -> bool {
        self.effective_visibility().allows_self_hit_test()
    }

    pub fn allows_child_hit_test(&self) -> bool {
        self.effective_visibility().allows_child_hit_test()
    }

    pub fn is_focus_candidate(&self) -> bool {
        self.state_flags.enabled && self.state_flags.focusable && self.is_render_visible()
    }

    pub fn supports_pointer(&self) -> bool {
        self.state_flags.enabled
            && self.is_self_hit_test_visible()
            && (self.state_flags.clickable
                || self.state_flags.hoverable
                || self.state_flags.focusable)
    }

    pub fn with_constraints(mut self, constraints: BoxConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn with_pivot(mut self, pivot: Pivot) -> Self {
        self.pivot = pivot;
        self
    }

    pub fn with_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    pub fn with_container(mut self, container: UiContainerKind) -> Self {
        self.container = container;
        self
    }

    pub fn with_layout_stretch_axes(mut self, width: bool, height: bool) -> Self {
        self.layout_stretch_width = width;
        self.layout_stretch_height = height;
        self
    }

    pub fn with_scroll_state(mut self, scroll_state: UiScrollState) -> Self {
        self.scroll_state = Some(scroll_state);
        self
    }

    pub fn with_input_policy(mut self, input_policy: UiInputPolicy) -> Self {
        self.input_policy = input_policy;
        self
    }

    pub fn with_layout_boundary(mut self, layout_boundary: LayoutBoundary) -> Self {
        self.layout_boundary = layout_boundary;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    pub fn with_clip_to_bounds(mut self, clip_to_bounds: bool) -> Self {
        self.clip_to_bounds = clip_to_bounds;
        self
    }

    pub fn with_template_metadata(mut self, template_metadata: UiTemplateNodeMetadata) -> Self {
        self.template_metadata = Some(template_metadata);
        self
    }
}
