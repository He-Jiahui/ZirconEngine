use crate::core::editor_message::DocumentId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayEditTarget {
    PlayDomain,
    EditDocument(DocumentId),
    EditWorkspace,
}
