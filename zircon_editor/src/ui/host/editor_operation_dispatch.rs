use std::collections::BTreeSet;

use crate::core::editor_event::{
    EditorEvent, EditorEventRecord, EditorEventRuntime, EditorEventSource, EditorOperationEvent,
};
use crate::core::editor_operation::{
    EditorOperationControlRequest, EditorOperationControlResponse, EditorOperationDescriptor,
    EditorOperationInvocation, EditorOperationPath, EditorOperationRegistryError,
    EditorOperationSource,
};
use serde_json::json;

impl EditorEventRuntime {
    pub fn invoke_operation(
        &self,
        source: EditorOperationSource,
        invocation: EditorOperationInvocation,
    ) -> Result<EditorEventRecord, String> {
        let event_source = editor_event_source(source.clone());
        let descriptor = {
            let inner = self.inner.lock().unwrap();
            let descriptor = match inner
                .operation_registry
                .descriptor(&invocation.operation_id)
                .cloned()
            {
                Some(descriptor) => descriptor,
                None => {
                    let error = EditorOperationRegistryError::MissingOperation(
                        invocation.operation_id.clone(),
                    )
                    .to_string();
                    drop(inner);
                    return self.record_operation_control_failure(
                        event_source,
                        invocation.operation_id,
                        error,
                        invocation.arguments,
                        invocation.operation_group,
                    );
                }
            };
            if operation_source_requires_remote_callable(&source)
                && !descriptor.callable_from_remote()
            {
                let error = EditorOperationRegistryError::OperationNotCallableFromRemote(
                    invocation.operation_id.clone(),
                )
                .to_string();
                drop(inner);
                return self.record_operation_control_failure(
                    event_source,
                    invocation.operation_id,
                    error,
                    invocation.arguments,
                    invocation.operation_group,
                );
            }
            if let Some(error) = operation_capability_error(
                &descriptor,
                inner.manager.capability_snapshot().enabled_capabilities(),
            ) {
                drop(inner);
                return self.record_operation_control_failure(
                    event_source,
                    invocation.operation_id,
                    error,
                    invocation.arguments,
                    invocation.operation_group,
                );
            }
            descriptor
        };
        let event = match descriptor.event().cloned() {
            Some(event) => event,
            None => {
                let error = EditorOperationRegistryError::OperationHasNoHandler(
                    invocation.operation_id.clone(),
                )
                .to_string();
                return self.record_operation_control_failure(
                    event_source,
                    invocation.operation_id,
                    error,
                    invocation.arguments,
                    invocation.operation_group,
                );
            }
        };

        self.dispatch_normalized_event_with_operation(
            event_source,
            event,
            Some((
                invocation.operation_id,
                descriptor.display_name().to_string(),
                descriptor.undoable().is_some(),
                invocation.arguments,
                invocation.operation_group,
            )),
        )
    }

    fn record_operation_control_failure(
        &self,
        source: EditorEventSource,
        operation_id: EditorOperationPath,
        error: String,
        arguments: serde_json::Value,
        operation_group: Option<String>,
    ) -> Result<EditorEventRecord, String> {
        self.dispatch_normalized_event_with_operation(
            source,
            EditorEvent::Operation(EditorOperationEvent::ControlFailure {
                operation_id: operation_id.to_string(),
                error,
            }),
            Some((
                operation_id.clone(),
                operation_id.to_string(),
                false,
                arguments,
                operation_group,
            )),
        )
    }

    pub fn handle_operation_control_request(
        &self,
        request: EditorOperationControlRequest,
    ) -> EditorOperationControlResponse {
        self.handle_operation_control_request_from_source(EditorOperationSource::Remote, request)
    }

    pub fn handle_operation_control_request_from_source(
        &self,
        source: EditorOperationSource,
        request: EditorOperationControlRequest,
    ) -> EditorOperationControlResponse {
        match request {
            EditorOperationControlRequest::InvokeOperation(invocation) => {
                let operation_id = invocation.operation_id.to_string();
                match self.invoke_operation(source, invocation) {
                    Ok(record) => {
                        EditorOperationControlResponse::success(operation_id, record.result.value)
                    }
                    Err(error) => EditorOperationControlResponse::failure(operation_id, error),
                }
            }
            EditorOperationControlRequest::ListOperations => {
                let operations = {
                    let inner = self.inner.lock().unwrap();
                    let enabled_capabilities = inner
                        .manager
                        .capability_snapshot()
                        .enabled_capabilities()
                        .to_vec();
                    inner
                        .operation_registry
                        .descriptors()
                        .filter(|descriptor| {
                            operation_capability_error(descriptor, &enabled_capabilities).is_none()
                        })
                        .map(|descriptor| {
                            json!({
                                "operation_id": descriptor.path().as_str(),
                                "display_name": descriptor.display_name(),
                                "menu_path": descriptor.menu_path(),
                                "callable_from_remote": descriptor.callable_from_remote(),
                                "undoable": descriptor.undoable().is_some(),
                                "undo_display_name": descriptor
                                    .undoable()
                                    .map(|operation| operation.display_name()),
                                "required_capabilities": descriptor.required_capabilities(),
                            })
                        })
                        .collect::<Vec<_>>()
                };
                EditorOperationControlResponse::success(
                    "Editor.Operation.List",
                    Some(json!({ "operations": operations })),
                )
            }
            EditorOperationControlRequest::QueryOperationStack => {
                let stack = self.operation_stack();
                EditorOperationControlResponse::success(
                    "Editor.Operation.Stack",
                    Some(json!({
                        "undo_stack": stack_entries(stack.undo_stack()),
                        "redo_stack": stack_entries(stack.redo_stack()),
                    })),
                )
            }
        }
    }
}

fn stack_entries(
    entries: &[crate::core::editor_operation::EditorOperationStackEntry],
) -> Vec<serde_json::Value> {
    entries
        .iter()
        .map(|entry| {
            json!({
                "operation_id": entry.operation_id.as_str(),
                "display_name": &entry.display_name,
                "source": &entry.source,
                "sequence": entry.sequence,
                "operation_group": &entry.operation_group,
            })
        })
        .collect()
}

fn operation_capability_error<I, S>(
    descriptor: &EditorOperationDescriptor,
    enabled_capabilities: I,
) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let enabled = enabled_capabilities
        .into_iter()
        .map(|capability| capability.as_ref().to_string())
        .collect::<BTreeSet<_>>();
    let missing = descriptor
        .required_capabilities()
        .iter()
        .filter(|capability| !enabled.contains(*capability))
        .cloned()
        .collect::<Vec<_>>();
    (!missing.is_empty()).then(|| {
        format!(
            "editor operation {} requires disabled capabilities: {}",
            descriptor.path(),
            missing.join(", ")
        )
    })
}

fn editor_event_source(source: EditorOperationSource) -> EditorEventSource {
    match source {
        EditorOperationSource::Menu | EditorOperationSource::UiBinding => EditorEventSource::Slint,
        EditorOperationSource::Remote => EditorEventSource::Headless,
        EditorOperationSource::Cli => EditorEventSource::Cli,
    }
}

fn operation_source_requires_remote_callable(source: &EditorOperationSource) -> bool {
    matches!(
        source,
        EditorOperationSource::Remote | EditorOperationSource::Cli
    )
}
