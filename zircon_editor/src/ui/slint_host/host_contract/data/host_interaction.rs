use slint::SharedString;

use super::FrameRect;

/// Menu indices use -1 as the closed/no-hover sentinel so a fresh host never paints a popup.
#[derive(Clone, PartialEq)]
pub(crate) struct HostMenuStateData {
    pub open_menu_index: i32,
    pub hovered_menu_index: i32,
    pub hovered_menu_item_index: i32,
    pub hovered_menu_item_path: Vec<usize>,
    pub open_submenu_path: Vec<usize>,
    pub menu_bar_scroll_px: f32,
    pub window_menu_scroll_px: f32,
    pub window_menu_popup_height_px: f32,
}

impl Default for HostMenuStateData {
    fn default() -> Self {
        Self {
            open_menu_index: -1,
            hovered_menu_index: -1,
            hovered_menu_item_index: -1,
            hovered_menu_item_path: Vec::new(),
            open_submenu_path: Vec::new(),
            menu_bar_scroll_px: 0.0,
            window_menu_scroll_px: 0.0,
            window_menu_popup_height_px: 0.0,
        }
    }
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct HostDragStateData {
    pub active_drag_target_group: SharedString,
    pub drag_active: bool,
    pub drag_tab_id: SharedString,
    pub drag_tab_title: SharedString,
    pub drag_tab_icon_key: SharedString,
    pub drag_source_group: SharedString,
    pub drag_pointer_x: f32,
    pub drag_pointer_y: f32,
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct HostTextInputFocusData {
    pub control_id: SharedString,
    pub dispatch_kind: SharedString,
    pub action_id: SharedString,
    pub edit_action_id: SharedString,
    pub commit_action_id: SharedString,
    pub value_text: SharedString,
    pub edit_frame: FrameRect,
}

impl HostTextInputFocusData {
    pub(crate) fn is_active(&self) -> bool {
        !self.control_id.is_empty()
    }

    pub(crate) fn edit_target_id(&self) -> SharedString {
        if !self.edit_action_id.is_empty() {
            self.edit_action_id.clone()
        } else if !self.action_id.is_empty() {
            self.action_id.clone()
        } else {
            self.control_id.clone()
        }
    }

    pub(crate) fn commit_target_id(&self) -> SharedString {
        if !self.commit_action_id.is_empty() {
            self.commit_action_id.clone()
        } else {
            self.edit_target_id()
        }
    }
}

/// Pointer-only pane state that can repaint native host pixels without rebuilding the whole scene.
#[derive(Clone, PartialEq)]
pub(crate) struct HostPaneInteractionStateData {
    pub hierarchy_scroll_px: f32,
    pub hovered_hierarchy_index: i32,
    pub activity_asset_tree_scroll_px: f32,
    pub activity_asset_tree_hovered_index: i32,
    pub browser_asset_tree_scroll_px: f32,
    pub browser_asset_tree_hovered_index: i32,
}

impl Default for HostPaneInteractionStateData {
    fn default() -> Self {
        Self {
            hierarchy_scroll_px: 0.0,
            hovered_hierarchy_index: -1,
            activity_asset_tree_scroll_px: 0.0,
            activity_asset_tree_hovered_index: -1,
            browser_asset_tree_scroll_px: 0.0,
            browser_asset_tree_hovered_index: -1,
        }
    }
}

#[derive(Clone, Default, PartialEq)]
pub(crate) struct HostResizeStateData {
    pub resize_active: bool,
    pub resize_group: SharedString,
    pub resize_pointer_x: f32,
    pub resize_pointer_y: f32,
}
