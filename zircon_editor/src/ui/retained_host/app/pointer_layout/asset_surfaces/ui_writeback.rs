use super::super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn apply_asset_pointer_state_to_ui(
        &self,
        surface_mode: &str,
    ) {
        let Some(surface) = self.asset_surface_pointer_state(surface_mode) else {
            return;
        };
        let pane_surface_host = self.pane_surface_host();

        let tree_hovered = hovered_row_index(surface.tree_state.hovered_row_index);
        let content_hovered = hovered_row_index(surface.content_state.hovered_row_index);
        let references_hovered = hovered_row_index(surface.references.state.hovered_row_index);
        let used_by_hovered = hovered_row_index(surface.used_by.state.hovered_row_index);

        match surface_mode {
            "activity" => {
                pane_surface_host.set_activity_asset_tree_hovered_index(tree_hovered);
                pane_surface_host
                    .set_activity_asset_tree_scroll_px(surface.tree_state.scroll_offset);
                pane_surface_host.set_activity_asset_content_hovered_index(content_hovered);
                pane_surface_host
                    .set_activity_asset_content_scroll_px(surface.content_state.scroll_offset);
                pane_surface_host.set_activity_asset_references_hovered_index(references_hovered);
                pane_surface_host.set_activity_asset_references_scroll_px(
                    surface.references.state.scroll_offset,
                );
                pane_surface_host.set_activity_asset_used_by_hovered_index(used_by_hovered);
                pane_surface_host
                    .set_activity_asset_used_by_scroll_px(surface.used_by.state.scroll_offset);
            }
            "browser" => {
                pane_surface_host.set_browser_asset_tree_hovered_index(tree_hovered);
                pane_surface_host
                    .set_browser_asset_tree_scroll_px(surface.tree_state.scroll_offset);
                pane_surface_host.set_browser_asset_content_hovered_index(content_hovered);
                pane_surface_host
                    .set_browser_asset_content_scroll_px(surface.content_state.scroll_offset);
                pane_surface_host.set_browser_asset_references_hovered_index(references_hovered);
                pane_surface_host
                    .set_browser_asset_references_scroll_px(surface.references.state.scroll_offset);
                pane_surface_host.set_browser_asset_used_by_hovered_index(used_by_hovered);
                pane_surface_host
                    .set_browser_asset_used_by_scroll_px(surface.used_by.state.scroll_offset);
            }
            _ => {}
        }
    }
}

fn hovered_row_index(index: Option<usize>) -> i32 {
    index.map(|index| index as i32).unwrap_or(-1)
}
