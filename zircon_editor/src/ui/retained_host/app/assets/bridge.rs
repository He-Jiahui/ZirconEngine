use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn ensure_asset_surface_bridge(&mut self) -> bool {
        if self.asset_surface_bridge.is_some() {
            return true;
        }
        zircon_runtime::profile_scope!("editor", "retained_host", "lazy_asset_surface_bridge");
        match callback_dispatch::BuiltinAssetSurfaceTemplateBridge::new_minimal() {
            Ok(bridge) => {
                self.asset_surface_bridge = Some(bridge);
                true
            }
            Err(error) => {
                self.set_status_line(format!("Failed to load asset UI controls: {error}"));
                false
            }
        }
    }

    pub(in crate::ui::retained_host::app) fn dispatch_asset_surface_control(
        &mut self,
        control_id: &str,
        event_kind: UiEventKind,
        arguments: Vec<UiBindingValue>,
    ) {
        self.focus_callback_source_window();
        if !self.ensure_asset_surface_bridge() {
            return;
        }
        let Some(asset_surface_bridge) = self.asset_surface_bridge.as_ref() else {
            self.set_status_line("Asset UI controls are not available");
            return;
        };
        let Some(result) = callback_dispatch::dispatch_builtin_asset_surface_control(
            &self.runtime,
            asset_surface_bridge,
            control_id,
            event_kind,
            arguments,
        ) else {
            self.set_status_line(format!("Unknown asset surface control {control_id}"));
            return;
        };
        self.apply_dispatch_result(result);
    }
}
