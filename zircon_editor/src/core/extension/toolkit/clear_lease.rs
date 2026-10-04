use crate::core::editor_message::DocumentId;

use super::{
    DocumentToolkitDescriptor, DocumentToolkitRegistry, ToolkitInstanceId, ToolkitRegistryError,
};

pub struct DocumentClearLease<'a, Host> {
    registry: &'a DocumentToolkitRegistry<Host>,
    documents: Vec<(DocumentId, ToolkitInstanceId)>,
    committed: bool,
}

impl<'a, Host> DocumentClearLease<'a, Host> {
    pub(super) fn new(
        registry: &'a DocumentToolkitRegistry<Host>,
        documents: Vec<(DocumentId, ToolkitInstanceId)>,
    ) -> Self {
        Self {
            registry,
            documents,
            committed: false,
        }
    }

    pub fn commit(mut self) -> Result<Vec<DocumentToolkitDescriptor>, ToolkitRegistryError> {
        let descriptors = self.registry.commit_clear(&self.documents)?;
        self.committed = true;
        Ok(descriptors)
    }
}

impl<Host> Drop for DocumentClearLease<'_, Host> {
    fn drop(&mut self) {
        if !self.committed {
            self.registry.rollback_clear(&self.documents);
        }
    }
}
