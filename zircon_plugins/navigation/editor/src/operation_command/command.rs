use std::any::Any;

use zircon_editor::core::editing::engine::{
    CommandExecutionError, EditCommand, EditCommandError, EditContext, MergeOutcome,
};
use zircon_editor::core::gateway::EditorRuntimeGateway;
use zircon_runtime::core::framework::navigation::{
    NavigationGeneratedBakeChange, NavigationGeneratedBakeSnapshot,
    NAVIGATION_BAKE_SCENE_OPERATION, NAVIGATION_BAKE_SURFACE_OPERATION,
    NAVIGATION_CLEAR_SURFACE_OPERATION, NAVIGATION_RESTORE_BAKE_OPERATION,
};
use zircon_runtime_interface::{ZrRuntimeOperationSubmitRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::error::NavigationOperationCommandError;

const MAX_OPERATION_POLLS: usize = 16;

pub(crate) struct NavigationOperationCommand {
    label: &'static str,
    request: ZrRuntimeOperationSubmitRequestV1,
    before: Option<NavigationGeneratedBakeSnapshot>,
    after: Option<NavigationGeneratedBakeSnapshot>,
}

impl NavigationOperationCommand {
    pub(crate) fn new(request: ZrRuntimeOperationSubmitRequestV1) -> Self {
        let label = operation_label(&request.operation_id);
        Self {
            label,
            request,
            before: None,
            after: None,
        }
    }

    fn execute(
        context: &dyn EditContext,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<NavigationGeneratedBakeChange, CommandExecutionError> {
        let gateway = context.runtime_gateway();
        let expected_operation = request.operation_id.clone();
        let handle = gateway
            .submit_operation(request)
            .map_err(unchanged_gateway_error)?;
        let mut terminal = false;
        for _ in 0..MAX_OPERATION_POLLS {
            let progress = gateway
                .poll_operation(handle)
                .map_err(applied_gateway_error)?;
            if progress.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
                return Err(applied_external_error(
                    NavigationOperationCommandError::Protocol {
                        message: format!(
                            "progress ABI version {} is unsupported",
                            progress.abi_version
                        ),
                    },
                ));
            }
            if progress.handle != handle {
                return Err(applied_external_error(
                    NavigationOperationCommandError::Protocol {
                        message: "progress returned a foreign operation handle".to_string(),
                    },
                ));
            }
            if progress.phase.is_terminal() {
                terminal = true;
                break;
            }
            std::thread::yield_now();
        }
        if !terminal {
            return Err(applied_external_error(
                NavigationOperationCommandError::PollBudgetExhausted,
            ));
        }
        let result = gateway
            .harvest_operation(handle)
            .map_err(applied_gateway_error)?;
        if result.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V1 {
            return Err(applied_external_error(
                NavigationOperationCommandError::Protocol {
                    message: format!("result ABI version {} is unsupported", result.abi_version),
                },
            ));
        }
        if result.handle != handle || result.operation_id != expected_operation {
            return Err(applied_external_error(
                NavigationOperationCommandError::Protocol {
                    message: "result did not match the submitted operation".to_string(),
                },
            ));
        }
        let output = result.succeeded_output().ok_or_else(|| {
            applied_external_error(NavigationOperationCommandError::Failed {
                message: result
                    .failure()
                    .unwrap_or("runtime operation failed without diagnostics")
                    .to_string(),
            })
        })?;
        serde_json::from_value(output.clone()).map_err(|error| {
            applied_external_error(NavigationOperationCommandError::Protocol {
                message: format!("decode generated bake change: {error}"),
            })
        })
    }

    fn restore_request(
        snapshot: &NavigationGeneratedBakeSnapshot,
    ) -> Result<ZrRuntimeOperationSubmitRequestV1, CommandExecutionError> {
        let payload = serde_json::to_value(snapshot).map_err(|error| {
            unchanged_external_error(NavigationOperationCommandError::Protocol {
                message: format!("encode generated bake snapshot: {error}"),
            })
        })?;
        Ok(ZrRuntimeOperationSubmitRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            NAVIGATION_RESTORE_BAKE_OPERATION,
            payload,
        ))
    }
}

impl EditCommand for NavigationOperationCommand {
    fn label(&self) -> &str {
        self.label
    }

    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        if let Some(after) = &self.after {
            Self::execute(context, Self::restore_request(after)?)?;
            return Ok(());
        }
        let change = Self::execute(context, self.request.clone())?;
        self.before = Some(change.before);
        self.after = Some(change.after);
        Ok(())
    }

    fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let before = self.before.as_ref().ok_or_else(|| {
            CommandExecutionError::unchanged(EditCommandError::InvariantViolation {
                invariant: "navigation operation command has no captured before snapshot",
            })
        })?;
        Self::execute(context, Self::restore_request(before)?)?;
        Ok(())
    }

    fn try_merge(&mut self, _next: &dyn EditCommand) -> MergeOutcome {
        MergeOutcome::Reject
    }

    fn serialize_journal(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "request": self.request,
            "before": self.before,
            "after": self.after,
        }))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn unchanged_gateway_error(
    source: zircon_editor::core::gateway::GatewayError,
) -> CommandExecutionError {
    unchanged_external_error(NavigationOperationCommandError::Gateway { source })
}

fn applied_gateway_error(
    source: zircon_editor::core::gateway::GatewayError,
) -> CommandExecutionError {
    applied_external_error(NavigationOperationCommandError::Gateway { source })
}

fn unchanged_external_error(source: NavigationOperationCommandError) -> CommandExecutionError {
    CommandExecutionError::unchanged(EditCommandError::ExternalEffect {
        source: Box::new(source),
    })
}

fn applied_external_error(source: NavigationOperationCommandError) -> CommandExecutionError {
    CommandExecutionError::applied(EditCommandError::ExternalEffect {
        source: Box::new(source),
    })
}

fn operation_label(operation_id: &str) -> &'static str {
    match operation_id {
        NAVIGATION_BAKE_SCENE_OPERATION => "Bake Navigation Scene",
        NAVIGATION_BAKE_SURFACE_OPERATION => "Bake Navigation Surface",
        NAVIGATION_CLEAR_SURFACE_OPERATION => "Clear Navigation Surface Bake",
        _ => "Navigation Runtime Operation",
    }
}
