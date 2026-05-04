use crate::core::editor_extension::ComponentDrawerDescriptor;
use crate::core::editor_operation::{
    EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
};
use crate::ui::host::EditorManager;
use crate::ui::workbench::state::EditorState;
use zircon_runtime_interface::ui::component::{
    UiComponentAdapterError, UiComponentAdapterResult, UiComponentEvent, UiComponentEventEnvelope,
};

pub(crate) fn validate_component_drawer_envelope(
    envelope: &UiComponentEventEnvelope,
    component_drawer: Option<&ComponentDrawerDescriptor>,
) -> Result<EditorOperationPath, UiComponentAdapterError> {
    if envelope.target.domain != "component_drawer" {
        return Err(UiComponentAdapterError::UnsupportedTargetDomain {
            domain: envelope.target.domain.clone(),
        });
    }
    if !is_safe_component_drawer_action(&envelope.event) {
        return Err(UiComponentAdapterError::UnsupportedEvent {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            event_kind: envelope.event_kind,
        });
    }
    let descriptor = component_drawer.ok_or_else(|| UiComponentAdapterError::MissingSource {
        domain: envelope.target.domain.clone(),
        path: envelope.target.path.clone(),
        source_name: "component_drawer".to_string(),
    })?;
    if !descriptor
        .bindings()
        .iter()
        .any(|binding| binding == &envelope.target.path)
    {
        return Err(UiComponentAdapterError::RejectedInput {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            reason: "operation is not declared by the enabled component drawer".to_string(),
        });
    }
    EditorOperationPath::parse(envelope.target.path.clone()).map_err(|error| {
        UiComponentAdapterError::RejectedInput {
            domain: envelope.target.domain.clone(),
            path: envelope.target.path.clone(),
            reason: error.to_string(),
        }
    })
}

fn is_safe_component_drawer_action(event: &UiComponentEvent) -> bool {
    matches!(
        event,
        UiComponentEvent::Press { pressed: true }
            | UiComponentEvent::Commit { .. }
            | UiComponentEvent::SelectOption { selected: true, .. }
            | UiComponentEvent::ToggleExpanded { .. }
            | UiComponentEvent::OpenReference { .. }
            | UiComponentEvent::LocateReference { .. }
    )
}

pub(crate) fn component_drawer_operation_result(
    operation_path: &EditorOperationPath,
) -> UiComponentAdapterResult {
    UiComponentAdapterResult::changed()
        .dirty(false)
        .with_mutation_source("component_drawer")
        .with_transaction(format!("component_drawer:{operation_path}"))
}

#[allow(dead_code)]
pub(crate) fn apply_component_drawer_envelope(
    _state: &mut EditorState,
    _manager: &EditorManager,
    envelope: &UiComponentEventEnvelope,
    component_drawer: Option<&ComponentDrawerDescriptor>,
) -> Result<UiComponentAdapterResult, UiComponentAdapterError> {
    let operation_path = validate_component_drawer_envelope(envelope, component_drawer)?;
    Ok(component_drawer_operation_result(&operation_path))
}

pub(crate) fn component_drawer_operation_invocation(
    operation_path: EditorOperationPath,
) -> (EditorOperationSource, EditorOperationInvocation) {
    (
        EditorOperationSource::UiBinding,
        EditorOperationInvocation::new(operation_path),
    )
}
