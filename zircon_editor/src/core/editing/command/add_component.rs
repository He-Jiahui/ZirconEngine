//! Absence-only Inspector Add command. The host validates UI scope before
//! dispatch; this owner repeats world, selection, schema and instance admission.

use super::*;
use crate::core::editing::engine::{
    CommandRetentionPlanV1, HistoryContextId, RetentionMaterializationV1, RetentionPhaseV1,
};
use crate::core::editing::operation::{
    OperationCommand, OperationCommandFactory, OperationCommandFactoryError,
};
use crate::core::editor_operation::EditorOperationInvocation;
use crate::core::gateway::GatewaySessionIdentity;
use crate::core::play::WorldDomain;
use zircon_runtime::scene::world::{DynamicComponentCreationId, DynamicComponentCreationSpec};
use zircon_runtime_interface::retained_memory::*;

pub(crate) const ADD_COMPONENT_OPERATION: &str = "inspector.component.add";
const JOURNAL_TYPE: &str = "zircon.editor.scene.add_dynamic_component";

/// All runtime/session identity fields are compared, including optional project/Play.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GatewayStamp {
    runtime_instance: u64,
    runtime_session: u64,
    gateway_generation: u64,
    transport_epoch: u64,
    project: Option<String>,
    play_instance: Option<u64>,
}
impl GatewayStamp {
    pub(crate) fn capture(identity: &GatewaySessionIdentity) -> Self {
        Self {
            runtime_instance: identity.runtime_instance(),
            runtime_session: identity.runtime_session().raw(),
            gateway_generation: identity.gateway_generation(),
            transport_epoch: identity.transport_epoch(),
            project: identity.project().map(str::to_owned),
            play_instance: identity.play_instance(),
        }
    }
    fn matches(&self, identity: &GatewaySessionIdentity) -> bool {
        self.runtime_instance == identity.runtime_instance()
            && self.runtime_session == identity.runtime_session().raw()
            && self.gateway_generation == identity.gateway_generation()
            && self.transport_epoch == identity.transport_epoch()
            && self.project.as_deref() == identity.project()
            && self.play_instance == identity.play_instance()
    }
}
zircon_runtime_interface::impl_retained_owner_v1!(GatewayStamp { project });

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AddComponentScope {
    pub(crate) history: HistoryContextId,
    pub(crate) gateway: GatewayStamp,
    pub(crate) ui_selection_generation: u64,
    pub(crate) core_selection_generation: u64,
    pub(crate) world_generation: u64,
    pub(crate) targets: Vec<NodeId>,
    pub(crate) primary: Option<NodeId>,
}
zircon_runtime_interface::impl_retained_owner_v1!(AddComponentScope { gateway, targets });

