use std::fmt::{Display, Formatter};
use std::sync::Mutex;

use crate::core::editing::operation::{DeferredOperationInvocation, EditOperationTarget};
use crate::core::editor_message::DocumentId;
use crate::core::editor_operation::EditorOperationInvocation;

use super::{
    PendingEditApplyBudget, PendingEditApplyReport, PendingEditDecisionPrompt,
    PendingEditDiscardReport, PendingEditId, PendingEditIntent, PendingEditPage,
    PendingEditPageCursor, PendingEditQueue, PendingEditQueueError, PendingEditQueueSummary,
    PlayEditDecision, PlayEditPolicy,
};

#[derive(Debug, Default)]
pub struct PlayEditProtection {
    state: Mutex<PlayEditProtectionState>,
    pending_edits: PendingEditQueue,
}

#[derive(Debug, Default)]
struct PlayEditProtectionState {
    policy: PlayEditPolicy,
    resolving: bool,
    decision_publishing: bool,
}

impl PlayEditProtection {
    pub(super) fn begin_play(
        &self,
        running_document: Option<DocumentId>,
    ) -> Result<(), PlayEditBeginError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.policy.is_playing() {
            return Err(PlayEditBeginError::AlreadyPlaying);
        }
        if state.resolving || state.decision_publishing {
            return Err(PlayEditBeginError::ResolutionInProgress);
        }
        if let Some(prompt) = self.pending_edits.decision_prompt() {
            return Err(PlayEditBeginError::PendingDecisionRequired {
                pending_count: prompt.pending_count,
            });
        }
        state.policy.begin_play(running_document);
        Ok(())
    }

    pub(super) fn end_play(&self) -> Option<PendingEditDecisionPrompt> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.policy.end_play();
        self.pending_edits.decision_prompt()
    }

    pub(super) fn route(
        &self,
        target: EditOperationTarget,
        deferred: DeferredOperationInvocation,
    ) -> Result<PlayEditRoute, PlayEditRouteError> {
        if target != deferred.target() {
            return Err(PlayEditRouteError::TargetMismatch {
                requested: target,
                registered: deferred.target(),
            });
        }
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.resolving || state.decision_publishing {
            return Err(PlayEditRouteError::PendingResolutionInProgress);
        }
        let decision = state.policy.evaluate(target);
        match decision {
            PlayEditDecision::ApplyNow => Ok(PlayEditRoute::ApplyNow {
                target,
                invocation: deferred.into_parts().0,
            }),
            PlayEditDecision::PlayDomainUnavailable => {
                Err(PlayEditRouteError::PlayDomainUnavailable)
            }
            PlayEditDecision::RunningDocumentLocked { document } => {
                Err(PlayEditRouteError::RunningDocumentLocked { document })
            }
            PlayEditDecision::QueueUntilPlayStops => {
                let queued = self
                    .pending_edits
                    .enqueue(target, deferred)
                    .map_err(|error| match error {
                        PendingEditQueueError::TargetMismatch {
                            requested,
                            registered,
                        } => PlayEditRouteError::TargetMismatch {
                            requested,
                            registered,
                        },
                        error => PlayEditRouteError::PendingQueue(error),
                    })?;
                Ok(PlayEditRoute::Queued {
                    id: queued.id,
                    coalesced: queued.coalesced,
                    evicted_ids: queued.evicted_ids,
                })
            }
        }
    }

    pub fn pending_summary(&self) -> PendingEditQueueSummary {
        self.pending_edits.summary()
    }

    pub fn pending_page(
        &self,
        after: Option<PendingEditPageCursor>,
        limit: usize,
    ) -> PendingEditPage {
        self.pending_edits.page(after, limit)
    }

    pub fn apply_pending<E>(
        &self,
        budget: PendingEditApplyBudget,
        apply: impl FnMut(&PendingEditIntent) -> Result<(), E>,
    ) -> Result<PendingEditApplyReport<E>, PlayEditResolutionError> {
        let resolution = self.begin_resolution()?;
        let report = self.pending_edits.apply_with_budget(budget, apply);
        drop(resolution);
        Ok(report)
    }

    pub fn discard_pending(&self) -> Result<PendingEditDiscardReport, PlayEditResolutionError> {
        let resolution = self.begin_resolution()?;
        let report = self.pending_edits.discard();
        drop(resolution);
        Ok(report)
    }

    pub(super) fn play_start_blocker(&self) -> Option<PlayEditBeginError> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.resolving || state.decision_publishing {
            return Some(PlayEditBeginError::ResolutionInProgress);
        }
        self.pending_edits.decision_prompt().map(|prompt| {
            PlayEditBeginError::PendingDecisionRequired {
                pending_count: prompt.pending_count,
            }
        })
    }

    pub(super) fn pending_decision_prompt(&self) -> Option<PendingEditDecisionPrompt> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.resolving || state.decision_publishing {
            return None;
        }
        self.pending_edits.decision_prompt()
    }

    pub(super) fn with_pending_decision_prompt<E>(
        &self,
        publish: impl FnOnce(&PendingEditDecisionPrompt) -> Result<(), E>,
    ) -> Result<bool, E> {
        let prompt = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if state.resolving || state.decision_publishing {
                return Ok(false);
            }
            let Some(prompt) = self.pending_edits.decision_prompt() else {
                return Ok(false);
            };
            state.decision_publishing = true;
            prompt
        };
        let publication = PendingEditDecisionPublicationGuard { state: &self.state };
        let result = publish(&prompt);
        drop(publication);
        result.map(|()| true)
    }

    fn begin_resolution(&self) -> Result<PendingEditResolutionGuard<'_>, PlayEditResolutionError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.policy.is_playing() {
            return Err(PlayEditResolutionError::PlayActive);
        }
        if state.resolving || state.decision_publishing {
            return Err(PlayEditResolutionError::ResolutionInProgress);
        }
        state.resolving = true;
        Ok(PendingEditResolutionGuard { state: &self.state })
    }
}

