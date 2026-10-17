//! 核对操作描述符与工厂的单一注册关系，并由工厂独占延迟编辑对象的保留期，防止事件描述符或异主工厂进入命令执行路径。
use std::any::Any;
use std::sync::Arc;

use crate::core::commands::{EditorCommandAction, EditorCommandDescriptor, EditorCommandRegistry};
use crate::core::editing::engine::{
    CommandExecutionError, EditCommand, EditContext, HistoryContextId,
};
use crate::core::editing::operation::{
    EditOperationTarget, OperationCommand, OperationCommandFactory, OperationCommandFactoryError,
    OperationCommandFactoryRegistration, PendingEditRetention,
};
use crate::core::editor_event::{EditorEvent, EditorEventTransient};
use crate::core::editor_operation::{EditorOperationInvocation, EditorOperationPath};

#[test]
fn command_registry_registers_descriptor_and_factory_as_one_operation() {
    let operation_id = EditorOperationPath::parse("test.operation.increment").unwrap();
    let descriptor = EditorCommandDescriptor::operation(operation_id.clone());
    assert!(matches!(
        descriptor.action(),
        EditorCommandAction::Operation
    ));
    let registration = OperationCommandFactoryRegistration::new(
        operation_id.clone(),
        "Increment",
        EditOperationTarget::EditWorkspace,
        Arc::new(FixtureOperationFactory),
    );
    let mut registry = EditorCommandRegistry::default();

    registry
        .register_operation(descriptor, registration)
        .unwrap();

    let invocation = EditorOperationInvocation::new(operation_id.clone())
        .with_arguments(serde_json::json!({ "delta": 7 }));
    let operation = registry
        .operation_factory(&operation_id)
        .unwrap()
        .create(&invocation)
        .unwrap();
    assert_eq!(operation.history(), HistoryContextId::Global);
    assert_eq!(
        registry
            .operation_factory(&operation_id)
            .unwrap()
            .edit_target(),
        EditOperationTarget::EditWorkspace
    );
    assert_eq!(operation.command().label(), "Increment");
    assert_eq!(
        registry
            .operation_factory(&operation_id)
            .unwrap()
            .undo_display_name(),
        "Increment"
    );
    assert_eq!(
        registry.command(operation_id.as_str()).unwrap(),
        registry.commands().next().unwrap()
    );
}

#[test]
fn operation_factory_is_the_only_owner_of_deferred_edit_retention() {
    let operation_id = EditorOperationPath::parse("test.operation.rename").unwrap();
    let registration = OperationCommandFactoryRegistration::new(
        operation_id.clone(),
        "Rename",
        EditOperationTarget::EditWorkspace,
        Arc::new(FixtureOperationFactory),
    )
    .with_pending_edit_retention(
        PendingEditRetention::bounded(2, 1_024, std::time::Duration::from_secs(5)).unwrap(),
    );
    let invocation = EditorOperationInvocation::new(operation_id)
        .with_arguments(serde_json::json!({ "delta": 7 }));

    let deferred = registration.defer(invocation).unwrap();

    assert_eq!(
        deferred.retention(),
        &PendingEditRetention::bounded(2, 1_024, std::time::Duration::from_secs(5)).unwrap()
    );
    assert_eq!(
        deferred.invocation().operation_id.as_str(),
        "test.operation.rename"
    );
}

#[test]
fn command_registry_rejects_factory_registered_for_another_operation() {
    let operation_id = EditorOperationPath::parse("test.operation.increment").unwrap();
    let foreign_id = EditorOperationPath::parse("test.operation.foreign").unwrap();
    let descriptor = EditorCommandDescriptor::operation(operation_id.clone());
    let registration = OperationCommandFactoryRegistration::new(
        foreign_id.clone(),
        "Increment",
        EditOperationTarget::EditWorkspace,
        Arc::new(FixtureOperationFactory),
    );
    let mut registry = EditorCommandRegistry::default();

    let error = registry
        .register_operation(descriptor, registration)
        .unwrap_err();

    assert!(matches!(
        error,
        crate::core::commands::EditorCommandRegistryError::OperationFactory(
            OperationCommandFactoryError::OperationMismatch {
                descriptor_operation,
                factory_operation,
            }
        ) if descriptor_operation == operation_id && factory_operation == foreign_id
    ));
    assert!(registry.commands().next().is_none());
    assert!(registry.operation_factory(&operation_id).is_none());
}

#[test]
fn command_registry_rejects_factory_for_event_descriptor() {
    let operation_id = EditorOperationPath::parse("test.operation.event_route").unwrap();
    let descriptor = EditorCommandDescriptor::new(
        operation_id.clone(),
        crate::core::commands::EditorCommandCategory::Command,
        EditorCommandAction::Emit(EditorEvent::Transient(
            EditorEventTransient::OpenCommandPalette,
        )),
    );
    let registration = OperationCommandFactoryRegistration::new(
        operation_id.clone(),
        "Event Route",
        EditOperationTarget::EditWorkspace,
        Arc::new(FixtureOperationFactory),
    );
    let mut registry = EditorCommandRegistry::default();

    let error = registry
        .register_operation(descriptor, registration)
        .unwrap_err();

    assert!(matches!(
        error,
        crate::core::commands::EditorCommandRegistryError::OperationFactory(
            OperationCommandFactoryError::DescriptorIsEvent { operation }
        ) if operation == operation_id
    ));
    assert!(registry.commands().next().is_none());
}

struct FixtureOperationFactory;

impl OperationCommandFactory for FixtureOperationFactory {
    fn create(
        &self,
        invocation: &EditorOperationInvocation,
    ) -> Result<OperationCommand, OperationCommandFactoryError> {
        let delta = invocation
            .arguments
            .get("delta")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| OperationCommandFactoryError::InvalidArguments {
                operation: invocation.operation_id.clone(),
                reason: "delta must be an integer".to_string(),
            })?;
        Ok(OperationCommand::new(
            Box::new(FixtureOperationCommand { delta }),
            HistoryContextId::Global,
        ))
    }
}

struct FixtureOperationCommand {
    delta: i64,
}

impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for FixtureOperationCommand {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        Ok(())
    }
}
impl EditCommand for FixtureOperationCommand {
    fn retention_plan_v1(
        &self,
        context: &dyn crate::core::editing::engine::EditContext,
        phase: crate::core::editing::engine::RetentionPhaseV1,
        model: zircon_runtime_interface::retained_memory::RetainedAccountingModelV1,
        limit: u64,
    ) -> Result<
        crate::core::editing::engine::CommandRetentionPlanV1,
        crate::core::editing::engine::EditCommandError,
    > {
        crate::core::editing::engine::CommandRetentionPlanV1::for_owner(
            self, context, phase, model, limit,
        )
    }
    fn prepare_retention_v1(
        &mut self,
        _: &mut dyn crate::core::editing::engine::EditContext,
        phase: crate::core::editing::engine::RetentionPhaseV1,
        permit: &mut crate::core::editing::engine::RetentionMaterializationV1,
    ) -> Result<(), crate::core::editing::engine::CommandExecutionError> {
        permit
            .consume(self, phase)
            .map_err(crate::core::editing::engine::CommandExecutionError::unchanged)
    }

    fn label(&self) -> &str {
        "Increment"
    }

    fn apply(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        Ok(())
    }

    fn revert(&mut self, _context: &mut dyn EditContext) -> Result<(), CommandExecutionError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
