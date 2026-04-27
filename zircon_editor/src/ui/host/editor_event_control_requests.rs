use crate::core::editor_event::{EditorEventRuntime, EditorEventSource};
use crate::core::editor_operation::{
    EditorOperationInvocation, EditorOperationPath, EditorOperationSource,
};
use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload};
use zircon_runtime::ui::{
    binding::UiEventBinding, event_ui::UiControlRequest, event_ui::UiControlResponse,
    event_ui::UiInvocationError, event_ui::UiInvocationResult, event_ui::UiNodePath,
    event_ui::UiRouteId,
};

impl EditorEventRuntime {
    pub fn handle_control_request(&self, request: UiControlRequest) -> UiControlResponse {
        match request {
            UiControlRequest::InvokeBinding { binding } => {
                UiControlResponse::Invocation(self.invoke_binding(binding))
            }
            UiControlRequest::InvokeRoute {
                route_id,
                arguments,
            } => UiControlResponse::Invocation(self.invoke_route(route_id, arguments)),
            UiControlRequest::CallAction {
                node_path,
                action_id,
                arguments,
            } => UiControlResponse::Invocation(self.call_action(node_path, action_id, arguments)),
            other => {
                let mut inner = self.inner.lock().unwrap();
                inner.control_service.handle_request(other)
            }
        }
    }

    fn invoke_binding(&self, binding: UiEventBinding) -> UiInvocationResult {
        let route_id = {
            let inner = self.inner.lock().unwrap();
            inner.control_service.route_id_for_binding(&binding)
        };
        let editor_binding = match EditorUiBinding::from_ui_binding(binding.clone()) {
            Ok(binding) => binding,
            Err(error) => {
                return UiInvocationResult {
                    route_id,
                    binding: Some(binding),
                    value: None,
                    error: Some(UiInvocationError::HandlerFailed(error.to_string())),
                };
            }
        };
        self.invoke_editor_binding(route_id, editor_binding)
    }

    fn invoke_route(
        &self,
        route_id: UiRouteId,
        arguments: Vec<zircon_runtime::ui::binding::UiBindingValue>,
    ) -> UiInvocationResult {
        let binding = {
            let inner = self.inner.lock().unwrap();
            inner.control_service.route_binding(route_id)
        };
        let Some(binding) = binding else {
            return UiInvocationResult::failure(
                Some(route_id),
                None,
                UiInvocationError::UnknownRoute(route_id),
            );
        };

        let editor_binding = match EditorUiBinding::from_ui_binding(binding.clone()) {
            Ok(binding) => binding,
            Err(error) => {
                return UiInvocationResult::failure(
                    Some(route_id),
                    Some(binding),
                    UiInvocationError::HandlerFailed(error.to_string()),
                );
            }
        };
        let editor_binding = if arguments.is_empty() {
            editor_binding
        } else {
            match editor_binding.with_arguments(arguments) {
                Ok(binding) => binding,
                Err(error) => {
                    return UiInvocationResult::failure(
                        Some(route_id),
                        Some(binding),
                        UiInvocationError::HandlerFailed(error.to_string()),
                    );
                }
            }
        };

        self.invoke_editor_binding(Some(route_id), editor_binding)
    }

    fn call_action(
        &self,
        node_path: UiNodePath,
        action_id: String,
        arguments: Vec<zircon_runtime::ui::binding::UiBindingValue>,
    ) -> UiInvocationResult {
        let route_id = {
            let inner = self.inner.lock().unwrap();
            let Some(node) = inner.control_service.query_node(&node_path) else {
                return UiInvocationResult::failure(
                    None,
                    None,
                    UiInvocationError::UnknownNode(node_path.0),
                );
            };
            let Some(action) = node.actions.get(&action_id) else {
                return UiInvocationResult::failure(
                    None,
                    None,
                    UiInvocationError::UnknownAction {
                        node_path: node.node_path.0,
                        action_id,
                    },
                );
            };
            if !action.callable_from_remote {
                return UiInvocationResult::failure(
                    action.route_id,
                    None,
                    UiInvocationError::ActionNotCallable {
                        node_path: node_path.0,
                        action_id: action.action_id.clone(),
                    },
                );
            }
            let Some(route_id) = action.route_id else {
                return UiInvocationResult::failure(
                    None,
                    None,
                    UiInvocationError::ActionMissingRoute {
                        node_path: node_path.0,
                        action_id: action.action_id.clone(),
                    },
                );
            };
            route_id
        };

        self.invoke_route(route_id, arguments)
    }

    fn invoke_editor_binding(
        &self,
        route_id: Option<UiRouteId>,
        binding: EditorUiBinding,
    ) -> UiInvocationResult {
        let ui_binding = binding.as_ui_binding();
        if let EditorUiBindingPayload::EditorOperation { operation_id } = binding.payload() {
            let invocation = match EditorOperationPath::parse(operation_id.clone()) {
                Ok(path) => EditorOperationInvocation::new(path),
                Err(error) => {
                    return UiInvocationResult {
                        route_id,
                        binding: Some(ui_binding),
                        value: None,
                        error: Some(UiInvocationError::HandlerFailed(error.to_string())),
                    };
                }
            };
            return match self.invoke_operation(EditorOperationSource::UiBinding, invocation) {
                Ok(record) => UiInvocationResult {
                    route_id,
                    binding: Some(ui_binding),
                    value: record.result.value,
                    error: None,
                },
                Err(error) => UiInvocationResult {
                    route_id,
                    binding: Some(ui_binding),
                    value: None,
                    error: Some(UiInvocationError::HandlerFailed(error.to_string())),
                },
            };
        }
        match self.dispatch_binding(binding, EditorEventSource::Headless) {
            Ok(record) => UiInvocationResult {
                route_id,
                binding: Some(ui_binding),
                value: record.result.value,
                error: None,
            },
            Err(error) => UiInvocationResult {
                route_id,
                binding: Some(ui_binding),
                value: None,
                error: Some(UiInvocationError::HandlerFailed(error)),
            },
        }
    }
}
