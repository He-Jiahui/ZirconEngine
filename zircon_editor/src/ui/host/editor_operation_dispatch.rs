use crate::core::asset::{AssetSourceAuthority, AssetTypeId, AssetTypeIdError, AssetWriteAccess};
use crate::core::commands::{
    AssetWriteTargetDescriptor, EditorCommandDescriptor, EditorCommandDispatchError,
    EditorCommandRegistry, EditorCommandRegistryError,
};
use crate::core::editing::engine::{EditCommandError, HistoryContextId};
use crate::core::editing::operation::OperationCommandFactoryError;
use crate::core::editor_event::{
    EditorEvent, EditorEventRecord, EditorEventResult, EditorEventSource, EditorOperationEvent,
};
use crate::core::editor_extension::EditorExtensionRegistryError;
use crate::core::editor_operation::{
    EditorOperationControlRequest, EditorOperationControlResponse, EditorOperationInvocation,
    EditorOperationPath, EditorOperationSource,
};
use crate::core::play::{PlayEditRoute, PlayEditRouteError};
use crate::ui::host::EditorEventDispatchError;
use crate::ui::host::EditorHostEventController;
use serde_json::json;
use thiserror::Error;
use zircon_runtime::plugin::native::ZIRCON_NATIVE_PLUGIN_STATUS_OK;
use zircon_runtime_interface::resource::ResourceLocatorError;

