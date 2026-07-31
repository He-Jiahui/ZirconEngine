use crate::core::editor_message::DocumentId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayEditDecision {
    ApplyNow,
    PlayDomainUnavailable,
    RunningDocumentLocked { document: DocumentId },
    QueueUntilPlayStops,
}
