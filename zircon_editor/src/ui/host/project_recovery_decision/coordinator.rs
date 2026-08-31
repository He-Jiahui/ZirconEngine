use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use thiserror::Error;

use crate::core::notifications::{
    DecisionNotification, DecisionNotificationCenter, DecisionNotificationError, DecisionOption,
    DecisionOptionId, DecisionReceiptCursor, DecisionTicket, NotificationId,
    NotificationIdentityError, NotificationSource, MAX_DECISION_DISPLAY_SUBJECT_BYTES,
};
use crate::core::recovery::{
    RestoreAction, RestoreCandidate, RestoreFlow, RestoreFlowError, RestoreResolution,
    RestoreStartup,
};

use super::model::RecoveryRestoreWork;

const RECOVERY_NOTIFICATION_ID_PREFIX: &str = "editor.recovery.candidate";
const RECOVERY_NOTIFICATION_SOURCE: &str = "editor.recovery";
const RECOVERY_DECISION_TITLE_KEY: &str = "editor.recovery.decision.title";
const RECOVERY_DECISION_MESSAGE_KEY: &str = "editor.recovery.decision.message";
const RECOVERY_RESTORE_OPTION_ID: &str = "restore";
const RECOVERY_DISCARD_OPTION_ID: &str = "discard";
const RECOVERY_COMPARE_OPTION_ID: &str = "compare";
const RECOVERY_RESTORE_OPTION_LABEL_KEY: &str = "editor.recovery.decision.restore";
const RECOVERY_DISCARD_OPTION_LABEL_KEY: &str = "editor.recovery.decision.discard";
const RECOVERY_COMPARE_OPTION_LABEL_KEY: &str = "editor.recovery.decision.compare";

/// Project-session domain owner for recovery choices published through the generic Decision
/// center. It deliberately holds at most one candidate ticket, so a large residual catalog cannot
/// consume the shared center's bounded pending capacity.
pub(super) struct ProjectRecoveryDecisionCoordinator {
    state: Mutex<ProjectRecoveryDecisionState>,
    next_notification_sequence: AtomicU64,
}

impl Default for ProjectRecoveryDecisionCoordinator {
    fn default() -> Self {
        Self {
            state: Mutex::new(ProjectRecoveryDecisionState::Idle),
            next_notification_sequence: AtomicU64::new(1),
        }
    }
}

enum ProjectRecoveryDecisionState {
    Idle,
    Collecting(RecoveryCollection),
    Completed,
}

struct RecoveryCollection {
    project_root: PathBuf,
    startup: RestoreStartup,
    cursor: DecisionReceiptCursor,
    next_candidate_index: usize,
    pending_ticket: Option<DecisionTicket>,
    publication_blocked_at_pending_count: Option<usize>,
    resolutions: Vec<RestoreResolution>,
}

impl ProjectRecoveryDecisionCoordinator {
    /// Starts recovery decision collection after the manager has acquired and committed the new
    /// project session. A no-candidate startup intentionally leaves the coordinator idle.
    pub(super) fn begin(
        &self,
        center: &DecisionNotificationCenter,
        project_root: impl AsRef<Path>,
        startup: RestoreStartup,
    ) -> Result<bool, ProjectRecoveryDecisionError> {
        if startup.candidates().is_empty() {
            return Ok(false);
        }
        let mut state = self.lock_state();
        if matches!(*state, ProjectRecoveryDecisionState::Collecting(_)) {
            return Err(ProjectRecoveryDecisionError::RecoveryAlreadyActive);
        }
        *state = ProjectRecoveryDecisionState::Collecting(RecoveryCollection {
            project_root: project_root.as_ref().to_path_buf(),
            startup,
            cursor: center.initial_cursor(),
            next_candidate_index: 0,
            pending_ticket: None,
            publication_blocked_at_pending_count: None,
            resolutions: Vec::new(),
        });
        Ok(true)
    }

