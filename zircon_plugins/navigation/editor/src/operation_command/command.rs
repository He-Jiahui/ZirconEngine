use std::any::Any;

use zircon_editor::core::editing::engine::{
    CommandExecutionError, CommandJournalPayload, CommandJournalUnavailable, EditCommand,
    EditCommandError, EditContext, MergeOutcome,
};
use zircon_runtime::core::framework::navigation::{
    NavigationGeneratedBakeChange, NavigationGeneratedBakeSnapshot,
    NAVIGATION_BAKE_SCENE_OPERATION, NAVIGATION_BAKE_SURFACE_OPERATION,
    NAVIGATION_CLEAR_SURFACE_OPERATION, NAVIGATION_RESTORE_BAKE_OPERATION,
};
use zircon_runtime_interface::{
    ZrRuntimeOperationSubmitRequestV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_ABI_VERSION_V2,
};

use super::error::NavigationOperationCommandError;

const MAX_OPERATION_POLLS: usize = 16;

pub(crate) struct NavigationOperationCommand {
    label: &'static str,
    request: ZrRuntimeOperationSubmitRequestV1,
    before: Option<NavigationGeneratedBakeSnapshot>,
    after: Option<NavigationGeneratedBakeSnapshot>,
    history_budget: Option<zircon_runtime::core::framework::navigation::NavigationHistoryBudgetV1>,
}

impl NavigationOperationCommand {
    pub(crate) fn new(request: ZrRuntimeOperationSubmitRequestV1) -> Self {
        let label = operation_label(&request.operation_id);
        Self {
            label,
            request,
            before: None,
            after: None,
            history_budget: None,
        }
    }

