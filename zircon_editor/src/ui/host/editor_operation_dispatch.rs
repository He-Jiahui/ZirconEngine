use crate::core::asset::{AssetSourceAuthority, AssetTypeId, AssetWriteAccess};
use crate::core::commands::{
    AssetWriteTargetDescriptor, EditorCommandDispatchError, EditorCommandRegistry,
    EditorCommandRegistryError,
};
use crate::core::editor_event::{
    EditorEvent, EditorEventRecord, EditorEventSource, EditorOperationEvent,
};
use crate::core::editor_operation::{
    EditorOperationControlErrorKind, EditorOperationControlRequest, EditorOperationControlResponse,
    EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
};
use crate::ui::host::EditorHostEventController;
use serde_json::json;

impl EditorHostEventController {
    pub fn invoke_operation(
        &self,
        source: EditorOperationSource,
        invocation: EditorOperationInvocation,
    ) -> Result<EditorEventRecord, String> {
        let event_source = editor_event_source(source.clone());
        let descriptor = {
            self.commands()
                .lock()
                .command(invocation.operation_id.as_str())
                .cloned()
        };
        let descriptor = match descriptor {
            Some(descriptor) => descriptor,
            None => {
                let error =
                    EditorCommandRegistryError::MissingCommand(invocation.operation_id.clone())
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
        if operation_source_requires_remote_callable(&source) && !descriptor.callable_from_remote()
        {
            let error = EditorCommandRegistryError::CommandNotCallableFromRemote(
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
        let mut context = self.command_eval_ctx_for_source(&source);
        if let Some(target) = descriptor.asset_write_target() {
            let (authority, locator) = match self.resolve_asset_write_target(
                &invocation.operation_id,
                target,
                &invocation.arguments,
            ) {
                Ok(resolved) => resolved,
                Err(error) => {
                    return self.record_operation_control_failure(
                        event_source,
                        invocation.operation_id,
                        error,
                        invocation.arguments,
                        invocation.operation_group,
                    );
                }
            };
            context = context.with_asset_write_access(authority.write_access());
            if authority.write_access() != AssetWriteAccess::Writable {
                let error = format!(
                    "asset operation {} cannot write to read-only {} source `{locator}`",
                    invocation.operation_id,
                    authority.kind().as_str(),
                );
                return self.record_operation_control_failure(
                    event_source,
                    invocation.operation_id,
                    error,
                    invocation.arguments,
                    invocation.operation_group,
                );
            }
        }
        if let Err(error) = EditorCommandRegistry::ensure_enabled(&descriptor, &context) {
            return self.record_operation_control_failure(
                event_source,
                invocation.operation_id,
                error.to_string(),
                invocation.arguments,
                invocation.operation_group,
            );
        }
        let event = match descriptor.event().cloned() {
            Some(event) => event,
            None => {
                let error = EditorCommandDispatchError::EditCommandFactoryPending {
                    command_id: invocation.operation_id.clone(),
                }
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
                invocation.arguments,
                invocation.operation_group,
            )),
        )
    }

    fn resolve_asset_write_target(
        &self,
        operation_id: &EditorOperationPath,
        target: &AssetWriteTargetDescriptor,
        arguments: &serde_json::Value,
    ) -> Result<(AssetSourceAuthority, String), String> {
        let asset_type_value =
            string_argument(operation_id, arguments, target.asset_type_argument())?;
        let asset_type = AssetTypeId::parse(asset_type_value).map_err(|error| {
            format!(
                "asset operation {operation_id} has invalid asset type argument `{asset_type_value}`: {error}"
            )
        })?;
        let definition = self.asset_type_definition(&asset_type).ok_or_else(|| {
            format!(
                "asset operation {operation_id} references unregistered asset type `{asset_type}`"
            )
        })?;
        let locator = string_argument(operation_id, arguments, target.locator_argument())?;
        let authority =
            AssetSourceAuthority::from_target_str(definition.source_write_policy(), locator)
                .map_err(|error| {
                    format!(
                "asset operation {operation_id} has invalid source target `{locator}`: {error}"
            )
                })?;
        Ok((authority, locator.to_owned()))
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
                    let context = self.command_eval_ctx_for_source(&source);
                    let operation_state = self.commands().lock();
                    operation_state
                        .commands()
                        .filter(|descriptor| {
                            !operation_source_requires_remote_callable(&source)
                                || descriptor.callable_from_remote()
                        })
                        .filter(|descriptor| descriptor.is_enabled(&context))
                        .map(|descriptor| {
                            json!({
                                "operation_id": descriptor.id().as_str(),
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
                    "editor.operation.list",
                    Some(json!({ "operations": operations })),
                )
            }
            EditorOperationControlRequest::QueryOperationHistory => {
                EditorOperationControlResponse::typed_failure(
                    "editor.operation.history",
                    EditorOperationControlErrorKind::OperationHistoryPendingFactory,
                    "editor operation history is unavailable until the edit-command factory is installed",
                )
            }
        }
    }
}

fn string_argument<'a>(
    operation_id: &EditorOperationPath,
    arguments: &'a serde_json::Value,
    name: &str,
) -> Result<&'a str, String> {
    arguments
        .get(name)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            format!("asset operation {operation_id} requires non-empty string argument `{name}`")
        })
}

fn editor_event_source(source: EditorOperationSource) -> EditorEventSource {
    match source {
        EditorOperationSource::Menu | EditorOperationSource::UiBinding => {
            EditorEventSource::RetainedHost
        }
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
