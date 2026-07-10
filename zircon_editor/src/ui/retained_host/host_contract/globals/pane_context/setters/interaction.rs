use super::super::PaneSurfaceHostContext;

impl PaneSurfaceHostContext<'_> {
    pub(crate) fn set_hierarchy_scroll_px(&self, value: f32) {
        self.state
            .borrow_mut()
            .pane_interaction_state
            .hierarchy_scroll_px = value.max(0.0);
    }

    pub(crate) fn set_hovered_hierarchy_index(&self, value: i32) {
        self.state
            .borrow_mut()
            .pane_interaction_state
            .hovered_hierarchy_index = value;
    }

    pub(crate) fn set_console_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_inspector_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_browser_asset_details_scroll_px(&self, _value: f32) {}

    pub(crate) fn set_activity_asset_tree_hovered_index(&self, value: i32) {
        self.state
            .borrow_mut()
            .pane_interaction_state
            .activity_asset_tree_hovered_index = value;
    }

    pub(crate) fn set_activity_asset_tree_scroll_px(&self, value: f32) {
        self.state
            .borrow_mut()
            .pane_interaction_state
            .activity_asset_tree_scroll_px = value.max(0.0);
    }

    pub(crate) fn set_activity_asset_content_hovered_index(&self, value: i32) {
        self.state
            .borrow_mut()
            .pane_interaction_state
            .activity_asset_content_hovered_index = value;
    }

    pub(crate) fn set_activity_asset_content_scroll_px(&self, value: f32) {
        self.state
            .borrow_mut()
            .pane_interaction_state
            .activity_asset_content_scroll_px = value.max(0.0);
    }
    pub(crate) fn set_activity_asset_references_hovered_index(&self, _value: i32) {}
    pub(crate) fn set_activity_asset_references_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_activity_asset_used_by_hovered_index(&self, _value: i32) {}
    pub(crate) fn set_activity_asset_used_by_scroll_px(&self, _value: f32) {}

    pub(crate) fn set_browser_asset_tree_hovered_index(&self, value: i32) {
        self.state
            .borrow_mut()
            .pane_interaction_state
            .browser_asset_tree_hovered_index = value;
    }

    pub(crate) fn set_browser_asset_tree_scroll_px(&self, value: f32) {
        self.state
            .borrow_mut()
            .pane_interaction_state
            .browser_asset_tree_scroll_px = value.max(0.0);
    }

    pub(crate) fn set_browser_asset_content_hovered_index(&self, value: i32) {
        self.state
            .borrow_mut()
            .pane_interaction_state
            .browser_asset_content_hovered_index = value;
    }

    pub(crate) fn set_browser_asset_content_scroll_px(&self, value: f32) {
        self.state
            .borrow_mut()
            .pane_interaction_state
            .browser_asset_content_scroll_px = value.max(0.0);
    }
    pub(crate) fn set_browser_asset_references_hovered_index(&self, _value: i32) {}
    pub(crate) fn set_browser_asset_references_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_browser_asset_used_by_hovered_index(&self, _value: i32) {}
    pub(crate) fn set_browser_asset_used_by_scroll_px(&self, _value: f32) {}
}

#[cfg(test)]
mod tests;
