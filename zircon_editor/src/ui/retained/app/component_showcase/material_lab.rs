use super::super::*;
use crate::ui::template_runtime::builtin::MATERIAL_COMPONENT_LAB_WINDOW_DOCUMENT_ID;

impl RetainedEditorHost {
    pub(super) fn dispatch_material_lab_event(&mut self, control_id: &str, action_id: &str) {
        if let Err(error) = self.ensure_component_showcase_runtime_loaded() {
            self.set_status_line(error);
            return;
        }

        let binding = self
            .component_showcase_runtime
            .project_document(MATERIAL_COMPONENT_LAB_WINDOW_DOCUMENT_ID)
            .ok()
            .and_then(|projection| {
                projection
                    .bindings
                    .into_iter()
                    .find(|binding| binding.binding_id == action_id)
            });
        if binding.is_none() {
            self.set_status_line(format!("Unknown Material Lab action {action_id}"));
            return;
        }

        self.set_status_line(format!(
            "Material Lab feedback: {control_id} -> {}",
            action_id.replace('/', ".")
        ));
    }
}