    /// Consumes one explicit core receipt and publishes the next candidate when capacity permits.
    /// It returns a work item exactly once, after `RestoreFlow` validates a complete resolution
    /// set. Disk I/O is intentionally deferred to the host-owned job adapter.
    pub(super) fn pump(
        &self,
        center: &DecisionNotificationCenter,
    ) -> Result<Option<RecoveryRestoreWork>, ProjectRecoveryDecisionError> {
        let mut state = self.lock_state();
        let ProjectRecoveryDecisionState::Collecting(collection) = &mut *state else {
            return Ok(None);
        };

        if collection.pending_ticket.is_some() {
            self.consume_pending_receipt(collection, center)?;
        }
        if collection.pending_ticket.is_some() {
            return Ok(None);
        }

        if collection.next_candidate_index == collection.startup.candidates().len() {
            let plan = RestoreFlow::plan(
                &collection.startup,
                std::mem::take(&mut collection.resolutions),
            )?;
            let work = RecoveryRestoreWork::new(
                collection.project_root.clone(),
                collection.startup.clone(),
                plan,
            );
            *state = ProjectRecoveryDecisionState::Completed;
            return Ok(Some(work));
        }

        if collection.publication_blocked_at_pending_count == Some(center.pending_count()) {
            return Ok(None);
        }
        self.publish_current_candidate(collection, center)?;
        Ok(None)
    }

    pub(super) fn is_active(&self) -> bool {
        matches!(
            *self.lock_state(),
            ProjectRecoveryDecisionState::Collecting(_)
        )
    }

    fn consume_pending_receipt(
        &self,
        collection: &mut RecoveryCollection,
        center: &DecisionNotificationCenter,
    ) -> Result<(), ProjectRecoveryDecisionError> {
        let ticket = collection
            .pending_ticket
            .clone()
            .ok_or(ProjectRecoveryDecisionError::MissingPendingTicket)?;
        let (batch, cursor_expired) = match center.receipts_since(collection.cursor) {
            Ok(batch) => (batch, false),
            Err(DecisionNotificationError::CursorExpired { resume_cursor, .. }) => {
                (center.receipts_since(resume_cursor)?, true)
            }
            Err(error) => return Err(error.into()),
        };
        collection.cursor = batch.next_cursor();
        if let Some(receipt) = batch
            .receipts()
            .iter()
            .find(|receipt| receipt.ticket() == &ticket)
        {
            let candidate = self.current_candidate(collection)?.clone();
            collection.resolutions.push(RestoreResolution::new(
                candidate.document().clone(),
                restore_action_for(receipt.option_id())?,
            ));
            collection.next_candidate_index = collection.next_candidate_index.saturating_add(1);
            collection.pending_ticket = None;
            return Ok(());
        }

        if cursor_expired
            && !center
                .pending_snapshot()
                .iter()
                .any(|snapshot| snapshot.ticket() == &ticket)
        {
            // The bounded receipt journal no longer proves a choice for this ticket. Re-prompt
            // the same candidate instead of inferring a destructive recovery action.
            collection.pending_ticket = None;
        }
        Ok(())
    }

    fn publish_current_candidate(
        &self,
        collection: &mut RecoveryCollection,
        center: &DecisionNotificationCenter,
    ) -> Result<(), ProjectRecoveryDecisionError> {
        let candidate = self.current_candidate(collection)?;
        let notification = recovery_notification(
            next_notification_id(self.next_notification_sequence()?)?,
            &collection.project_root,
            candidate,
        )?;
        match center.publish(notification) {
            Ok(ticket) => {
                collection.pending_ticket = Some(ticket);
                collection.publication_blocked_at_pending_count = None;
            }
            Err(DecisionNotificationError::PendingCapacityReached { .. }) => {
                collection.publication_blocked_at_pending_count = Some(center.pending_count());
            }
            Err(error) => return Err(error.into()),
        }
        Ok(())
    }

    fn next_notification_sequence(&self) -> Result<u64, ProjectRecoveryDecisionError> {
        self.next_notification_sequence
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |sequence| {
                sequence.checked_add(1)
            })
            .map_err(|_| ProjectRecoveryDecisionError::NotificationSequenceExhausted)
    }

    fn current_candidate<'a>(
        &self,
        collection: &'a RecoveryCollection,
    ) -> Result<&'a RestoreCandidate, ProjectRecoveryDecisionError> {
        collection
            .startup
            .candidates()
            .get(collection.next_candidate_index)
            .ok_or(ProjectRecoveryDecisionError::CandidateIndexOutOfRange {
                candidate_index: collection.next_candidate_index,
                candidate_count: collection.startup.candidates().len(),
            })
    }

    fn lock_state(&self) -> std::sync::MutexGuard<'_, ProjectRecoveryDecisionState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn next_notification_id(sequence: u64) -> Result<NotificationId, ProjectRecoveryDecisionError> {
    NotificationId::parse(format!("{RECOVERY_NOTIFICATION_ID_PREFIX}.{sequence}"))
        .map_err(ProjectRecoveryDecisionError::NotificationIdentity)
}

