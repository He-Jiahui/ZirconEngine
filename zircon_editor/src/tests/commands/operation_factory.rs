use std::any::Any;
use std::sync::Arc;

use crate::core::commands::{EditorCommandAction, EditorCommandDescriptor, EditorCommandRegistry};
use crate::core::editing::engine::{
    CommandExecutionError, EditCommand, EditContext, HistoryContextId,
};
use crate::core::editing::operation::{
    OperationCommand, OperationCommandFactory, OperationCommandFactoryError,
    OperationCommandFactoryRegistration,
};
use crate::core::editor_event::{EditorEvent, EditorEventTransient};
use crate::core::editor_operation::{EditorOperationInvocation, EditorOperationPath};

#[test]
fn command_registry_registers_descriptor_and_factory_as_one_operation() {
    let operation_id = EditorOperationPath::parse("test.operation.increment").unwrap();
    let descriptor = EditorCommandDescriptor::operation(operation_id.clone(), "Increment");
    assert!(matches!(
        descriptor.action(),
        EditorCommandAction::Operation
    ));
    let registration = OperationCommandFactoryRegistration::new(
        operation_id.clone(),
        "Increment",
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
fn command_registry_rejects_factory_registered_for_another_operation() {
    let operation_id = EditorOperationPath::parse("test.operation.increment").unwrap();
    let foreign_id = EditorOperationPath::parse("test.operation.foreign").unwrap();
    let descriptor = EditorCommandDescriptor::operation(operation_id.clone(), "Increment");
    let registration = OperationCommandFactoryRegistration::new(
        foreign_id.clone(),
        "Increment",
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
        "Event Route",
        crate::core::commands::EditorCommandCategory::Command,
        EditorCommandAction::Emit(EditorEvent::Transient(
            EditorEventTransient::OpenCommandPalette,
        )),
    );
    let registration = OperationCommandFactoryRegistration::new(
        operation_id.clone(),
        "Event Route",
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

impl EditCommand for FixtureOperationCommand {
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
