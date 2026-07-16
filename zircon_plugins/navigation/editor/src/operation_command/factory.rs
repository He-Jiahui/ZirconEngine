use zircon_editor::core::editing::engine::HistoryContextId;
use zircon_editor::core::editing::operation::{
    OperationCommand, OperationCommandFactory, OperationCommandFactoryError,
};
use zircon_editor::core::editor_operation::{EditorOperationInvocation, EditorOperationPath};
use zircon_runtime::core::framework::navigation::{
    NavMeshBakeRequest, NavigationClearBakeRequest, NAVIGATION_BAKE_SCENE_OPERATION,
    NAVIGATION_BAKE_SURFACE_OPERATION, NAVIGATION_CLEAR_SURFACE_OPERATION,
};
use zircon_runtime_interface::{ZrRuntimeOperationSubmitRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::NavigationOperationCommand;

pub(crate) struct NavigationOperationCommandFactory {
    operation: EditorOperationPath,
}

impl NavigationOperationCommandFactory {
    pub(crate) fn for_operation(
        operation: &EditorOperationPath,
    ) -> Result<Self, OperationCommandFactoryError> {
        if !matches!(
            operation.as_str(),
            NAVIGATION_BAKE_SCENE_OPERATION
                | NAVIGATION_BAKE_SURFACE_OPERATION
                | NAVIGATION_CLEAR_SURFACE_OPERATION
        ) {
            return Err(OperationCommandFactoryError::Factory {
                operation: operation.clone(),
                reason: "unsupported navigation runtime operation".to_string(),
            });
        }
        Ok(Self {
            operation: operation.clone(),
        })
    }

    fn runtime_request(
        &self,
        arguments: serde_json::Value,
    ) -> Result<ZrRuntimeOperationSubmitRequestV1, OperationCommandFactoryError> {
        let payload = match self.operation.as_str() {
            NAVIGATION_BAKE_SCENE_OPERATION => {
                let mut request: NavMeshBakeRequest =
                    decode_arguments(&self.operation, arguments, NavMeshBakeRequest::default())?;
                request.surface_entity = None;
                serde_json::to_value(request)
            }
            NAVIGATION_BAKE_SURFACE_OPERATION => {
                let request = decode_bake_surface_arguments(&self.operation, arguments)?;
                if request.surface_entity.is_none() {
                    return Err(OperationCommandFactoryError::InvalidArguments {
                        operation: self.operation.clone(),
                        reason: "surface_entity is required".to_string(),
                    });
                }
                serde_json::to_value(request)
            }
            NAVIGATION_CLEAR_SURFACE_OPERATION => {
                let request = decode_clear_surface_arguments(&self.operation, arguments)?;
                if request.surface_entity.is_none() {
                    return Err(OperationCommandFactoryError::InvalidArguments {
                        operation: self.operation.clone(),
                        reason: "surface_entity is required".to_string(),
                    });
                }
                serde_json::to_value(request)
            }
            _ => {
                return Err(OperationCommandFactoryError::Factory {
                    operation: self.operation.clone(),
                    reason: "unsupported navigation runtime operation".to_string(),
                });
            }
        }
        .map_err(|error| OperationCommandFactoryError::Factory {
            operation: self.operation.clone(),
            reason: error.to_string(),
        })?;
        Ok(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            self.operation.to_string(),
            payload,
        ))
    }
}

fn decode_bake_surface_arguments(
    operation: &EditorOperationPath,
    arguments: serde_json::Value,
) -> Result<NavMeshBakeRequest, OperationCommandFactoryError> {
    match arguments {
        serde_json::Value::Array(values) => {
            let [surface_entity, force_full_rebuild]: [serde_json::Value; 2] =
                values.try_into().map_err(|_| {
                    invalid_arguments(operation, "expected [surface_entity, force_full_rebuild]")
                })?;
            let surface_entity = surface_entity.as_u64().ok_or_else(|| {
                invalid_arguments(operation, "surface_entity must be an unsigned integer")
            })?;
            let force_full_rebuild = force_full_rebuild.as_bool().ok_or_else(|| {
                invalid_arguments(operation, "force_full_rebuild must be a boolean")
            })?;
            Ok(NavMeshBakeRequest {
                surface_entity: Some(surface_entity),
                force_full_rebuild,
                ..NavMeshBakeRequest::default()
            })
        }
        arguments => decode_arguments(operation, arguments, NavMeshBakeRequest::default()),
    }
}

fn decode_clear_surface_arguments(
    operation: &EditorOperationPath,
    arguments: serde_json::Value,
) -> Result<NavigationClearBakeRequest, OperationCommandFactoryError> {
    match arguments {
        serde_json::Value::Array(values) => {
            let [surface_entity]: [serde_json::Value; 1] = values
                .try_into()
                .map_err(|_| invalid_arguments(operation, "expected [surface_entity]"))?;
            let surface_entity = surface_entity.as_u64().ok_or_else(|| {
                invalid_arguments(operation, "surface_entity must be an unsigned integer")
            })?;
            Ok(NavigationClearBakeRequest {
                surface_entity: Some(surface_entity),
            })
        }
        arguments => decode_arguments(operation, arguments, NavigationClearBakeRequest::default()),
    }
}

fn invalid_arguments(
    operation: &EditorOperationPath,
    reason: impl Into<String>,
) -> OperationCommandFactoryError {
    OperationCommandFactoryError::InvalidArguments {
        operation: operation.clone(),
        reason: reason.into(),
    }
}

impl OperationCommandFactory for NavigationOperationCommandFactory {
    fn create(
        &self,
        invocation: &EditorOperationInvocation,
    ) -> Result<OperationCommand, OperationCommandFactoryError> {
        if invocation.operation_id != self.operation {
            return Err(OperationCommandFactoryError::OperationMismatch {
                descriptor_operation: invocation.operation_id.clone(),
                factory_operation: self.operation.clone(),
            });
        }
        let request = self.runtime_request(invocation.arguments.clone())?;
        Ok(OperationCommand::new(
            Box::new(NavigationOperationCommand::new(request)),
            HistoryContextId::Global,
        ))
    }
}

fn decode_arguments<T: serde::de::DeserializeOwned>(
    operation: &EditorOperationPath,
    arguments: serde_json::Value,
    default: T,
) -> Result<T, OperationCommandFactoryError> {
    if arguments.is_null() {
        return Ok(default);
    }
    serde_json::from_value(arguments).map_err(|error| {
        OperationCommandFactoryError::InvalidArguments {
            operation: operation.clone(),
            reason: error.to_string(),
        }
    })
}
