use std::collections::{BTreeSet, VecDeque};
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use super::request::{
    ScriptBuildCompletion, ScriptBuildOutcome, ScriptBuildRequest, ScriptBuildRequestId,
    ScriptBuildStepDispatch, ScriptBuildTrigger,
};

pub const DEFAULT_SCRIPT_WATCH_DEBOUNCE_MS: u64 = 300;
pub const MAX_INCREMENTAL_SCRIPT_WATCH_PATHS: usize = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptBuildPhase {
    Idle,
    Debouncing,
    Queued,
    Building,
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScriptBuildSnapshot {
    phase: ScriptBuildPhase,
    active_request_id: Option<ScriptBuildRequestId>,
    queued_request_count: usize,
    pending_watch_path_count: usize,
    watch_deadline_ms: Option<u64>,
    last_outcome: Option<Arc<ScriptBuildOutcome>>,
}

impl ScriptBuildSnapshot {
    pub fn phase(&self) -> ScriptBuildPhase {
        self.phase
    }

    pub fn active_request_id(&self) -> Option<ScriptBuildRequestId> {
        self.active_request_id
    }

    pub fn queued_request_count(&self) -> usize {
        self.queued_request_count
    }

    pub fn pending_watch_path_count(&self) -> usize {
        self.pending_watch_path_count
    }

    pub fn watch_deadline_ms(&self) -> Option<u64> {
        self.watch_deadline_ms
    }

    pub fn last_outcome(&self) -> Option<&ScriptBuildOutcome> {
        self.last_outcome.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScriptBuildCompletionError {
    NoActiveRequest {
        actual: ScriptBuildRequestId,
    },
    RequestMismatch {
        expected: ScriptBuildRequestId,
        actual: ScriptBuildRequestId,
    },
    StepMismatch {
        request_id: ScriptBuildRequestId,
        expected: usize,
        actual: usize,
    },
    NoStepInFlight {
        request_id: ScriptBuildRequestId,
    },
}

impl fmt::Display for ScriptBuildCompletionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoActiveRequest { actual } => write!(
                formatter,
                "script build request {} completed with no active request",
                actual.get()
            ),
            Self::RequestMismatch { expected, actual } => write!(
                formatter,
                "script build request {} completed while request {} is active",
                actual.get(),
                expected.get()
            ),
            Self::StepMismatch {
                request_id,
                expected,
                actual,
            } => write!(
                formatter,
                "script build request {} step {} completed while step {} is active",
                request_id.get(),
                actual,
                expected
            ),
            Self::NoStepInFlight { request_id } => write!(
                formatter,
                "script build request {} has no step in flight",
                request_id.get()
            ),
        }
    }
}

impl std::error::Error for ScriptBuildCompletionError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptBuildEnqueueError {
    RequestIdExhausted,
}

impl fmt::Display for ScriptBuildEnqueueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestIdExhausted => {
                formatter.write_str("script build request id space is exhausted")
            }
        }
    }
}

impl std::error::Error for ScriptBuildEnqueueError {}

struct ActiveScriptBuild {
    request: ScriptBuildRequest,
    step_index: usize,
    step_in_flight: bool,
}

pub struct ScriptBuildOrchestrator {
    debounce_ms: u64,
    next_request_id: u64,
    pending_watch_paths: BTreeSet<PathBuf>,
    pending_watch_requires_full_rebuild: bool,
    watch_deadline_ms: Option<u64>,
    queued_requests: VecDeque<ScriptBuildRequest>,
    active_request: Option<ActiveScriptBuild>,
    last_outcome: Option<Arc<ScriptBuildOutcome>>,
}

impl Default for ScriptBuildOrchestrator {
    fn default() -> Self {
        Self::new(DEFAULT_SCRIPT_WATCH_DEBOUNCE_MS)
    }
}

