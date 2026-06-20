use super::super::super::RetainedEditorHost;
use zircon_runtime_interface::ui::layout::UiPoint;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn asset_tree_pointer_moved(
        &mut self,
        surface_mode: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        if !self.prepare_asset_tree_pointer_target(surface_mode, width, height) {
            return;
        }

        let dispatch = {
            let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) else {
                self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
                return;
            };
            surface.tree_bridge.handle_move(UiPoint::new(x, y))
        };

        match dispatch {
            Ok(dispatch) => {
                if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
                    surface.tree_state = dispatch.state;
                }
                self.apply_asset_pointer_state_to_ui(surface_mode);
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
