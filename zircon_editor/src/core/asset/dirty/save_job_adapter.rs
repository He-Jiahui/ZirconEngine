use std::collections::VecDeque;
use std::sync::Arc;

use thiserror::Error;

use crate::core::editor_message::DocumentId;
use crate::core::jobs::{
    EditorJob, EditorJobAdmissionRequest, EditorJobSpec, EditorJobSystem, JobCategory, JobContext,
    JobError, JobId, JobPriority, JobSubmitError, JobTicket, MutexGroup,
};

use super::{
    SaveDirtyViewCompletion, SaveDirtyViewFailure, SaveDirtyViewFailureKind, SaveDirtyViewIntent,
    SaveDirtyViewsRequest,
};

pub const DEFAULT_SAVE_DIRTY_VIEWS_COMPLETION_BUDGET: usize = 64;

/// Executes one already-admitted save intent through the document authority.
///
/// The implementation may resolve a weak host/service handle when the worker
/// starts, but the pending queue retains only the light intent and this shared
/// executor. Serialized document bytes never enter admission state.
pub trait SaveDirtyViewExecutor: Send + Sync + 'static {
    fn save(&self, intent: &SaveDirtyViewIntent, context: &JobContext) -> SaveDirtyViewCompletion;
}

impl<F> SaveDirtyViewExecutor for F
where
    F: Fn(&SaveDirtyViewIntent, &JobContext) -> SaveDirtyViewCompletion + Send + Sync + 'static,
{
    fn save(&self, intent: &SaveDirtyViewIntent, context: &JobContext) -> SaveDirtyViewCompletion {
        self(intent, context)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SaveDirtyViewsAdmissionError {
    #[error("interactive save adapter is shutting down and no longer accepts batches")]
    ShuttingDown,
    #[error("interactive save adapter already owns a batch in flight")]
    BatchInFlight,
    #[error("document {document:?} save mutex resolution failed: {message}")]
    SaveMutex {
        document: DocumentId,
        message: String,
    },
    #[error(transparent)]
    JobSubmit(#[from] JobSubmitError),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveDirtyViewsCompletionBatch {
    completions: Vec<SaveDirtyViewCompletionSlot>,
}

impl SaveDirtyViewsCompletionBatch {
    pub fn len(&self) -> usize {
        self.completions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.completions.is_empty()
    }

    pub fn completion(&self, document: DocumentId) -> Option<&SaveDirtyViewCompletion> {
        self.completions
            .iter()
            .find(|slot| slot.document == document)
            .and_then(|slot| slot.completion.as_ref())
    }

    pub fn into_completions(self) -> impl Iterator<Item = (DocumentId, SaveDirtyViewCompletion)> {
        self.completions.into_iter().map(|slot| {
            (
                slot.document,
                slot.completion
                    .expect("terminal save batch must contain every completion"),
            )
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SaveDirtyViewCompletionSlot {
    document: DocumentId,
    completion: Option<SaveDirtyViewCompletion>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveDirtyViewsCompletionPoll {
    inspected_tickets: usize,
    pending_tickets: usize,
    completed: Option<SaveDirtyViewsCompletionBatch>,
}

impl SaveDirtyViewsCompletionPoll {
    pub const fn inspected_tickets(&self) -> usize {
        self.inspected_tickets
    }

    pub const fn pending_tickets(&self) -> usize {
        self.pending_tickets
    }

    pub fn completed(&self) -> Option<&SaveDirtyViewsCompletionBatch> {
        self.completed.as_ref()
    }

    pub fn into_completed(self) -> Option<SaveDirtyViewsCompletionBatch> {
        self.completed
    }
}

/// Bounded domain adapter over the one `EditorJobSystem` admission owner.
pub struct SaveDirtyViewsJobAdapter {
    jobs: EditorJobSystem,
    tickets: VecDeque<(usize, JobTicket<SaveDirtyViewCompletion>)>,
    completions: Vec<SaveDirtyViewCompletionSlot>,
    accepting: bool,
}

impl SaveDirtyViewsJobAdapter {
    pub fn new(jobs: EditorJobSystem) -> Self {
        Self {
            jobs,
            tickets: VecDeque::new(),
            completions: Vec::new(),
            accepting: true,
        }
    }

    pub const fn is_accepting(&self) -> bool {
        self.accepting
    }

    pub fn is_in_flight(&self) -> bool {
        !self.tickets.is_empty() || !self.completions.is_empty()
    }

    /// Atomically reserves and then materializes a complete preflighted batch.
    ///
    /// The shared reservation claims queue capacity before save-mutex
    /// resolution, job objects, result channels, or executor calls. It rolls
    /// back automatically if any materialization step returns early.
    pub fn schedule(
        &mut self,
        request: &SaveDirtyViewsRequest,
        mut save_mutex_for: impl FnMut(&SaveDirtyViewIntent) -> Result<MutexGroup, String>,
        executor_factory: impl FnOnce() -> Arc<dyn SaveDirtyViewExecutor>,
    ) -> Result<bool, SaveDirtyViewsAdmissionError> {
        if !self.accepting {
            return Err(SaveDirtyViewsAdmissionError::ShuttingDown);
        }
        if self.is_in_flight() {
            return Err(SaveDirtyViewsAdmissionError::BatchInFlight);
        }
        if request.intents().is_empty() {
            return Ok(false);
        }

        let reservation = self.jobs.reserve_batch_admission(
            request
                .intents()
                .iter()
                .map(|intent| {
                    EditorJobAdmissionRequest::new(
                        JobCategory::InteractiveSave,
                        intent_estimated_bytes(intent),
                    )
                    .with_priority(JobPriority::Interactive)
                })
                .collect(),
        )?;

        let mut prepared = Vec::with_capacity(request.intents().len());
        for intent in request.intents().iter().cloned() {
            let document = intent.document_id();
            let save_mutex = save_mutex_for(&intent)
                .map_err(|message| SaveDirtyViewsAdmissionError::SaveMutex { document, message })?;
            prepared.push((document, intent, save_mutex));
        }

        let executor = executor_factory();
        let mut documents = Vec::with_capacity(prepared.len());
        let mut jobs = Vec::with_capacity(prepared.len());
        for (document, intent, save_mutex) in prepared {
            let spec = EditorJobSpec::new(
                format!("save_dirty_document_{}", document.value()),
                JobCategory::InteractiveSave,
            )
            .with_priority(JobPriority::Interactive)
            .with_mutex_group(save_mutex)
            .with_estimated_bytes(intent_estimated_bytes(&intent));
            documents.push(document);
            jobs.push((
                spec,
                SaveDirtyViewJob {
                    intent,
                    executor: Arc::clone(&executor),
                },
            ));
        }

        let tickets = reservation.commit(jobs)?;
        self.completions = documents
            .into_iter()
            .map(|document| SaveDirtyViewCompletionSlot {
                document,
                completion: None,
            })
            .collect();
        self.tickets = tickets.into_iter().enumerate().collect();
        Ok(true)
    }

    pub fn pump_completed(&mut self) -> SaveDirtyViewsCompletionPoll {
        self.pump_completed_with_budget(DEFAULT_SAVE_DIRTY_VIEWS_COMPLETION_BUDGET)
    }

    pub fn pump_completed_with_budget(
        &mut self,
        max_tickets: usize,
    ) -> SaveDirtyViewsCompletionPoll {
        let inspected_tickets = max_tickets.min(self.tickets.len());
        for _ in 0..inspected_tickets {
            let Some((slot_index, ticket)) = self.tickets.pop_front() else {
                break;
            };
            match ticket.try_take() {
                Some(Ok(completion)) => {
                    self.complete_slot(slot_index, completion);
                }
                Some(Err(JobError::Cancelled)) => {
                    self.complete_slot(slot_index, SaveDirtyViewCompletion::Cancelled);
                }
                Some(Err(error)) => {
                    let failure = error
                        .downcast_ref::<SaveDirtyViewFailure>()
                        .cloned()
                        .unwrap_or_else(|| {
                            SaveDirtyViewFailure::new(
                                SaveDirtyViewFailureKind::Job,
                                error.to_string(),
                            )
                        });
                    self.complete_slot(slot_index, SaveDirtyViewCompletion::Failed(failure));
                }
                None => self.tickets.push_back((slot_index, ticket)),
            }
        }

        let completed = if self.tickets.is_empty() && !self.completions.is_empty() {
            Some(SaveDirtyViewsCompletionBatch {
                completions: std::mem::take(&mut self.completions),
            })
        } else {
            None
        };
        SaveDirtyViewsCompletionPoll {
            inspected_tickets,
            pending_tickets: self.tickets.len(),
            completed,
        }
    }

    fn complete_slot(&mut self, slot_index: usize, completion: SaveDirtyViewCompletion) {
        let slot = self
            .completions
            .get_mut(slot_index)
            .expect("save ticket must retain its completion slot");
        debug_assert!(slot.completion.is_none());
        slot.completion = Some(completion);
    }

    /// Stops future batches and requests cooperative cancellation for all
    /// owned tickets. The global job-system shutdown remains the deadline owner.
    pub fn begin_shutdown(&mut self) -> Vec<JobId> {
        self.accepting = false;
        let ids = self
            .tickets
            .iter()
            .map(|(_, ticket)| ticket.id())
            .collect::<Vec<_>>();
        for id in &ids {
            self.jobs.cancel(*id);
        }
        ids
    }
}

fn intent_estimated_bytes(intent: &SaveDirtyViewIntent) -> usize {
    usize::try_from(intent.estimated_bytes())
        .unwrap_or(usize::MAX)
        .max(1)
}

struct SaveDirtyViewJob {
    intent: SaveDirtyViewIntent,
    executor: Arc<dyn SaveDirtyViewExecutor>,
}

impl EditorJob for SaveDirtyViewJob {
    type Output = SaveDirtyViewCompletion;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        match self.executor.save(&self.intent, &context) {
            SaveDirtyViewCompletion::Saved { written_bytes } => {
                Ok(SaveDirtyViewCompletion::Saved { written_bytes })
            }
            SaveDirtyViewCompletion::Failed(failure) => Err(JobError::failed(failure)),
            SaveDirtyViewCompletion::Cancelled => Err(JobError::Cancelled),
        }
    }
}

#[cfg(test)]
mod tests;
