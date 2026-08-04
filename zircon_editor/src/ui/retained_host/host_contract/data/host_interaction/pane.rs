use crate::ui::retained_host::primitives::SharedString;

use super::super::FrameRect;

/// Pointer-only pane state that can repaint native host pixels without rebuilding the whole scene.
#[derive(Clone, PartialEq)]
pub(crate) struct HostPaneInteractionStateData {
    pub hierarchy_scroll_px: f32,
    pub hovered_hierarchy_index: i32,
    pub console_scroll_px: f32,
    pub activity_asset_tree_scroll_px: f32,
    pub activity_asset_tree_hovered_index: i32,
    pub activity_asset_content_scroll_px: f32,
    pub activity_asset_content_hovered_index: i32,
    pub activity_asset_references_scroll_px: f32,
    pub activity_asset_references_hovered_index: i32,
    pub activity_asset_used_by_scroll_px: f32,
    pub activity_asset_used_by_hovered_index: i32,
    pub activity_asset_reference_hover_frame: FrameRect,
    pub browser_asset_tree_scroll_px: f32,
    pub browser_asset_tree_hovered_index: i32,
    pub browser_asset_content_scroll_px: f32,
    pub browser_asset_content_hovered_index: i32,
    pub browser_asset_references_scroll_px: f32,
    pub browser_asset_references_hovered_index: i32,
    pub browser_asset_used_by_scroll_px: f32,
    pub browser_asset_used_by_hovered_index: i32,
    pub browser_asset_reference_hover_frame: FrameRect,
    pub hovered_template_control_id: SharedString,
    pub hovered_template_dispatch_kind: SharedString,
    pub hovered_template_action_id: SharedString,
    pub hovered_template_value_text: SharedString,
    pub hovered_template_frame: FrameRect,
}

impl Default for HostPaneInteractionStateData {
    fn default() -> Self {
        Self {
            hierarchy_scroll_px: 0.0,
            hovered_hierarchy_index: -1,
            console_scroll_px: 0.0,
            activity_asset_tree_scroll_px: 0.0,
            activity_asset_tree_hovered_index: -1,
            activity_asset_content_scroll_px: 0.0,
            activity_asset_content_hovered_index: -1,
            activity_asset_references_scroll_px: 0.0,
            activity_asset_references_hovered_index: -1,
            activity_asset_used_by_scroll_px: 0.0,
            activity_asset_used_by_hovered_index: -1,
            activity_asset_reference_hover_frame: FrameRect::default(),
            browser_asset_tree_scroll_px: 0.0,
            browser_asset_tree_hovered_index: -1,
            browser_asset_content_scroll_px: 0.0,
            browser_asset_content_hovered_index: -1,
            browser_asset_references_scroll_px: 0.0,
            browser_asset_references_hovered_index: -1,
            browser_asset_used_by_scroll_px: 0.0,
            browser_asset_used_by_hovered_index: -1,
            browser_asset_reference_hover_frame: FrameRect::default(),
            hovered_template_control_id: SharedString::default(),
            hovered_template_dispatch_kind: SharedString::default(),
            hovered_template_action_id: SharedString::default(),
            hovered_template_value_text: SharedString::default(),
            hovered_template_frame: FrameRect::default(),
        }
    }
}
