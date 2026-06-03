use zircon_runtime_interface::ui::{binding::UiEventKind, component::UiValue};

use crate::ui::binding::EditorUiBindingPayload;

use super::{
    componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge,
    error::BuiltinHostWindowTemplateBridgeError,
};

const WORKBENCH_MODULE_BINDING_PREFIX: &str = "WorkbenchModule/";
const EDIT_ACTION_PREFIX: &str = "EditWorkbench";
const COMMIT_ACTION_PREFIX: &str = "CommitWorkbench";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn edit_workbench_module_field(
        &mut self,
        control_id: &str,
        binding_id: &str,
        value: &str,
    ) -> Result<Option<bool>, BuiltinHostWindowTemplateBridgeError> {
        if !is_workbench_module_field_binding(self, binding_id) {
            return Ok(None);
        }
        if control_id.trim().is_empty() || !self.has_control(control_id) {
            return Ok(Some(false));
        }
        if !self.module_field_control_owns_binding(control_id, binding_id) {
            return Ok(Some(false));
        }

        let raw_value = value.to_string();
        self.mutate_control_property(control_id, "value", UiValue::String(raw_value.clone()))?;
        self.mutate_control_property(control_id, "value_text", UiValue::String(raw_value))?;
        self.template_surface
            .refresh_after_state_change(self.runtime.as_ref())?;
        Ok(Some(true))
    }

    fn module_field_control_owns_binding(&self, control_id: &str, binding_id: &str) -> bool {
        self.host_projection()
            .node_by_control_id(control_id)
            .is_some_and(|node| {
                node.routes.iter().any(|route| {
                    route.binding_id == binding_id
                        && matches!(route.event_kind, UiEventKind::Change | UiEventKind::Submit)
                })
            })
    }
}

fn is_workbench_module_field_binding(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    binding_id: &str,
) -> bool {
    if !binding_id.starts_with(WORKBENCH_MODULE_BINDING_PREFIX) {
        return false;
    }

    bridge.binding_by_id(binding_id).is_some_and(|binding| {
        matches!(
            binding.payload(),
            EditorUiBindingPayload::MenuAction { action_id }
                if action_id.starts_with(EDIT_ACTION_PREFIX)
                    || action_id.starts_with(COMMIT_ACTION_PREFIX)
        )
    })
}
