use crate::core::editor_message::DocumentId;

use super::DocumentToolkitRegistry;

/// Keeps a document registered while an editor mutation and its dirty projection finish.
pub struct DocumentEditLease<'a, Host> {
    registry: &'a DocumentToolkitRegistry<Host>,
    document: DocumentId,
}

impl<'a, Host> DocumentEditLease<'a, Host> {
    pub(super) fn new(registry: &'a DocumentToolkitRegistry<Host>, document: DocumentId) -> Self {
        Self { registry, document }
    }
}

impl<Host> Drop for DocumentEditLease<'_, Host> {
    fn drop(&mut self) {
        self.registry.finish_edit(self.document);
    }
}
