use super::super::RetainedEditorHost;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn prepare_asset_tree_pointer_target(
        &mut self,
        surface_mode: &str,
        width: f32,
        height: f32,
    ) -> bool {
        self.use_committed_pointer_layout();
        let Some(tree_size) = self
            .asset_surface_pointer_state(surface_mode)
            .and_then(|surface| {
                self.resolve_callback_surface_size_for_asset_surface(
                    surface_mode,
                    width,
                    height,
                    surface.tree_size,
                )
            })
        else {
            self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
            return false;
        };

        let state_changed = {
            let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
                self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
                return false;
            };
            if surface.tree_size == tree_size {
                return true;
            }
            surface.tree_size = tree_size;
            if let Some(state) = surface.tree_bridge.sync_pane_size(surface.tree_size) {
                surface.tree_state = state;
                true
            } else {
                false
            }
        };
        if state_changed {
            self.apply_asset_pointer_state_to_ui(surface_mode);
        }
        true
    }
}