struct PendingEditResolutionGuard<'a> {
    state: &'a Mutex<PlayEditProtectionState>,
}

impl Drop for PendingEditResolutionGuard<'_> {
    fn drop(&mut self) {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .resolving = false;
    }
}

struct PendingEditDecisionPublicationGuard<'a> {
    state: &'a Mutex<PlayEditProtectionState>,
}

impl Drop for PendingEditDecisionPublicationGuard<'_> {
    fn drop(&mut self) {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .decision_publishing = false;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlayEditRoute {
    ApplyNow {
        target: EditOperationTarget,
        invocation: EditorOperationInvocation,
    },
    Queued {
        id: PendingEditId,
        coalesced: bool,
        evicted_ids: Vec<PendingEditId>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlayEditRouteError {
    TargetMismatch {
        requested: EditOperationTarget,
        registered: EditOperationTarget,
    },
    PlayDomainUnavailable,
    RunningDocumentLocked {
        document: DocumentId,
    },
    PendingResolutionInProgress,
    PendingQueue(PendingEditQueueError),
}

impl Display for PlayEditRouteError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TargetMismatch {
                requested,
                registered,
            } => write!(
                formatter,
                "edit operation target {requested:?} does not match its registered target {registered:?}"
            ),
            Self::PlayDomainUnavailable => {
                formatter.write_str("play-domain edits require an active play session")
            }
            Self::RunningDocumentLocked { document } => write!(
                formatter,
                "edit document {} is locked by the active play session",
                document.value()
            ),
            Self::PendingResolutionInProgress => {
                formatter.write_str("pending edits are currently being resolved")
            }
            Self::PendingQueue(error) => Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for PlayEditRouteError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayEditBeginError {
    AlreadyPlaying,
    PendingDecisionRequired { pending_count: usize },
    ResolutionInProgress,
}

impl Display for PlayEditBeginError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyPlaying => formatter.write_str("play edit protection is already active"),
            Self::PendingDecisionRequired { pending_count } => write!(
                formatter,
                "{pending_count} pending edit intents require an apply or discard decision"
            ),
            Self::ResolutionInProgress => {
                formatter.write_str("pending edits are currently being resolved")
            }
        }
    }
}

impl std::error::Error for PlayEditBeginError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayEditResolutionError {
    PlayActive,
    ResolutionInProgress,
}

impl Display for PlayEditResolutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlayActive => {
                formatter.write_str("pending edits can only be resolved after play stops")
            }
            Self::ResolutionInProgress => {
                formatter.write_str("pending edits are already being resolved")
            }
        }
    }
}

impl std::error::Error for PlayEditResolutionError {}
