use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::core::editing::operation::{
    OperationCommandFactoryError, OperationCommandFactoryRegistration,
};
use crate::core::editor_operation::EditorOperationPath;

use super::{
    EditorCommandAction, EditorCommandDescriptor, EditorCommandRegistry, EditorCommandRegistryError,
};

/// One-shot command descriptors plus the stable ids retained after registration.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorCommandContributionSet {
    command_ids: BTreeSet<EditorOperationPath>,
    pending: BTreeMap<EditorOperationPath, EditorCommandDescriptor>,
    #[serde(skip)]
    pending_factories: BTreeMap<EditorOperationPath, OperationCommandFactoryRegistration>,
}

impl EditorCommandContributionSet {
    pub fn register(
        &mut self,
        descriptor: EditorCommandDescriptor,
    ) -> Result<(), EditorCommandRegistryError> {
        EditorCommandRegistry::validate_descriptor(&descriptor)?;
        if self.command_ids.contains(descriptor.id()) {
            return Err(EditorCommandRegistryError::DuplicateCommand(
                descriptor.id().clone(),
            ));
        }
        self.command_ids.insert(descriptor.id().clone());
        self.pending.insert(descriptor.id().clone(), descriptor);
        Ok(())
    }

    pub fn register_operation(
        &mut self,
        descriptor: EditorCommandDescriptor,
        factory: OperationCommandFactoryRegistration,
    ) -> Result<(), EditorCommandRegistryError> {
        if descriptor.id() != factory.operation() {
            return Err(EditorCommandRegistryError::OperationFactory(
                OperationCommandFactoryError::OperationMismatch {
                    descriptor_operation: descriptor.id().clone(),
                    factory_operation: factory.operation().clone(),
                },
            ));
        }
        if !matches!(descriptor.action(), EditorCommandAction::Operation) {
            return Err(EditorCommandRegistryError::OperationFactory(
                OperationCommandFactoryError::DescriptorIsEvent {
                    operation: descriptor.id().clone(),
                },
            ));
        }
        EditorCommandRegistry::validate_descriptor(&descriptor)?;
        if self.command_ids.contains(descriptor.id()) {
            return Err(EditorCommandRegistryError::DuplicateCommand(
                descriptor.id().clone(),
            ));
        }
        if self.pending_factories.contains_key(factory.operation()) {
            return Err(EditorCommandRegistryError::OperationFactory(
                OperationCommandFactoryError::DuplicateFactory {
                    operation: factory.operation().clone(),
                },
            ));
        }
        let operation = descriptor.id().clone();
        self.command_ids.insert(operation.clone());
        self.pending.insert(operation.clone(), descriptor);
        self.pending_factories.insert(operation, factory);
        Ok(())
    }

    pub fn command_ids(&self) -> impl Iterator<Item = &EditorOperationPath> {
        self.command_ids.iter()
    }

    pub fn pending_command(&self, id: &EditorOperationPath) -> Option<&EditorCommandDescriptor> {
        self.pending.get(id)
    }

    pub fn pending_commands(&self) -> impl Iterator<Item = &EditorCommandDescriptor> {
        self.pending.values()
    }

    pub fn pending_factory(
        &self,
        id: &EditorOperationPath,
    ) -> Option<&OperationCommandFactoryRegistration> {
        self.pending_factories.get(id)
    }

    pub fn take_pending(&mut self) -> Vec<EditorCommandDescriptor> {
        std::mem::take(&mut self.pending).into_values().collect()
    }

    pub fn take_pending_factories(&mut self) -> Vec<OperationCommandFactoryRegistration> {
        std::mem::take(&mut self.pending_factories)
            .into_values()
            .collect()
    }

    pub(crate) fn record_registered_id(&mut self, id: EditorOperationPath) {
        self.command_ids.insert(id);
    }
}