    // 提交前的错误保持工作区不变；提交后的轮询、ABI 或结果错误按可能已应用处理，交由事务引擎决定恢复。
    fn execute(
        context: &dyn EditContext,
        request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<NavigationGeneratedBakeChange, CommandExecutionError> {
        let gateway = context
            .runtime_operations()
            .map_err(CommandExecutionError::unchanged)?;
        let expected_operation = request.operation_id.clone();
        let handle = gateway
            .submit_operation(request)
            .map_err(unchanged_gateway_error)?;
        let mut terminal = false;
        for _ in 0..MAX_OPERATION_POLLS {
            gateway.tick_frame().map_err(applied_gateway_error)?;
            let progress = gateway
                .poll_operation(handle)
                .map_err(applied_gateway_error)?;
            if progress.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V2 {
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
            let phase = progress.phase().ok_or_else(|| {
                applied_external_error(NavigationOperationCommandError::Protocol {
                    message: format!("progress phase {} is unsupported", progress.phase),
                })
            })?;
            if phase.is_terminal() {
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
        if expected_operation.starts_with("navigation.history.v1.") {
            let receipt = output.get("_history_retention_v1");
            if receipt
                .and_then(|r| r.get("version"))
                .and_then(serde_json::Value::as_u64)
                != Some(1)
            {
                return Err(applied_external_error(
                    NavigationOperationCommandError::Protocol {
                        message: "missing history admission receipt".into(),
                    },
                ));
            }
            if receipt
                .and_then(|r| r.get("accepted"))
                .and_then(serde_json::Value::as_bool)
                == Some(false)
                && receipt
                    .and_then(|r| r.get("reason"))
                    .and_then(serde_json::Value::as_str)
                    == Some("prepublication_admission")
            {
                return Err(CommandExecutionError::unchanged(
                    EditCommandError::RetentionAdmissionRefused {
                        reason: "navigation receiver DTO exceeds its prepublication allowance",
                    },
                ));
            }
            if receipt
                .and_then(|r| r.get("accepted"))
                .and_then(serde_json::Value::as_bool)
                != Some(true)
            {
                return Err(applied_external_error(
                    NavigationOperationCommandError::Protocol {
                        message: "invalid history admission receipt".into(),
                    },
                ));
            }
        }
        serde_json::from_value(output.clone()).map_err(|error| {
            applied_external_error(NavigationOperationCommandError::Protocol {
                message: format!("decode generated bake change: {error}"),
            })
        })
    }

    fn admission_request(
        &self,
        mut request: ZrRuntimeOperationSubmitRequestV1,
    ) -> Result<ZrRuntimeOperationSubmitRequestV1, CommandExecutionError> {
        use zircon_runtime::core::framework::navigation::*;
        let Some(budget) = self.history_budget else {
            return Ok(request);
        };
        request.operation_id = match request.operation_id.as_str() {
            NAVIGATION_BAKE_SCENE_OPERATION => NAVIGATION_HISTORY_BAKE_SCENE_V1,
            NAVIGATION_BAKE_SURFACE_OPERATION => NAVIGATION_HISTORY_BAKE_SURFACE_V1,
            NAVIGATION_CLEAR_SURFACE_OPERATION => NAVIGATION_HISTORY_CLEAR_SURFACE_V1,
            NAVIGATION_RESTORE_BAKE_OPERATION => NAVIGATION_HISTORY_RESTORE_V1,
            _ => {
                return Err(CommandExecutionError::unchanged(
                    EditCommandError::InvariantViolation {
                        invariant: "unsupported navigation history operation",
                    },
                ))
            }
        }
        .to_string();
        request.payload = serde_json::to_value(NavigationHistoryRequestV1 {
            budget,
            payload: request.payload,
        })
        .map_err(|_| {
            CommandExecutionError::unchanged(EditCommandError::InvariantViolation {
                invariant: "history admission envelope serialization",
            })
        })?;
        Ok(request)
    }

    fn input_peak_v1(
        &self,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<u64, zircon_runtime_interface::retained_memory::RetentionContractError> {
        use zircon_runtime::core::framework::navigation::{
            navigation_history_envelope_peak_v1, navigation_history_request_peak_v1,
            navigation_history_restore_peak_v1,
        };
        use zircon_runtime_interface::retained_memory::RetentionContractError;
        let mut input = navigation_history_request_peak_v1(model, &self.request, limit)?;
        // Compensation may restore either side. Quote both without constructing
        // a Value, request clone or transport buffer and keep the larger peak.
        for snapshot in self.before.iter().chain(self.after.iter()) {
            input = input.max(navigation_history_restore_peak_v1(model, snapshot, limit)?);
        }
        navigation_history_envelope_peak_v1(model)?
            .checked_add(input)
            .ok_or(RetentionContractError::Overflow)
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
    fn retention_plan_v1(
        &self,
        context: &dyn EditContext,
        phase: zircon_editor::core::editing::engine::RetentionPhaseV1,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<zircon_editor::core::editing::engine::CommandRetentionPlanV1, EditCommandError>
    {
        use zircon_editor::core::editing::engine::CommandRetentionPlanV1;
        use zircon_runtime_interface::retained_memory::RetentionContractError;
        let budget = self
            .history_budget
            .ok_or(RetentionContractError::Provider {
                reason: "navigation history requires an engine residual reservation",
            })?;
        // Compensation reuses the receiver ceiling already admitted by the
        // real engine. Ordinary direct apply retains its legacy no-history API.
        let mut plan = CommandRetentionPlanV1::for_owner(self, context, phase, model, limit)?;
        plan.peak_bytes = plan
            .retained_bytes
            .checked_add(self.input_peak_v1(model, limit)?)
            .and_then(|n| n.checked_add(budget.peak_bytes))
            .ok_or(RetentionContractError::Overflow)?;
        plan.transfer_budget_v1 = Some((budget.retained_bytes, budget.peak_bytes));
        Ok(plan)
    }

    fn retention_plan_with_limits_v1(
        &self,
        context: &dyn EditContext,
        phase: zircon_editor::core::editing::engine::RetentionPhaseV1,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limits: zircon_editor::core::editing::engine::CommandRetentionLimitsV1,
    ) -> Result<zircon_editor::core::editing::engine::CommandRetentionPlanV1, EditCommandError>
    {
        use zircon_editor::core::editing::engine::CommandRetentionPlanV1;
        use zircon_runtime_interface::retained_memory::RetentionContractError;
        let mut plan =
            CommandRetentionPlanV1::for_owner(self, context, phase, model, limits.retained_bytes)?;
        if plan.selection_bytes > limits.selection_bytes {
            return Err(RetentionContractError::Stale.into());
        }
        let command = plan.retained_bytes;
        let envelope = self.input_peak_v1(model, limits.materialization_bytes)?;
        // Engine peak = 2*(A+C+D) + (C+E+P) + H + lease.
        // Its supplied materialization remainder already subtracts 2*A+H+lease.
        // Thus 3*C+E+2*D+P must fit, with P >= 8*D for all DTO owners.
        let fixed = command
            .checked_mul(3)
            .and_then(|n| n.checked_add(envelope))
            .ok_or(RetentionContractError::Overflow)?;
        let available = limits.materialization_bytes.checked_sub(fixed).ok_or(
            RetentionContractError::Budget {
                budget: "navigation command and envelope peak",
                requested: fixed,
                limit: limits.materialization_bytes,
            },
        )?;
        let retained = limits
            .retained_bytes
            .checked_sub(command)
            .ok_or(RetentionContractError::Overflow)?;
        let dto = retained.min(available / 10);
        if dto == 0 {
            return Err(RetentionContractError::Budget {
                budget: "navigation receiver owner",
                requested: 1,
                limit: dto,
            }
            .into());
        }
        let dto_peak = available
            .checked_sub(dto.checked_mul(2).ok_or(RetentionContractError::Overflow)?)
            .ok_or(RetentionContractError::Overflow)?;
        plan.retained_bytes = command
            .checked_add(dto)
            .ok_or(RetentionContractError::Overflow)?;
        plan.peak_bytes = command
            .checked_add(envelope)
            .and_then(|n| n.checked_add(dto_peak))
            .ok_or(RetentionContractError::Overflow)?;
        plan.transfer_budget_v1 = Some((dto, dto_peak));
        Ok(plan)
    }

    fn prepare_retention_v1(
        &mut self,
        _context: &mut dyn EditContext,
        phase: zircon_editor::core::editing::engine::RetentionPhaseV1,
        permit: &mut zircon_editor::core::editing::engine::RetentionMaterializationV1,
    ) -> Result<(), CommandExecutionError> {
        permit
            .consume(self, phase)
            .map_err(CommandExecutionError::unchanged)?;
        let (retained_bytes, peak_bytes) = permit.plan().transfer_budget_v1.ok_or_else(|| {
            CommandExecutionError::unchanged(
                zircon_runtime_interface::retained_memory::RetentionContractError::Provider {
                    reason: "navigation receiver permit has no transfer ceiling",
                }
                .into(),
            )
        })?;
        self.history_budget = Some(
            zircon_runtime::core::framework::navigation::NavigationHistoryBudgetV1 {
                version: 1,
                retained_bytes,
                peak_bytes,
            },
        );
        Ok(())
    }

    fn label(&self) -> &str {
        self.label
    }

    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        if let Some(after) = &self.after {
            Self::execute(
                context,
                self.admission_request(Self::restore_request(after)?)?,
            )?;
            return Ok(());
        }
        let change = Self::execute(context, self.admission_request(self.request.clone())?)?;
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
        Self::execute(
            context,
            self.admission_request(Self::restore_request(before)?)?,
        )?;
        Ok(())
    }

    fn try_merge(&mut self, _next: &dyn EditCommand) -> MergeOutcome {
        MergeOutcome::Reject
    }

    fn journal_payload(&self) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        Ok(CommandJournalPayload::new(
            "navigation.operation",
            1,
            serde_json::json!({
                "request": self.request,
                "before": self.before,
                "after": self.after,
            }),
        ))
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

impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for NavigationOperationCommand {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        use zircon_runtime_interface::retained_memory::RetainedOwnerV1;
        self.request.retained_bound_v1(m)?;
        self.before.retained_bound_v1(m)?;
        self.after.retained_bound_v1(m)
    }
}
