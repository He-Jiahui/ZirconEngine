use super::super::*;
use super::MATERIAL_LAB_BINDING_PREFIX;
use crate::ui::template_runtime::{UiComponentShowcaseDemoEventInput, SHOWCASE_DOCUMENT_ID};

impl RetainedEditorHost {
    pub(super) fn dispatch_component_showcase_event(
        &mut self,
        control_id: &str,
        action_id: &str,
        input: UiComponentShowcaseDemoEventInput,
    ) {
        if action_id.starts_with(MATERIAL_LAB_BINDING_PREFIX) {
            self.dispatch_material_lab_event(control_id, action_id);
            return;
        }
        if let Err(error) = self.ensure_component_showcase_runtime_loaded() {
            self.set_status_line(error);
            return;
        }

        let binding = self
            .component_showcase_runtime
            .project_document(SHOWCASE_DOCUMENT_ID)
            .ok()
            .and_then(|projection| {
                projection
                    .bindings
                    .into_iter()
                    .find(|binding| binding.binding_id == action_id)
            });
        let Some(binding) = binding else {
            self.set_status_line(format!("Unknown component showcase action {action_id}"));
            return;
        };

        match self
            .component_showcase_runtime
            .apply_showcase_demo_binding(&binding.binding, input)
        {
            Ok(result) => {
                self.set_status_line(
                    result
                        .status_text
                        .unwrap_or_else(|| format!("Showcase event dispatched: {control_id}")),
                );
                if result.changed || result.refresh_projection {
                    self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA);
                }
            }
            Err(error) => {
                self.set_status_line(format!("Showcase event failed: {error}"));
            }
        }
    }
}
