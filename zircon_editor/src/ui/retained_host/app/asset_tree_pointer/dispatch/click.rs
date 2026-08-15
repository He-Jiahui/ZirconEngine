use super::super::super::{callback_dispatch, RetainedEditorHost};
use zircon_runtime_interface::ui::layout::UiPoint;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn asset_tree_pointer_clicked(
        &mut self,
        surface_mode: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        self.focus_callback_source_window();
        if !self.prepare_asset_tree_pointer_target(surface_mode, width, height) {
            return;
        }

        if !self.ensure_asset_surface_bridge() {
            return;
        }
        let Some(bridge) = self.asset_surface_bridge.as_ref() else {
            self.set_status_line("Asset UI controls are not available");
            return;
        };
        let runtime = &self.runtime;
        let point = UiPoint::new(x, y);
        let dispatch = match surface_mode {
            "activity" => callback_dispatch::dispatch_shared_asset_tree_pointer_click(
                runtime,
                bridge,
                &mut self.activity_asset_pointer.tree_bridge,
                point,
            ),
            "browser" => callback_dispatch::dispatch_shared_asset_tree_pointer_click(
                runtime,
                bridge,
                &mut self.browser_asset_pointer.tree_bridge,
                point,
            ),
            _ => {
                self.set_status_line(format!("Unknown asset surface mode {surface_mode}"));
                return;
            }
        };

        match dispatch {
            Ok(dispatch) => {
                if let Some(surface) = self.asset_surface_pointer_state_mut(surface_mode) {
                    surface.tree_state = dispatch.pointer.state;
                }
                self.apply_asset_pointer_state_to_ui(surface_mode);
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
