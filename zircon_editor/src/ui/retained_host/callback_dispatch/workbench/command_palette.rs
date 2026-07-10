use crate::ui::binding::EditorUiEventKind;
use crate::ui::host::EditorHostEventController;
use crate::ui::retained_host::event_bridge::UiHostEventEffects;
use crate::ui::template_runtime::component_adapter::command::{
    editor_command_binding_for_envelope, COMMAND_DOMAIN,
};
use crate::ui::template_runtime::WORKBENCH_WINDOW_DOCUMENT_ID;
use zircon_runtime_interface::ui::component::{
    UiComponentBindingTarget, UiComponentEvent, UiComponentEventEnvelope, UiValue,
};

use super::super::{common::dispatch_editor_binding, BuiltinWorkbenchWindowTemplateSurfaceBridge};

pub(crate) const WORKBENCH_COMMAND_PALETTE_CONTROL_ID: &str = "WorkbenchCommandPalette";
pub(crate) const WORKBENCH_COMMAND_PALETTE_COMMIT_BINDING_ID: &str = "CommandPalette/Commit";

const COMMAND_PALETTE_COMPONENT_ID: &str = "CommandPalette";
const COMMITTED_COMMAND_ID: &str = "committed_command_id";

pub(crate) fn dispatch_componentized_workbench_command_palette_committed(
    runtime: &EditorHostEventController,
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    binding_id: &str,
    command_id: &str,
) -> Option<Result<UiHostEventEffects, String>> {
    if !is_command_palette_commit_route(bridge, control_id, binding_id) {
        return None;
    }

    let envelope = command_palette_commit_envelope(control_id, command_id);
    let binding = match editor_command_binding_for_envelope(&envelope) {
        Ok(binding) => binding,
        Err(error) => return Some(Err(error.to_string())),
    };
    Some(dispatch_editor_binding(runtime, binding))
}

fn is_command_palette_commit_route(
    bridge: &BuiltinWorkbenchWindowTemplateSurfaceBridge,
    control_id: &str,
    binding_id: &str,
) -> bool {
    if control_id != WORKBENCH_COMMAND_PALETTE_CONTROL_ID
        || binding_id != WORKBENCH_COMMAND_PALETTE_COMMIT_BINDING_ID
        || !bridge.has_control(control_id)
    {
        return false;
    }

    bridge
        .binding_by_id(WORKBENCH_COMMAND_PALETTE_COMMIT_BINDING_ID)
        .is_some_and(|binding| binding.path().event_kind == EditorUiEventKind::Submit)
}

fn command_palette_commit_envelope(control_id: &str, command_id: &str) -> UiComponentEventEnvelope {
    UiComponentEventEnvelope::new(
        WORKBENCH_WINDOW_DOCUMENT_ID,
        control_id,
        UiComponentBindingTarget::new(COMMAND_DOMAIN, COMMITTED_COMMAND_ID),
        UiComponentEvent::Commit {
            property: COMMITTED_COMMAND_ID.to_string(),
            value: UiValue::String(command_id.to_string()),
        },
    )
    .with_component_id(COMMAND_PALETTE_COMPONENT_ID)
}
