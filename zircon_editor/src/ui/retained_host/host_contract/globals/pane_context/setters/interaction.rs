use super::super::super::super::data::HostPaneInteractionStateData;
use super::super::PaneSurfaceHostContext;

impl PaneSurfaceHostContext<'_> {
    fn update_interaction(&self, update: impl FnOnce(&mut HostPaneInteractionStateData)) {
        self.state.borrow_mut().update_pane_interaction(update);
    }

    pub(crate) fn set_hierarchy_scroll_px(&self, value: f32) {
        self.update_interaction(|state| state.hierarchy_scroll_px = value.max(0.0));
    }

    pub(crate) fn set_hovered_hierarchy_index(&self, value: i32) {
        self.update_interaction(|state| state.hovered_hierarchy_index = value);
    }

    pub(crate) fn set_console_scroll_px(&self, value: f32) {
        self.update_interaction(|state| state.console_scroll_px = value.max(0.0));
    }
    pub(crate) fn set_inspector_scroll_px(&self, _value: f32) {}
    pub(crate) fn set_browser_asset_details_scroll_px(&self, _value: f32) {}

    pub(crate) fn set_activity_asset_tree_hovered_index(&self, value: i32) {
        self.update_interaction(|state| state.activity_asset_tree_hovered_index = value);
    }

    pub(crate) fn set_activity_asset_tree_scroll_px(&self, value: f32) {
        self.update_interaction(|state| state.activity_asset_tree_scroll_px = value.max(0.0));
    }

    pub(crate) fn set_activity_asset_content_hovered_index(&self, value: i32) {
        self.update_interaction(|state| state.activity_asset_content_hovered_index = value);
    }

    pub(crate) fn set_activity_asset_content_scroll_px(&self, value: f32) {
        self.update_interaction(|state| state.activity_asset_content_scroll_px = value.max(0.0));
    }
    pub(crate) fn set_activity_asset_references_hovered_index(&self, value: i32) {
        self.update_interaction(|state| state.activity_asset_references_hovered_index = value);
    }

    pub(crate) fn set_activity_asset_references_scroll_px(&self, value: f32) {
        self.update_interaction(|state| {
            state.activity_asset_references_scroll_px = value.max(0.0);
        });
    }

    pub(crate) fn set_activity_asset_used_by_hovered_index(&self, value: i32) {
        self.update_interaction(|state| state.activity_asset_used_by_hovered_index = value);
    }

    pub(crate) fn set_activity_asset_used_by_scroll_px(&self, value: f32) {
        self.update_interaction(|state| state.activity_asset_used_by_scroll_px = value.max(0.0));
    }

    pub(crate) fn set_activity_asset_reference_hover_frame(
        &self,
        value: crate::ui::retained_host::host_contract::data::FrameRect,
    ) {
        self.update_interaction(|state| state.activity_asset_reference_hover_frame = value);
    }

    pub(crate) fn set_browser_asset_tree_hovered_index(&self, value: i32) {
        self.update_interaction(|state| state.browser_asset_tree_hovered_index = value);
    }

    pub(crate) fn set_browser_asset_tree_scroll_px(&self, value: f32) {
        self.update_interaction(|state| state.browser_asset_tree_scroll_px = value.max(0.0));
    }

    pub(crate) fn set_browser_asset_content_hovered_index(&self, value: i32) {
        self.update_interaction(|state| state.browser_asset_content_hovered_index = value);
    }

    pub(crate) fn set_browser_asset_content_scroll_px(&self, value: f32) {
        self.update_interaction(|state| state.browser_asset_content_scroll_px = value.max(0.0));
    }
    pub(crate) fn set_browser_asset_references_hovered_index(&self, value: i32) {
        self.update_interaction(|state| state.browser_asset_references_hovered_index = value);
    }

    pub(crate) fn set_browser_asset_references_scroll_px(&self, value: f32) {
        self.update_interaction(|state| {
            state.browser_asset_references_scroll_px = value.max(0.0);
        });
    }

    pub(crate) fn set_browser_asset_used_by_hovered_index(&self, value: i32) {
        self.update_interaction(|state| state.browser_asset_used_by_hovered_index = value);
    }

    pub(crate) fn set_browser_asset_used_by_scroll_px(&self, value: f32) {
        self.update_interaction(|state| state.browser_asset_used_by_scroll_px = value.max(0.0));
    }

    pub(crate) fn set_browser_asset_reference_hover_frame(
        &self,
        value: crate::ui::retained_host::host_contract::data::FrameRect,
    ) {
        self.update_interaction(|state| state.browser_asset_reference_hover_frame = value);
    }
}

#[cfg(test)]
mod tests;
