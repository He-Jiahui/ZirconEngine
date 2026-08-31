use crate::core::editor_message::DocumentId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditOperationTarget {
    PlayDomain,
    EditDocument(DocumentId),
    EditWorkspace,
}
