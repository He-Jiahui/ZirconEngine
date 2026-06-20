use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn sync_hierarchy_pointer_layout(
        &mut self,
        scene_entries: &[crate::ui::workbench::snapshot::SceneEntry],
    ) {
        if self.hierarchy_pointer_size.width <= 0.0 || self.hierarchy_pointer_size.height <= 0.0 {
            self.apply_hierarchy_pointer_state_to_ui();
            return;
        }

        self.hierarchy_pointer_bridge.sync(
            HierarchyPointerLayout {
                pane_width: self.hierarchy_pointer_size.width,
                pane_height: self.hierarchy_pointer_size.height,
                node_ids: scene_entries
                    .iter()
                    .map(|entry| entry.id.to_string())
                    .collect(),
            },
            self.hierarchy_pointer_state.clone(),
        );
        self.apply_hierarchy_pointer_state_to_ui();
    }

    pub(in crate::ui::retained_host::app) fn apply_hierarchy_pointer_state_to_ui(&self) {
        let pane_surface_host = self.pane_surface_host();
        pane_surface_host.set_hierarchy_scroll_px(self.hierarchy_pointer_state.scroll_offset);
        pane_surface_host.set_hovered_hierarchy_index(
            self.hierarchy_pointer_state
                .hovered_item_index
                .map(|index| index as i32)
                .unwrap_or(-1),
        );
    }
}