impl ScriptBuildOrchestrator {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            debounce_ms,
            next_request_id: 1,
            pending_watch_paths: BTreeSet::new(),
            pending_watch_requires_full_rebuild: false,
            watch_deadline_ms: None,
            queued_requests: VecDeque::new(),
            active_request: None,
            last_outcome: None,
        }
    }

    pub fn notify_watch_change(&mut self, path: impl Into<PathBuf>, observed_at_ms: u64) {
        if !self.pending_watch_requires_full_rebuild {
            self.pending_watch_paths.insert(path.into());
            if self.pending_watch_paths.len() > MAX_INCREMENTAL_SCRIPT_WATCH_PATHS {
                self.pending_watch_paths.clear();
                self.pending_watch_requires_full_rebuild = true;
            }
        }
        self.watch_deadline_ms = Some(observed_at_ms.saturating_add(self.debounce_ms));
    }

    pub fn enqueue_command(&mut self) -> Result<ScriptBuildRequestId, ScriptBuildEnqueueError> {
        self.enqueue_explicit(ScriptBuildTrigger::Command)
    }

    pub fn enqueue_play(&mut self) -> Result<ScriptBuildRequestId, ScriptBuildEnqueueError> {
        self.enqueue_explicit(ScriptBuildTrigger::Play)
    }

    pub fn take_ready(
        &mut self,
        now_ms: u64,
    ) -> Result<Option<ScriptBuildStepDispatch>, ScriptBuildEnqueueError> {
        self.flush_watch_if_due(now_ms)?;
        if self.active_request.is_none() {
            let Some(request) = self.queued_requests.pop_front() else {
                return Ok(None);
            };
            self.active_request = Some(ActiveScriptBuild {
                request,
                step_index: 0,
                step_in_flight: false,
            });
        }
        let Some(active) = self.active_request.as_mut() else {
            return Ok(None);
        };
        if active.step_in_flight {
            return Ok(None);
        }
        active.step_in_flight = true;
        Ok(Some(ScriptBuildStepDispatch::new(
            &active.request,
            active.step_index,
        )))
    }

    pub fn complete(
        &mut self,
        dispatch: ScriptBuildStepDispatch,
        outcome: ScriptBuildOutcome,
    ) -> Result<ScriptBuildCompletion, ScriptBuildCompletionError> {
        let request_id = dispatch.request_id();
        let Some(active) = self.active_request.as_ref() else {
            return Err(ScriptBuildCompletionError::NoActiveRequest { actual: request_id });
        };
        if active.request.id() != request_id {
            return Err(ScriptBuildCompletionError::RequestMismatch {
                expected: active.request.id(),
                actual: request_id,
            });
        }
        if active.step_index != dispatch.step_index() {
            return Err(ScriptBuildCompletionError::StepMismatch {
                request_id,
                expected: active.step_index,
                actual: dispatch.step_index(),
            });
        }
        if !active.step_in_flight {
            return Err(ScriptBuildCompletionError::NoStepInFlight { request_id });
        }

        let completed_step_index = active.step_index;
        let request_completed =
            outcome.is_succeeded() && completed_step_index + 1 == active.request.step_count();
        let resume_play = request_completed && active.request.play_after_build();
        let (dropped_queued_request_count, dropped_watch_path_count) = if !outcome.is_succeeded() {
            self.active_request.take();
            let dropped_queued_request_count = self.queued_requests.len();
            let dropped_watch_path_count = self.pending_watch_path_count();
            self.queued_requests.clear();
            self.pending_watch_paths.clear();
            self.pending_watch_requires_full_rebuild = false;
            self.watch_deadline_ms = None;
            self.last_outcome = Some(Arc::new(outcome.clone()));
            (dropped_queued_request_count, dropped_watch_path_count)
        } else if request_completed {
            self.active_request.take();
            self.last_outcome = Some(Arc::new(outcome.clone()));
            (0, 0)
        } else {
            let Some(active) = self.active_request.as_mut() else {
                return Err(ScriptBuildCompletionError::NoActiveRequest { actual: request_id });
            };
            active.step_index += 1;
            active.step_in_flight = false;
            (0, 0)
        };

        Ok(ScriptBuildCompletion::new(
            request_id,
            completed_step_index,
            outcome,
            request_completed,
            dropped_queued_request_count,
            dropped_watch_path_count,
            resume_play,
        ))
    }

    pub fn snapshot(&self) -> ScriptBuildSnapshot {
        ScriptBuildSnapshot {
            phase: self.phase(),
            active_request_id: self
                .active_request
                .as_ref()
                .map(|active| active.request.id()),
            queued_request_count: self.queued_requests.len(),
            pending_watch_path_count: self.pending_watch_path_count(),
            watch_deadline_ms: self.watch_deadline_ms,
            last_outcome: self.last_outcome.clone(),
        }
    }

    fn phase(&self) -> ScriptBuildPhase {
        if self.active_request.is_some() {
            ScriptBuildPhase::Building
        } else if !self.queued_requests.is_empty() {
            ScriptBuildPhase::Queued
        } else if self.has_pending_watch_changes() {
            ScriptBuildPhase::Debouncing
        } else {
            match self.last_outcome.as_deref() {
                Some(ScriptBuildOutcome::Succeeded) => ScriptBuildPhase::Succeeded,
                Some(ScriptBuildOutcome::Failed { .. }) => ScriptBuildPhase::Failed,
                None => ScriptBuildPhase::Idle,
            }
        }
    }

    fn enqueue_explicit(
        &mut self,
        trigger: ScriptBuildTrigger,
    ) -> Result<ScriptBuildRequestId, ScriptBuildEnqueueError> {
        let request_id = self.reserve_request_id()?;
        let changed_paths = self.take_pending_watch_paths();
        self.enqueue_request(request_id, trigger, changed_paths);
        Ok(request_id)
    }

    fn flush_watch_if_due(&mut self, now_ms: u64) -> Result<(), ScriptBuildEnqueueError> {
        if !self
            .watch_deadline_ms
            .is_some_and(|deadline_ms| now_ms >= deadline_ms)
        {
            return Ok(());
        }
        let request_id = self.reserve_request_id()?;
        let changed_paths = self.take_pending_watch_paths();
        self.enqueue_request(request_id, ScriptBuildTrigger::Watch, changed_paths);
        Ok(())
    }

    fn take_pending_watch_paths(&mut self) -> Vec<PathBuf> {
        self.watch_deadline_ms = None;
        if self.pending_watch_requires_full_rebuild {
            self.pending_watch_requires_full_rebuild = false;
            return Vec::new();
        }
        std::mem::take(&mut self.pending_watch_paths)
            .into_iter()
            .collect()
    }

    fn has_pending_watch_changes(&self) -> bool {
        self.pending_watch_requires_full_rebuild || !self.pending_watch_paths.is_empty()
    }

    fn pending_watch_path_count(&self) -> usize {
        if self.pending_watch_requires_full_rebuild {
            MAX_INCREMENTAL_SCRIPT_WATCH_PATHS + 1
        } else {
            self.pending_watch_paths.len()
        }
    }

    fn reserve_request_id(&mut self) -> Result<ScriptBuildRequestId, ScriptBuildEnqueueError> {
        let next_request_id = self
            .next_request_id
            .checked_add(1)
            .ok_or(ScriptBuildEnqueueError::RequestIdExhausted)?;
        let request_id = ScriptBuildRequestId::new(self.next_request_id);
        self.next_request_id = next_request_id;
        Ok(request_id)
    }

    fn enqueue_request(
        &mut self,
        request_id: ScriptBuildRequestId,
        trigger: ScriptBuildTrigger,
        changed_paths: Vec<PathBuf>,
    ) {
        self.queued_requests
            .push_back(ScriptBuildRequest::new(request_id, trigger, changed_paths));
    }

    #[cfg(test)]
    pub(super) fn exhaust_request_ids(&mut self) {
        self.next_request_id = u64::MAX;
    }
}
