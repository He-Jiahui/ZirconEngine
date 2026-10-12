use super::super::super::{HostInvalidationMask, RetainedEditorHost};
use super::field_ids::inspector_field_id;
use zircon_runtime_interface::ui::component::{
    UiComponentBindingTarget, UiComponentEvent, UiComponentEventEnvelope, UiValue,
};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_inspector_control_changed(
        &mut self,
        control_id: &str,
        value: &str,
    ) {
        let Some(field_id) = inspector_field_id(control_id) else {
            self.set_status_line(format!("Unknown inspector change control {control_id}"));
            return;
        };

        self.focus_callback_source_window();
        let envelope = UiComponentEventEnvelope::new(
            "res://ui/editor/host/inspector_surface_controls.zui",
            control_id,
            UiComponentBindingTarget::inspector("entity://selected", field_id.clone()),
            UiComponentEvent::ValueChanged {
                property: "value".to_string(),
                value: UiValue::String(value.to_string()),
            },
        )
        .with_component_id("InspectorField");

        match self.runtime.dispatch_ui_component_adapter_event(&envelope) {
            Ok(result) => {
                let refresh_presentation = result.refresh_projection || result.changed;
                self.set_status_line(
                    result
                        .status_text
                        .unwrap_or_else(|| format!("Inspector field updated: {field_id}")),
                );
                if refresh_presentation {
                    self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
                }
            }
            Err(error) => {
                self.set_status_line(format!("Inspector component binding failed: {error}"));
            }
        }
    }
}
