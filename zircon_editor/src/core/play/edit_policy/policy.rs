use crate::core::editor_message::DocumentId;

use super::{PlayEditDecision, PlayEditTarget};

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

    pub fn evaluate(self, target: PlayEditTarget) -> PlayEditDecision {
        if !self.playing {
            return match target {
                PlayEditTarget::PlayDomain => PlayEditDecision::PlayDomainUnavailable,
                PlayEditTarget::EditDocument(_) | PlayEditTarget::EditWorkspace => {
                    PlayEditDecision::ApplyNow
                }
            };
        }

        match target {
            PlayEditTarget::PlayDomain => PlayEditDecision::ApplyNow,
            PlayEditTarget::EditDocument(document) if self.running_document == Some(document) => {
                PlayEditDecision::RunningDocumentLocked { document }
            }
            PlayEditTarget::EditDocument(_) | PlayEditTarget::EditWorkspace => {
                PlayEditDecision::QueueUntilPlayStops
            }
        }
    }
}
