use std::collections::BTreeSet;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use super::request::{
    ScriptBuildCompletion, ScriptBuildOutcome, ScriptBuildRequest, ScriptBuildRequestId,
    ScriptBuildStepDispatch, ScriptBuildTrigger,
};

pub const DEFAULT_SCRIPT_WATCH_DEBOUNCE_MS: u64 = 300;
pub const DEFAULT_SCRIPT_WATCH_MAX_LATENCY_MS: u64 = 1_000;
pub const MAX_INCREMENTAL_SCRIPT_WATCH_PATHS: usize = 20;
pub const MAX_INCREMENTAL_SCRIPT_WATCH_PATH_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptBuildPhase {
    Idle,
    Debouncing,
    Queued,
    Building,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScriptBuildSnapshot {
    phase: ScriptBuildPhase,
    active_request_id: Option<ScriptBuildRequestId>,
    queued_request_count: usize,
    pending_watch_path_count: usize,
    watch_first_observed_at_ms: Option<u64>,
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

    pub fn watch_first_observed_at_ms(&self) -> Option<u64> {
        self.watch_first_observed_at_ms
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
    max_latency_ms: u64,
    next_request_id: u64,
    pending_watch_paths: BTreeSet<PathBuf>,
    pending_watch_path_bytes: usize,
    pending_watch_requires_full_rebuild: bool,
    watch_first_observed_at_ms: Option<u64>,
    watch_deadline_ms: Option<u64>,
    queued_request: Option<ScriptBuildRequest>,
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
        Self::with_debounce_limits(
            debounce_ms,
            DEFAULT_SCRIPT_WATCH_MAX_LATENCY_MS.max(debounce_ms),
        )
    }

    pub fn with_debounce_limits(debounce_ms: u64, max_latency_ms: u64) -> Self {
        Self {
            debounce_ms,
            max_latency_ms: max_latency_ms.max(debounce_ms),
            next_request_id: 1,
            pending_watch_paths: BTreeSet::new(),
            pending_watch_path_bytes: 0,
            pending_watch_requires_full_rebuild: false,
            watch_first_observed_at_ms: None,
            watch_deadline_ms: None,
            queued_request: None,
            active_request: None,
            last_outcome: None,
        }
    }

    pub fn notify_watch_change(&mut self, path: impl Into<PathBuf>, observed_at_ms: u64) {
        if !self.pending_watch_requires_full_rebuild
            && !try_insert_incremental_path(
                &mut self.pending_watch_paths,
                &mut self.pending_watch_path_bytes,
                path.into(),
            )
        {
            self.pending_watch_paths.clear();
            self.pending_watch_path_bytes = 0;
            self.pending_watch_requires_full_rebuild = true;
        }
        let first_observed_at_ms = self
            .watch_first_observed_at_ms
            .map_or(observed_at_ms, |first| first.min(observed_at_ms));
        self.watch_first_observed_at_ms = Some(first_observed_at_ms);
        let hard_deadline_ms = first_observed_at_ms.saturating_add(self.max_latency_ms);
        self.watch_deadline_ms = Some(
            observed_at_ms
                .saturating_add(self.debounce_ms)
                .min(hard_deadline_ms),
        );
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
            let Some(request) = self.queued_request.take() else {
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
        let generation = active.request.generation();
        let request_completed =
            outcome.is_succeeded() && completed_step_index + 1 == active.request.step_count();
        let resume_play = request_completed && active.request.play_after_build();
        let (dropped_queued_request_count, dropped_watch_path_count) = if !outcome.is_succeeded() {
            self.active_request.take();
            let dropped_queued_request_count = usize::from(self.queued_request.take().is_some());
            let dropped_watch_path_count = self.pending_watch_path_count();
            self.pending_watch_paths.clear();
            self.pending_watch_path_bytes = 0;
            self.pending_watch_requires_full_rebuild = false;
            self.watch_first_observed_at_ms = None;
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
            generation,
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
            queued_request_count: usize::from(self.queued_request.is_some()),
            pending_watch_path_count: self.pending_watch_path_count(),
            watch_first_observed_at_ms: self.watch_first_observed_at_ms,
            watch_deadline_ms: self.watch_deadline_ms,
            last_outcome: self.last_outcome.clone(),
        }
    }

    fn phase(&self) -> ScriptBuildPhase {
        if self.active_request.is_some() {
            ScriptBuildPhase::Building
        } else if self.queued_request.is_some() {
            ScriptBuildPhase::Queued
        } else if self.has_pending_watch_changes() {
            ScriptBuildPhase::Debouncing
        } else {
            match self.last_outcome.as_deref() {
                Some(ScriptBuildOutcome::Succeeded) => ScriptBuildPhase::Succeeded,
                Some(ScriptBuildOutcome::Failed { .. }) => ScriptBuildPhase::Failed,
                Some(ScriptBuildOutcome::Cancelled { .. }) => ScriptBuildPhase::Cancelled,
                None => ScriptBuildPhase::Idle,
            }
        }
    }

    fn enqueue_explicit(
        &mut self,
        trigger: ScriptBuildTrigger,
    ) -> Result<ScriptBuildRequestId, ScriptBuildEnqueueError> {
        if self.has_pending_watch_changes() {
            if self.queued_request.is_some() {
                let changed_paths = self.take_pending_watch_paths();
                return Ok(self.merge_queued_request(trigger, changed_paths));
            }
            let request_id = self.reserve_request_id()?;
            let changed_paths = self.take_pending_watch_paths();
            self.queued_request = Some(ScriptBuildRequest::new(request_id, trigger, changed_paths));
            return Ok(request_id);
        }
        if let Some(queued_request) = self.queued_request.as_mut() {
            queued_request.promote_trigger(trigger);
            return Ok(queued_request.id());
        }
        if let Some(active_request) = self.active_request.as_mut() {
            active_request.request.promote_trigger(trigger);
            return Ok(active_request.request.id());
        }
        let request_id = self.reserve_request_id()?;
        self.queued_request = Some(ScriptBuildRequest::new(request_id, trigger, Vec::new()));
        Ok(request_id)
    }

    fn flush_watch_if_due(&mut self, now_ms: u64) -> Result<(), ScriptBuildEnqueueError> {
        if !self
            .watch_deadline_ms
            .is_some_and(|deadline_ms| now_ms >= deadline_ms)
        {
            return Ok(());
        }
        if self.queued_request.is_some() {
            let changed_paths = self.take_pending_watch_paths();
            self.merge_queued_request(ScriptBuildTrigger::Watch, changed_paths);
            return Ok(());
        }
        let request_id = self.reserve_request_id()?;
        let changed_paths = self.take_pending_watch_paths();
        self.queued_request = Some(ScriptBuildRequest::new(
            request_id,
            ScriptBuildTrigger::Watch,
            changed_paths,
        ));
        Ok(())
    }

    fn take_pending_watch_paths(&mut self) -> Vec<PathBuf> {
        self.watch_first_observed_at_ms = None;
        self.watch_deadline_ms = None;
        self.pending_watch_path_bytes = 0;
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

    fn merge_queued_request(
        &mut self,
        trigger: ScriptBuildTrigger,
        changed_paths: Vec<PathBuf>,
    ) -> ScriptBuildRequestId {
        let merged_paths = {
            let queued_request = self
                .queued_request
                .as_ref()
                .expect("a queued request exists before it is merged");
            merge_incremental_paths(queued_request.changed_paths(), changed_paths)
        };
        let queued_request = self
            .queued_request
            .as_mut()
            .expect("a queued request exists before it is merged");
        queued_request.promote_trigger(trigger);
        queued_request.replace_changed_paths(merged_paths);
        queued_request.id()
    }

    #[cfg(test)]
    pub(super) fn exhaust_request_ids(&mut self) {
        self.next_request_id = u64::MAX;
    }

    #[cfg(test)]
    pub(super) const fn pending_watch_path_bytes_for_test(&self) -> usize {
        self.pending_watch_path_bytes
    }
}

fn try_insert_incremental_path(
    paths: &mut BTreeSet<PathBuf>,
    path_bytes: &mut usize,
    path: PathBuf,
) -> bool {
    let added_bytes = path.as_os_str().len();
    if !paths.insert(path) {
        return true;
    }
    let Some(next_bytes) = path_bytes.checked_add(added_bytes) else {
        return false;
    };
    *path_bytes = next_bytes;
    paths.len() <= MAX_INCREMENTAL_SCRIPT_WATCH_PATHS
        && next_bytes <= MAX_INCREMENTAL_SCRIPT_WATCH_PATH_BYTES
}

pub(super) fn merge_incremental_paths(
    current_paths: &[PathBuf],
    incoming_paths: Vec<PathBuf>,
) -> Vec<PathBuf> {
    if current_paths.is_empty() || incoming_paths.is_empty() {
        return Vec::new();
    }
    let mut merged_paths = BTreeSet::new();
    let mut merged_path_bytes = 0;
    for path in current_paths.iter().cloned().chain(incoming_paths) {
        if !try_insert_incremental_path(&mut merged_paths, &mut merged_path_bytes, path) {
            return Vec::new();
        }
    }
    merged_paths.into_iter().collect()
}
