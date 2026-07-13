use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::core::editor_operation::EditorOperationPath;

use super::{EditorCommandDescriptor, EditorCommandRegistry, EditorCommandRegistryError};

/// One-shot command descriptors plus the stable ids retained after registration.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorCommandContributionSet {
    command_ids: BTreeSet<EditorOperationPath>,
    pending: BTreeMap<EditorOperationPath, EditorCommandDescriptor>,
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

    pub fn command_ids(&self) -> impl Iterator<Item = &EditorOperationPath> {
        self.command_ids.iter()
    }

    pub fn pending_command(&self, id: &EditorOperationPath) -> Option<&EditorCommandDescriptor> {
        self.pending.get(id)
    }

    pub fn pending_commands(&self) -> impl Iterator<Item = &EditorCommandDescriptor> {
        self.pending.values()
    }

    pub fn take_pending(&mut self) -> Vec<EditorCommandDescriptor> {
        std::mem::take(&mut self.pending).into_values().collect()
    }

    pub(crate) fn record_registered_id(&mut self, id: EditorOperationPath) {
        self.command_ids.insert(id);
    }
}
