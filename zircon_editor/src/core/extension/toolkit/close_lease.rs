use crate::core::editor_message::DocumentId;

use super::{
    DocumentToolkitDescriptor, DocumentToolkitRegistry, ToolkitInstanceId, ToolkitRegistryError,
};

pub struct DocumentCloseLease<'a, Host> {
    registry: &'a DocumentToolkitRegistry<Host>,
    document: DocumentId,
    instance: ToolkitInstanceId,
    committed: bool,
}

impl<'a, Host> DocumentCloseLease<'a, Host> {
    pub(super) fn new(
        registry: &'a DocumentToolkitRegistry<Host>,
        document: DocumentId,
        instance: ToolkitInstanceId,
    ) -> Self {
        Self {
            registry,
            document,
            instance,
            committed: false,
        }
    }

    pub const fn document_id(&self) -> DocumentId {
        self.document
    }

    pub fn instance_id(&self) -> &ToolkitInstanceId {
        &self.instance
    }

    pub fn commit(mut self) -> Result<DocumentToolkitDescriptor, ToolkitRegistryError> {
        let descriptor = self.registry.commit_close(self.document, &self.instance)?;
        self.committed = true;
        Ok(descriptor)
    }
}

impl<Host> Drop for DocumentCloseLease<'_, Host> {
    fn drop(&mut self) {
        if !self.committed {
            self.registry.rollback_close(self.document, &self.instance);
        }
    }
}
