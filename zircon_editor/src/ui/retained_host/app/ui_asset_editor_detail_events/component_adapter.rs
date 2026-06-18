use super::*;
use zircon_runtime_interface::ui::component::{
    UiComponentBindingTarget, UiComponentEvent, UiComponentEventEnvelope, UiValue,
};

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_component_adapter_commit(
        &mut self,
        instance_id: &str,
        control_id: &str,
        target_path: &str,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let envelope = UiComponentEventEnvelope::new(
            "ui_asset.editor",
            control_id,
            UiComponentBindingTarget::asset_editor(instance_id.to_string(), target_path),
            UiComponentEvent::Commit {
                property: "value".to_string(),
                value: UiValue::String(value.to_string()),
            },
        )
        .with_component_id(control_id);

        match self.runtime.dispatch_ui_component_adapter_event(&envelope) {
            Ok(result) => {
                if let Some(status_text) = result.status_text {
                    self.set_status_line(status_text);
                }
                if result.changed || result.refresh_projection || !result.patches.is_empty() {
                    self.mark_presentation_dirty();
                }
            }
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
