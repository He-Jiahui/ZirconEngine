use super::super::super::*;
use crate::ui::retained_host::HostAssetSurfaceInteractionState;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn apply_asset_pointer_state_to_ui(
        &self,
        surface_mode: &str,
    ) {
        let Some(surface) = self.asset_surface_pointer_state(surface_mode) else {
            return;
        };
        let pane_surface_host = self.pane_surface_host();
        pane_surface_host.set_asset_surface_interaction(
            surface_mode,
            HostAssetSurfaceInteractionState {
                tree_hovered_index: hovered_row_index(surface.tree_state.hovered_row_index),
                tree_scroll_px: surface.tree_state.scroll_offset,
                content_hovered_index: hovered_row_index(surface.content_state.hovered_row_index),
                content_scroll_px: surface.content_state.scroll_offset,
                references_hovered_index: hovered_row_index(
                    surface.references.state.hovered_row_index,
                ),
                references_scroll_px: surface.references.state.scroll_offset,
                used_by_hovered_index: hovered_row_index(surface.used_by.state.hovered_row_index),
                used_by_scroll_px: surface.used_by.state.scroll_offset,
            },
        );
    }
}

fn hovered_row_index(index: Option<usize>) -> i32 {
    index.map(|index| index as i32).unwrap_or(-1)
}
