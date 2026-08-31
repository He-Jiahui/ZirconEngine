use super::super::super::super::data::HostPaneInteractionStateData;
use super::super::{HostAssetSurfaceInteractionState, PaneSurfaceHostContext};

impl PaneSurfaceHostContext<'_> {
    fn update_interaction(&self, update: impl FnOnce(&mut HostPaneInteractionStateData)) {
        self.state.borrow_mut().update_pane_interaction(update);
    }

    pub(crate) fn set_asset_surface_interaction(
        &self,
        surface_mode: &str,
        mut next: HostAssetSurfaceInteractionState,
    ) -> bool {
        next.tree_scroll_px = next.tree_scroll_px.max(0.0);
        next.content_scroll_px = next.content_scroll_px.max(0.0);
        next.references_scroll_px = next.references_scroll_px.max(0.0);
        next.used_by_scroll_px = next.used_by_scroll_px.max(0.0);
        let mut state = self.state.borrow_mut();
        let current = state.pane_interaction_state.as_ref();
        let changed = match surface_mode {
            "activity" => {
                current.activity_asset_tree_hovered_index != next.tree_hovered_index
                    || current.activity_asset_tree_scroll_px != next.tree_scroll_px
                    || current.activity_asset_content_hovered_index != next.content_hovered_index
                    || current.activity_asset_content_scroll_px != next.content_scroll_px
                    || current.activity_asset_references_hovered_index
                        != next.references_hovered_index
                    || current.activity_asset_references_scroll_px != next.references_scroll_px
                    || current.activity_asset_used_by_hovered_index != next.used_by_hovered_index
                    || current.activity_asset_used_by_scroll_px != next.used_by_scroll_px
            }
            "browser" => {
                current.browser_asset_tree_hovered_index != next.tree_hovered_index
                    || current.browser_asset_tree_scroll_px != next.tree_scroll_px
                    || current.browser_asset_content_hovered_index != next.content_hovered_index
                    || current.browser_asset_content_scroll_px != next.content_scroll_px
                    || current.browser_asset_references_hovered_index
                        != next.references_hovered_index
                    || current.browser_asset_references_scroll_px != next.references_scroll_px
                    || current.browser_asset_used_by_hovered_index != next.used_by_hovered_index
                    || current.browser_asset_used_by_scroll_px != next.used_by_scroll_px
            }
            _ => false,
        };
        if !changed {
            return false;
        }
        state.update_pane_interaction(|interaction| match surface_mode {
            "activity" => {
                interaction.activity_asset_tree_hovered_index = next.tree_hovered_index;
                interaction.activity_asset_tree_scroll_px = next.tree_scroll_px;
                interaction.activity_asset_content_hovered_index = next.content_hovered_index;
                interaction.activity_asset_content_scroll_px = next.content_scroll_px;
                interaction.activity_asset_references_hovered_index = next.references_hovered_index;
                interaction.activity_asset_references_scroll_px = next.references_scroll_px;
                interaction.activity_asset_used_by_hovered_index = next.used_by_hovered_index;
                interaction.activity_asset_used_by_scroll_px = next.used_by_scroll_px;
            }
            "browser" => {
                interaction.browser_asset_tree_hovered_index = next.tree_hovered_index;
                interaction.browser_asset_tree_scroll_px = next.tree_scroll_px;
                interaction.browser_asset_content_hovered_index = next.content_hovered_index;
                interaction.browser_asset_content_scroll_px = next.content_scroll_px;
                interaction.browser_asset_references_hovered_index = next.references_hovered_index;
                interaction.browser_asset_references_scroll_px = next.references_scroll_px;
                interaction.browser_asset_used_by_hovered_index = next.used_by_hovered_index;
                interaction.browser_asset_used_by_scroll_px = next.used_by_scroll_px;
            }
            _ => unreachable!("surface mode was validated before interaction update"),
        })
    }

    pub(crate) fn set_hierarchy_interaction(&self, scroll_px: f32, hovered_index: i32) -> bool {
        let scroll_px = scroll_px.max(0.0);
        let mut state = self.state.borrow_mut();
        let current = state.pane_interaction_state.as_ref();
        let changed = current.hierarchy_scroll_px != scroll_px
            || current.hovered_hierarchy_index != hovered_index;
        if !changed {
            return false;
        }
        state.update_pane_interaction(|interaction| {
            interaction.hierarchy_scroll_px = scroll_px;
            interaction.hovered_hierarchy_index = hovered_index;
        })
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
