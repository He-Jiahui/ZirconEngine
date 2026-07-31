use super::super::super::super::{RetainedEditorHost, UiPoint, callback_dispatch};

impl RetainedEditorHost {
    pub(super) fn dispatch_asset_reference_pointer_click_to_bridge(
        &mut self,
        surface_mode: &str,
        list_kind: &str,
        point: UiPoint,
    ) {
        if !self.ensure_asset_surface_bridge() {
            return;
        }
        let Some(bridge) = self.asset_surface_bridge.as_ref() else {
            self.set_status_line("Asset UI controls are not available");
            return;
        };
        let runtime = &self.runtime;
        let dispatch = match (surface_mode, list_kind) {
            ("activity", "references") => {
                callback_dispatch::dispatch_shared_asset_reference_pointer_click(
                    runtime,
                    bridge,
                    &mut self.activity_asset_pointer.references.bridge,
                    point,
                )
            }
            ("activity", "used_by") => {
                callback_dispatch::dispatch_shared_asset_reference_pointer_click(
                    runtime,
                    bridge,
                    &mut self.activity_asset_pointer.used_by.bridge,
                    point,
                )
            }
            ("browser", "references") => {
                callback_dispatch::dispatch_shared_asset_reference_pointer_click(
                    runtime,
                    bridge,
                    &mut self.browser_asset_pointer.references.bridge,
                    point,
                )
            }
            ("browser", "used_by") => {
                callback_dispatch::dispatch_shared_asset_reference_pointer_click(
                    runtime,
                    bridge,
                    &mut self.browser_asset_pointer.used_by.bridge,
                    point,
                )
            }
            _ => {
                self.set_status_line(format!(
                    "Unknown asset reference pointer target {surface_mode}/{list_kind}"
                ));
                return;
            }
        };

        match dispatch {
            Ok(dispatch) => {
                self.write_asset_reference_pointer_state(
                    surface_mode,
                    list_kind,
                    dispatch.pointer.state,
                );
                if let Some(effects) = dispatch.effects {
                    self.apply_dispatch_effects(effects);
                }
            }
            Err(error) => self.set_status_line(error),
        }
    }
}