fn recovery_notification(
    notification_id: NotificationId,
    project_root: &Path,
    candidate: &RestoreCandidate,
) -> Result<DecisionNotification, ProjectRecoveryDecisionError> {
    let restore = DecisionOptionId::parse(RECOVERY_RESTORE_OPTION_ID)?;
    let discard = DecisionOptionId::parse(RECOVERY_DISCARD_OPTION_ID)?;
    let compare = DecisionOptionId::parse(RECOVERY_COMPARE_OPTION_ID)?;
    let source = NotificationSource::builtin(RECOVERY_NOTIFICATION_SOURCE)
        .map_err(ProjectRecoveryDecisionError::NotificationIdentity)?;
    DecisionNotification::new(
        notification_id,
        source,
        RECOVERY_DECISION_TITLE_KEY,
        RECOVERY_DECISION_MESSAGE_KEY,
        vec![
            DecisionOption::new(restore, RECOVERY_RESTORE_OPTION_LABEL_KEY)?,
            DecisionOption::new(discard, RECOVERY_DISCARD_OPTION_LABEL_KEY)?,
            DecisionOption::new(compare, RECOVERY_COMPARE_OPTION_LABEL_KEY)?,
        ],
    )?
    .with_display_subject(candidate_display_subject(project_root, candidate)?)
    .map_err(ProjectRecoveryDecisionError::from)
}

fn candidate_display_subject(
    project_root: &Path,
    candidate: &RestoreCandidate,
) -> Result<String, ProjectRecoveryDecisionError> {
    let relative = candidate
        .source_path()
        .strip_prefix(project_root)
        .map_err(
            |_| ProjectRecoveryDecisionError::CandidateSourceOutsideProject {
                document: candidate.document().as_str().to_string(),
            },
        )?;
    let relative = relative.to_string_lossy();
    if !relative.is_empty() && relative.len() <= MAX_DECISION_DISPLAY_SUBJECT_BYTES {
        return Ok(relative.into_owned());
    }
    let file_name = relative
        .rsplit(['/', '\\'])
        .next()
        .filter(|file_name| !file_name.is_empty());
    if let Some(file_name) =
        file_name.filter(|file_name| file_name.len() <= MAX_DECISION_DISPLAY_SUBJECT_BYTES)
    {
        return Ok(file_name.to_string());
    }
    Ok(candidate.document().as_str().to_string())
}

fn restore_action_for(
    option: &DecisionOptionId,
) -> Result<RestoreAction, ProjectRecoveryDecisionError> {
    match option.as_str() {
        RECOVERY_RESTORE_OPTION_ID => Ok(RestoreAction::RestoreAutosave),
        RECOVERY_DISCARD_OPTION_ID => Ok(RestoreAction::DiscardAutosave),
        RECOVERY_COMPARE_OPTION_ID => Ok(RestoreAction::OpenComparison),
        _ => Err(ProjectRecoveryDecisionError::UnexpectedReceiptOption {
            option: option.as_str().to_string(),
        }),
    }
}

#[derive(Debug, Error)]
pub(super) enum ProjectRecoveryDecisionError {
    #[error(transparent)]
    Decision(#[from] DecisionNotificationError),
    #[error(transparent)]
    NotificationIdentity(#[from] NotificationIdentityError),
    #[error(transparent)]
    RestoreFlow(#[from] RestoreFlowError),
    #[error("project recovery is already collecting operator decisions")]
    RecoveryAlreadyActive,
    #[error("project recovery notification sequence is exhausted")]
    NotificationSequenceExhausted,
    #[error("project recovery is missing its current decision ticket")]
    MissingPendingTicket,
    #[error(
        "project recovery candidate index {candidate_index} is outside the {candidate_count} candidate collection"
    )]
    CandidateIndexOutOfRange {
        candidate_index: usize,
        candidate_count: usize,
    },
    #[error("recovery candidate `{document}` has a source outside the active project")]
    CandidateSourceOutsideProject { document: String },
    #[error("recovery decision receipt selected unknown option `{option}`")]
    UnexpectedReceiptOption { option: String },
}