impl AddComponentScope {
    pub(crate) fn validate(&self, core: &CoreEditContext) -> Result<(), EditCommandError> {
        if !matches!(self.history, HistoryContextId::Document(_)) {
            return Err(invalid(
                "component creation must use the committed scene document history",
            ));
        }
        validate_gateway(core, &self.gateway)?;
        let selected = core.scene_selection()?;
        if core.selection_snapshot().generation() != self.core_selection_generation
            || selected.items() != self.targets.as_slice()
            || selected.primary() != self.primary
        {
            return Err(invalid("component creation selection changed"));
        }
        core.with_scene(|scene| {
            if scene.world_generation() != self.world_generation {
                Err(invalid("component creation world changed"))
            } else {
                Ok(())
            }
        })?
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AddComponentCatalogEntry {
    pub(crate) type_path: String,
    pub(crate) display_name: String,
    pub(crate) already_present: bool,
}
#[derive(Clone, Debug)]
pub(crate) struct AddComponentCatalog {
    pub(crate) scope: AddComponentScope,
    pub(crate) page: usize,
    pub(crate) total: usize,
    pub(crate) entries: Vec<AddComponentCatalogEntry>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AddComponentRequest {
    pub(crate) scope: AddComponentScope,
    pub(crate) component: DynamicComponentCreationSpec,
}
impl AddComponentRequest {
    pub(crate) fn from_invocation(
        invocation: &EditorOperationInvocation,
    ) -> Result<Self, OperationCommandFactoryError> {
        serde_json::from_value(invocation.arguments.clone()).map_err(|error| {
            OperationCommandFactoryError::InvalidArguments {
                operation: invocation.operation_id.clone(),
                reason: error.to_string(),
            }
        })
    }
    pub(crate) fn validate(&self, core: &CoreEditContext) -> Result<(), EditCommandError> {
        self.scope.validate(core)?;
        core.with_scene(|scene| {
            scene.validate_dynamic_component_creation(&self.scope.targets, &self.component)
        })?
        .map_err(creation_error)
    }
}
zircon_runtime_interface::impl_retained_owner_v1!(AddComponentRequest { scope, component });

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AddComponentPayload {
    pub(crate) targets: Vec<NodeId>,
    pub(crate) component: DynamicComponentCreationSpec,
}
zircon_runtime_interface::impl_retained_owner_v1!(AddComponentPayload { targets, component });

pub(crate) struct AddDynamicComponentFactory;
impl OperationCommandFactory for AddDynamicComponentFactory {
    fn create(
        &self,
        invocation: &EditorOperationInvocation,
    ) -> Result<OperationCommand, OperationCommandFactoryError> {
        let request = AddComponentRequest::from_invocation(invocation)?;
        if !matches!(request.scope.history, HistoryContextId::Document(_))
            || request.scope.targets.is_empty()
        {
            return Err(OperationCommandFactoryError::InvalidArguments {
                operation: invocation.operation_id.clone(),
                reason: "component creation requires a document and selected targets".into(),
            });
        }
        let history = request.scope.history;
        Ok(OperationCommand::new(
            Box::new(AddDynamicComponentCommand::new(request)),
            history,
        ))
    }
}

pub(crate) struct AddDynamicComponentCommand {
    payload: AddComponentPayload,
    initial_scope: Option<AddComponentScope>,
    bound_gateway: Option<GatewayStamp>,
    receipts: Vec<DynamicComponentCreationId>,
    replay: bool,
    prepared_revision: Option<u64>,
}
zircon_runtime_interface::impl_retained_owner_v1!(AddDynamicComponentCommand {
    payload,
    initial_scope,
    bound_gateway,
    receipts
});

impl AddDynamicComponentCommand {
    pub(crate) fn new(request: AddComponentRequest) -> Self {
        Self {
            payload: AddComponentPayload {
                targets: request.scope.targets.clone(),
                component: request.component,
            },
            bound_gateway: Some(request.scope.gateway.clone()),
            initial_scope: Some(request.scope),
            receipts: Vec::new(),
            replay: false,
            prepared_revision: None,
        }
    }
    pub(crate) fn from_journal(payload: AddComponentPayload) -> Result<Self, EditCommandError> {
        if payload.targets.is_empty()
            || payload
                .targets
                .iter()
                .enumerate()
                .any(|(i, target)| payload.targets[..i].contains(target))
        {
            return Err(invalid(
                "component creation journal targets must be nonempty and distinct",
            ));
        }
        Ok(Self {
            payload,
            initial_scope: None,
            bound_gateway: None,
            receipts: Vec::new(),
            replay: true,
            prepared_revision: None,
        })
    }

    fn validate(
        &self,
        core: &CoreEditContext,
        phase: RetentionPhaseV1,
    ) -> Result<(), EditCommandError> {
        if let Some(gateway) = &self.bound_gateway {
            validate_gateway(core, gateway)?;
        }
        if !matches!(phase, RetentionPhaseV1::Undo | RetentionPhaseV1::Rollback) {
            if let Some(scope) = &self.initial_scope {
                scope.validate(core)?;
            }
        }
        core.with_scene(|scene| {
            if matches!(phase, RetentionPhaseV1::Undo | RetentionPhaseV1::Rollback) {
                scene.validate_created_dynamic_components(
                    &self.payload.targets,
                    &self.payload.component,
                    &self.receipts,
                )
            } else if self.replay {
                scene.validate_dynamic_component_creation_replay(
                    &self.payload.targets,
                    &self.payload.component,
                )
            } else {
                scene.validate_dynamic_component_creation(
                    &self.payload.targets,
                    &self.payload.component,
                )
            }
        })?
        .map_err(creation_error)
    }
}

impl EditCommand for AddDynamicComponentCommand {
    fn retention_plan_v1(
        &self,
        context: &dyn EditContext,
        phase: RetentionPhaseV1,
        model: RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<CommandRetentionPlanV1, EditCommandError> {
        let core = context.as_any().downcast_ref::<CoreEditContext>().ok_or(
            EditCommandError::ContextTypeMismatch {
                expected: "CoreEditContext",
            },
        )?;
        self.validate(core, phase)?;
        let mut plan = CommandRetentionPlanV1::for_owner(self, context, phase, model, limit)?;
        plan.owner_revision = Some(core.with_scene(|scene| scene.world_generation())?);
        if !matches!(phase, RetentionPhaseV1::Undo | RetentionPhaseV1::Rollback) {
            let mut future = RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(limit));
            future.array::<DynamicComponentCreationId>(self.payload.targets.len())?;
            for _ in &self.payload.targets {
                DynamicComponentCreationId::allocation_bound_v1(&mut future)?;
            }
            // This includes every future owned token, even when redo overlaps the
            // old receipt allocation. Never substitute an unsupported/zero quote.
            plan.add_retained(future.bytes().get())?;
            if self.bound_gateway.is_none() {
                let route = core.capture_world_route(WorldDomain::Edit)?;
                if let Some(project) = route
                    .gateway_identity()
                    .and_then(|identity| identity.project())
                {
                    future.array::<u8>(project.len())?;
                    plan.add_retained(project.len() as u64)?;
                }
            }
        }
        // Removal and compensation may also clone archetype signatures. Admit
        // the same conservative staged-row peak for every phase before mutation.
        let mut scratch = RetainedBoundBuilderV1::with_limit(model, RetainedBytesV1::new(limit));
        core.with_scene(|scene| {
            scene.dynamic_component_creation_peak_v1(
                &self.payload.targets,
                &self.payload.component.value,
                &self.payload.component.type_path,
                &mut scratch,
            )
        })??;
        plan.add_peak(scratch.bytes().get())?;
        plan.add_peak(plan.retained_bytes)?;
        Ok(plan)
    }

    fn prepare_retention_v1(
        &mut self,
        context: &mut dyn EditContext,
        phase: RetentionPhaseV1,
        permit: &mut RetentionMaterializationV1,
    ) -> Result<(), CommandExecutionError> {
        permit.consume(self, phase).map_err(unchanged)?;
        let core = core_context(context)?;
        self.validate(core, phase).map_err(unchanged)?;
        let revision = core
            .with_scene(|scene| scene.world_generation())
            .map_err(unchanged)?;
        if permit.plan().owner_revision != Some(revision) {
            return Err(unchanged(RetentionContractError::Stale.into()));
        }
        if self.bound_gateway.is_none() {
            let route = core
                .capture_world_route(WorldDomain::Edit)
                .map_err(unchanged)?;
            let identity = route.gateway_identity().ok_or_else(|| {
                unchanged(invalid("component creation needs a runtime world route"))
            })?;
            self.bound_gateway = Some(GatewayStamp::capture(identity));
        }
        if self.replay && !matches!(phase, RetentionPhaseV1::Undo | RetentionPhaseV1::Rollback) {
            self.payload.component.schema_generation = core
                .with_scene(|scene| {
                    scene.component_type_schema_generation(&self.payload.component.type_path)
                })
                .map_err(unchanged)?;
        }
        self.prepared_revision = Some(revision);
        permit.check_prepared(self).map_err(unchanged)
    }

    fn label(&self) -> &str {
        "Add dynamic component"
    }

    fn apply(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let core = core_context(context)?;
        self.validate(core, RetentionPhaseV1::Apply)
            .map_err(unchanged)?;
        let completion = execute_scene_write(core, |scene| {
            if self.prepared_revision != Some(scene.world_generation()) {
                return Err(RetentionContractError::Stale.into());
            }
            scene
                .create_dynamic_components(&self.payload.targets, &self.payload.component)
                .map_err(creation_error)
        })?;
        let (receipts, error) = match completion {
            SceneWriteCompletion::Completed(receipts) => (receipts, None),
            SceneWriteCompletion::AppliedThenGatewayFailed { value, error } => (value, Some(error)),
        };
        self.receipts = receipts;
        self.initial_scope = None;
        self.replay = false;
        self.prepared_revision = None;
        if let Some(error) = error {
            return Err(applied(error));
        }
        Ok(())
    }

    fn revert(&mut self, context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        let core = core_context(context)?;
        self.validate(core, RetentionPhaseV1::Undo)
            .map_err(unchanged)?;
        match execute_scene_write(core, |scene| {
            if self.prepared_revision != Some(scene.world_generation()) {
                return Err(RetentionContractError::Stale.into());
            }
            scene
                .remove_created_dynamic_components(
                    &self.payload.targets,
                    &self.payload.component,
                    &self.receipts,
                )
                .map_err(creation_error)
        })? {
            SceneWriteCompletion::Completed(()) => {}
            SceneWriteCompletion::AppliedThenGatewayFailed { error, .. } => {
                return Err(applied(error))
            }
        }
        self.prepared_revision = None;
        Ok(())
    }

    fn journal_payload(&self) -> Result<CommandJournalPayload, CommandJournalUnavailable> {
        journal_payload(JOURNAL_TYPE, &self.payload)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn validate_gateway(
    core: &CoreEditContext,
    expected: &GatewayStamp,
) -> Result<(), EditCommandError> {
    let route = core.capture_world_route(WorldDomain::Edit)?;
    if !route
        .gateway_identity()
        .is_some_and(|identity| expected.matches(identity))
    {
        return Err(EditCommandError::WorldRouteStale {
            world_domain: WorldDomain::Edit,
        });
    }
    Ok(())
}
fn invalid(invariant: &'static str) -> EditCommandError {
    EditCommandError::InvariantViolation { invariant }
}
fn creation_error(
    error: zircon_runtime::scene::world::DynamicComponentCreationError,
) -> EditCommandError {
    EditCommandError::ReflectError {
        source: Box::new(error),
    }
}

#[cfg(test)]
#[path = "add_component/tests.rs"]
mod tests;
