use crate::core::editing::operation::EditOperationTarget;
use crate::core::editor_message::DocumentId;

use super::PlayEditDecision;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayEditPolicy {
    playing: bool,
    running_document: Option<DocumentId>,
}

impl PlayEditPolicy {
    pub fn begin_play(&mut self, running_document: Option<DocumentId>) {
        self.playing = true;
        self.running_document = running_document;
    }

    pub fn end_play(&mut self) {
        self.playing = false;
        self.running_document = None;
    }

    pub const fn is_playing(self) -> bool {
        self.playing
    }

    pub const fn running_document(self) -> Option<DocumentId> {
        self.running_document
    }

    pub fn evaluate(self, target: EditOperationTarget) -> PlayEditDecision {
        if !self.playing {
            return match target {
                EditOperationTarget::PlayDomain => PlayEditDecision::PlayDomainUnavailable,
                EditOperationTarget::EditDocument(_) | EditOperationTarget::EditWorkspace => {
                    PlayEditDecision::ApplyNow
                }
            };
        }

        match target {
            EditOperationTarget::PlayDomain => PlayEditDecision::ApplyNow,
            EditOperationTarget::EditDocument(document)
                if self.running_document == Some(document) =>
            {
                PlayEditDecision::RunningDocumentLocked { document }
            }
            EditOperationTarget::EditDocument(_) | EditOperationTarget::EditWorkspace => {
                PlayEditDecision::QueueUntilPlayStops
            }
        }
    }
}