#[derive(Debug, Error)]
pub enum EditorOperationDispatchError {
    #[error(transparent)]
    ExtensionRegistry(#[from] EditorExtensionRegistryError),
    #[error(transparent)]
    Registry(#[from] EditorCommandRegistryError),
    #[error(transparent)]
    Command(#[from] EditorCommandDispatchError),
    #[error(transparent)]
    Factory(#[from] OperationCommandFactoryError),
    #[error(transparent)]
    Transaction(#[from] EditCommandError),
    #[error(transparent)]
    PlayEditRoute(#[from] PlayEditRouteError),
    #[error("asset operation {operation} requires non-empty string argument `{argument}`")]
    MissingAssetArgument {
        operation: EditorOperationPath,
        argument: String,
    },
    #[error("asset operation {operation} has invalid asset type argument `{value}`: {source}")]
    InvalidAssetType {
        operation: EditorOperationPath,
        value: String,
        #[source]
        source: AssetTypeIdError,
    },
    #[error("asset operation {operation} references unregistered asset type `{asset_type}`")]
    UnregisteredAssetType {
        operation: EditorOperationPath,
        asset_type: AssetTypeId,
    },
    #[error("asset operation {operation} has invalid source target `{locator}`: {source}")]
    InvalidAssetSource {
        operation: EditorOperationPath,
        locator: String,
        #[source]
        source: ResourceLocatorError,
    },
    #[error(
        "asset operation {operation} cannot write to read-only {source_kind} source `{locator}`"
    )]
    ReadOnlyAssetSource {
        operation: EditorOperationPath,
        source_kind: &'static str,
        locator: String,
    },
    #[error(transparent)]
    EventDispatch(#[from] EditorEventDispatchError),
    #[error("native editor command {operation} arguments could not be encoded: {detail}")]
    NativeInputEncoding {
        operation: EditorOperationPath,
        detail: String,
    },
    #[error("native editor command {operation} was rejected: {detail}")]
    NativeInvocationRejected {
        operation: EditorOperationPath,
        detail: String,
    },
    #[error("native editor command {operation} returned a result that could not be decoded as {codec}: {detail}")]
    NativeResultDecoding {
        operation: EditorOperationPath,
        codec: String,
        detail: String,
    },
    #[error("native editor command {operation} returned no result payload")]
    NativeResultMissing { operation: EditorOperationPath },
}

impl EditorHostEventController {
    pub fn invoke_operation(
        &self,
        source: EditorOperationSource,
        invocation: EditorOperationInvocation,
    ) -> Result<EditorEventRecord, EditorOperationDispatchError> {
        self.invoke_operation_with_binding_path(source, invocation, None)
    }

    pub(crate) fn invoke_operation_with_binding_path(
        &self,
        source: EditorOperationSource,
        invocation: EditorOperationInvocation,
        binding_path: Option<String>,
    ) -> Result<EditorEventRecord, EditorOperationDispatchError> {
        let event_source = editor_event_source(source.clone());
        let (descriptor, operation_factory) = {
            let commands = self.commands().lock();
            (
                commands.command(invocation.operation_id.as_str()).cloned(),
                commands
                    .operation_factory(&invocation.operation_id)
                    .cloned(),
            )
        };
        let descriptor = match descriptor {
            Some(descriptor) => descriptor,
            None => {
                let error = EditorOperationDispatchError::from(
                    EditorCommandRegistryError::MissingCommand(invocation.operation_id.clone()),
                );
                return self.record_operation_control_failure(
                    event_source,
                    invocation.operation_id,
                    error,
                    invocation.arguments,
                    invocation.operation_group,
                    binding_path,
                );
            }
        };
        let i18n = self.context().i18n();
        let locale = i18n.active_locale();
        let operation_label = descriptor.localized_label(i18n, &locale);
        if operation_source_requires_remote_callable(&source) && !descriptor.callable_from_remote()
        {
            let error = EditorOperationDispatchError::from(
                EditorCommandRegistryError::CommandNotCallableFromRemote(
                    invocation.operation_id.clone(),
                ),
            );
            return self.record_operation_control_failure(
                event_source,
                invocation.operation_id,
                error,
                invocation.arguments,
                invocation.operation_group,
                binding_path,
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
                        binding_path,
                    );
                }
            };
            context = context.with_asset_write_access(authority.write_access());
            if authority.write_access() != AssetWriteAccess::Writable {
                let error = EditorOperationDispatchError::ReadOnlyAssetSource {
                    operation: invocation.operation_id.clone(),
                    source_kind: authority.kind().as_str(),
                    locator,
                };
                return self.record_operation_control_failure(
                    event_source,
                    invocation.operation_id,
                    error,
                    invocation.arguments,
                    invocation.operation_group,
                    binding_path,
                );
            }
        }
        if let Err(error) = EditorCommandRegistry::ensure_enabled(&descriptor, &context) {
            return self.record_operation_control_failure(
                event_source,
                invocation.operation_id,
                error.into(),
                invocation.arguments,
                invocation.operation_group,
                binding_path,
            );
        }
        if matches!(
            descriptor.action(),
            crate::core::commands::EditorCommandAction::NativeEndpoint
        ) {
            return self.invoke_native_command(
                source,
                invocation,
                descriptor,
                operation_label,
                binding_path,
            );
        }
        if let Some(event) = descriptor.event().cloned() {
            return self
                .dispatch_normalized_event_with_operation(
                    event_source,
                    event,
                    Some((
                        invocation.operation_id,
                        operation_label.to_string(),
                        invocation.arguments,
                        invocation.operation_group,
                    )),
                    binding_path,
                )
                .map_err(EditorOperationDispatchError::from);
        }

        let Some(operation_factory) = operation_factory else {
            let error =
                EditorOperationDispatchError::from(OperationCommandFactoryError::MissingFactory {
                    operation: invocation.operation_id.clone(),
                });
            return self.record_operation_control_failure(
                event_source,
                invocation.operation_id,
                error,
                invocation.arguments,
                invocation.operation_group,
                binding_path,
            );
        };
        let deferred = match operation_factory.defer(invocation.clone()) {
            Ok(deferred) => deferred,
            Err(error) => {
                return self.record_operation_control_failure(
                    event_source,
                    invocation.operation_id,
                    error.into(),
                    invocation.arguments,
                    invocation.operation_group,
                    binding_path,
                );
            }
        };
        let route = match self
            .play_sessions()
            .route_edit(operation_factory.edit_target(), deferred)
        {
            Ok(route) => route,
            Err(error) => {
                return self.record_operation_control_failure(
                    event_source,
                    invocation.operation_id,
                    error.into(),
                    invocation.arguments,
                    invocation.operation_group,
                    binding_path,
                );
            }
        };
        let invocation = match route {
            PlayEditRoute::ApplyNow { invocation, .. } => invocation,
            PlayEditRoute::Queued {
                id,
                coalesced,
                evicted_ids,
            } => {
                let operation_id = invocation.operation_id;
                return self
                    .dispatch_normalized_event_with_operation(
                        event_source,
                        EditorEvent::Operation(EditorOperationEvent::EditQueued {
                            operation_id: operation_id.to_string(),
                            pending_edit_id: id.value(),
                            coalesced,
                            evicted_pending_edit_ids: evicted_ids
                                .into_iter()
                                .map(|id| id.value())
                                .collect(),
                        }),
                        Some((
                            operation_id,
                            operation_label.to_string(),
                            invocation.arguments,
                            invocation.operation_group,
                        )),
                        binding_path,
                    )
                    .map_err(EditorOperationDispatchError::from);
            }
        };
        let operation = match operation_factory.create(&invocation) {
            Ok(operation) => operation,
            Err(error) => {
                return self.record_operation_control_failure(
                    event_source,
                    invocation.operation_id,
                    error.into(),
                    invocation.arguments,
                    invocation.operation_group,
                    binding_path,
                );
            }
        };
        let (command, history, merge_mode) = operation.into_parts();
        let execution = match self.context().transactions().execute_operation(
            operation_label.as_ref(),
            history,
            invocation.operation_group.as_deref(),
            merge_mode,
            command,
        ) {
            Ok(execution) => execution,
            Err(error) => {
                return self.record_operation_control_failure(
                    event_source,
                    invocation.operation_id,
                    error.into(),
                    invocation.arguments,
                    invocation.operation_group,
                    binding_path,
                );
            }
        };
        let operation_id = invocation.operation_id;

        self.dispatch_normalized_event_with_operation(
            event_source,
            EditorEvent::Operation(EditorOperationEvent::CommandExecuted {
                operation_id: operation_id.to_string(),
                transaction_id: execution.transaction_id.raw(),
                group_open: execution.group_open,
            }),
            Some((
                operation_id,
                operation_label.to_string(),
                invocation.arguments,
                invocation.operation_group,
            )),
            binding_path,
        )
        .map_err(EditorOperationDispatchError::from)
    }

    fn invoke_native_command(
        &self,
        source: EditorOperationSource,
        invocation: EditorOperationInvocation,
        descriptor: EditorCommandDescriptor,
        operation_label: std::sync::Arc<str>,
        binding_path: Option<String>,
    ) -> Result<EditorEventRecord, EditorOperationDispatchError> {
        let operation_id = invocation.operation_id.clone();
        let arguments = invocation.arguments.clone();
        let operation_group = invocation.operation_group.clone();
        let payload = match serde_json::to_vec(&arguments) {
            Ok(payload) => payload,
            Err(error) => {
                return self.record_operation_control_failure(
                    editor_event_source(source.clone()),
                    operation_id,
                    EditorOperationDispatchError::NativeInputEncoding {
                        operation: invocation.operation_id.clone(),
                        detail: error.to_string(),
                    },
                    invocation.arguments,
                    invocation.operation_group,
                    binding_path,
                )
            }
        };
        let receipt = match self
            .commands()
            .lock()
            .invoke_native_executor(&operation_id, &payload)
        {
            Ok(receipt) => receipt,
            Err(error) => {
                return self.record_operation_control_failure(
                    editor_event_source(source.clone()),
                    operation_id,
                    error.into(),
                    invocation.arguments,
                    invocation.operation_group,
                    binding_path,
                )
            }
        };
        if receipt.status_code() != ZIRCON_NATIVE_PLUGIN_STATUS_OK {
            let diagnostics = receipt.diagnostics().join("; ");
            let detail = if diagnostics.is_empty() {
                format!("native endpoint returned status {}", receipt.status_code())
            } else {
                diagnostics
            };
            return self.record_operation_control_failure(
                editor_event_source(source),
                operation_id,
                EditorOperationDispatchError::NativeInvocationRejected {
                    operation: invocation.operation_id,
                    detail,
                },
                invocation.arguments,
                invocation.operation_group,
                binding_path,
            );
        }
        let result = match decode_native_command_result(&descriptor, receipt.payload()) {
            Ok(result) => result,
            Err(error) => {
                return self.record_operation_control_failure(
                    editor_event_source(source.clone()),
                    operation_id,
                    error,
                    invocation.arguments,
                    invocation.operation_group,
                    binding_path,
                )
            }
        };
        self.dispatch_normalized_native_result(
            editor_event_source(source),
            EditorEvent::Operation(EditorOperationEvent::NativeCommandExecuted {
                operation_id: operation_id.to_string(),
                status_code: receipt.status_code(),
            }),
            Some((
                operation_id,
                operation_label.to_string(),
                arguments,
                operation_group,
            )),
            binding_path,
            EditorEventResult::success(result),
        )
        .map_err(EditorOperationDispatchError::from)
    }

    fn resolve_asset_write_target(
        &self,
        operation_id: &EditorOperationPath,
        target: &AssetWriteTargetDescriptor,
        arguments: &serde_json::Value,
    ) -> Result<(AssetSourceAuthority, String), EditorOperationDispatchError> {
        let asset_type_value =
            string_argument(operation_id, arguments, target.asset_type_argument())?;
        let asset_type = AssetTypeId::parse(asset_type_value).map_err(|source| {
            EditorOperationDispatchError::InvalidAssetType {
                operation: operation_id.clone(),
                value: asset_type_value.to_string(),
                source,
            }
        })?;
        let definition = self.asset_type_definition(&asset_type)?.ok_or_else(|| {
            EditorOperationDispatchError::UnregisteredAssetType {
                operation: operation_id.clone(),
                asset_type: asset_type.clone(),
            }
        })?;
        let locator = string_argument(operation_id, arguments, target.locator_argument())?;
        let authority =
            AssetSourceAuthority::from_target_str(definition.source_write_policy(), locator)
                .map_err(|source| EditorOperationDispatchError::InvalidAssetSource {
                    operation: operation_id.clone(),
                    locator: locator.to_string(),
                    source,
                })?;
        Ok((authority, locator.to_owned()))
    }

    fn record_operation_control_failure(
        &self,
        source: EditorEventSource,
        operation_id: EditorOperationPath,
        error: EditorOperationDispatchError,
        arguments: serde_json::Value,
        operation_group: Option<String>,
        binding_path: Option<String>,
    ) -> Result<EditorEventRecord, EditorOperationDispatchError> {
        let error_message = error.to_string();
        let _ = self.dispatch_normalized_event_with_operation(
            source,
            EditorEvent::Operation(EditorOperationEvent::ControlFailure {
                operation_id: operation_id.to_string(),
                error: error_message,
            }),
            Some((
                operation_id.clone(),
                operation_id.to_string(),
                arguments,
                operation_group,
            )),
            binding_path,
        );

        // The ControlFailure event is journaled before its executor returns an error. Keep the
        // original error typed for every caller instead of reparsing its presentation text.
        Err(error)
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
                    Err(error) => {
                        EditorOperationControlResponse::failure(operation_id, error.to_string())
                    }
                }
            }
            EditorOperationControlRequest::ListOperations => {
                let i18n = self.context().i18n();
                let locale = i18n.active_locale();
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
                            let factory = operation_state.operation_factory(descriptor.id());
                            json!({
                                "operation_id": descriptor.id().as_str(),
                                "label": descriptor.localized_label(i18n, &locale),
                                "label_key": descriptor.presentation().label_key(),
                                "menu_path": descriptor.menu_path(),
                                "callable_from_remote": descriptor.callable_from_remote(),
                                "undoable": factory.is_some(),
                                "undo_display_name": factory.map(|factory| factory.undo_display_name()),
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
                match self.context().transactions().history_details(
                    HistoryContextId::Global,
                    None,
                    128,
                ) {
                    Ok(page) => {
                        let history = page.status();
                        let records = page.into_records();
                        EditorOperationControlResponse::success(
                            "editor.operation.history",
                            Some(json!({
                                "history": "global",
                                "len": history.len,
                                "top": history.top.map(|transaction| transaction.raw()),
                                "saved_top": history.saved_top.map(|transaction| transaction.raw()),
                                "saved_top_reachable": history.saved_top_reachable,
                                "can_undo": history.can_undo,
                                "can_redo": history.can_redo,
                                "dirty": history.dirty,
                                "generation": history.generation,
                                "records": records.into_iter().map(|record| json!({
                                    "transaction_id": record.id.raw(),
                                    "label": record.label,
                                    "timestamp_frame": record.timestamp_frame,
                                    "command_count": record.command_count,
                                    "participants": record.participants.into_iter().map(|document| document.value()).collect::<Vec<_>>(),
                                    "significant": record.significant,
                                })).collect::<Vec<_>>(),
                            })),
                        )
                    }
                    Err(error) => EditorOperationControlResponse::failure(
                        "editor.operation.history",
                        error.to_string(),
                    ),
                }
            }
        }
    }
}

fn decode_native_command_result(
    descriptor: &EditorCommandDescriptor,
    payload: Option<&[u8]>,
) -> Result<serde_json::Value, EditorOperationDispatchError> {
    let contract = descriptor.execution_contract().ok_or_else(|| {
        EditorOperationDispatchError::NativeResultDecoding {
            operation: descriptor.id().clone(),
            codec: "unknown".to_owned(),
            detail: "native endpoint has no execution contract".to_owned(),
        }
    })?;
    let codec = contract.result_codec().to_string();
    let Some(payload) = payload else {
        if contract.resource_budget().max_output_bytes() == 0 {
            return Ok(serde_json::Value::Null);
        }
        return Err(EditorOperationDispatchError::NativeResultMissing {
            operation: descriptor.id().clone(),
        });
    };
    if codec != "zircon.editor.command-result.v1" {
        return Err(EditorOperationDispatchError::NativeResultDecoding {
            operation: descriptor.id().clone(),
            codec,
            detail: "no host decoder is registered for this result codec".to_owned(),
        });
    }
    serde_json::from_slice(payload).map_err(|error| {
        EditorOperationDispatchError::NativeResultDecoding {
            operation: descriptor.id().clone(),
            codec,
            detail: error.to_string(),
        }
    })
}

fn string_argument<'a>(
    operation_id: &EditorOperationPath,
    arguments: &'a serde_json::Value,
    name: &str,
) -> Result<&'a str, EditorOperationDispatchError> {
    arguments
        .get(name)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| EditorOperationDispatchError::MissingAssetArgument {
            operation: operation_id.clone(),
            argument: name.to_string(),
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
