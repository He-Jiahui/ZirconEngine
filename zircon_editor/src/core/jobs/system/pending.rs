use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::mpsc::SyncSender;
use std::time::{Duration, Instant};

use super::super::{
    EditorJob, EditorJobAdmissionKey, EditorJobAdmissionLimits, EditorJobAdmissionSnapshot,
    EditorJobLimits, EditorJobSpec, JobCategory, JobContext, JobError, JobEventKind, JobId,
    JobPriority, JobSubmitError,
};

pub(super) type PendingCancelTask = Box<dyn FnOnce(JobContext) + Send + 'static>;

pub(super) trait PendingTask: Any + Send {
    fn run(self: Box<Self>, context: JobContext);
    fn replace_with(&mut self, latest: Box<dyn PendingTask>) -> bool;
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send>;
}

impl<F> PendingTask for F
where
    F: FnOnce(JobContext) + Send + 'static,
{
    fn run(self: Box<Self>, context: JobContext) {
        (*self)(context);
    }

    fn replace_with(&mut self, _latest: Box<dyn PendingTask>) -> bool {
        false
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }
}

pub(super) struct LatestPendingTask<J>
where
    J: EditorJob,
{
    job: J,
    sender: SyncSender<Result<J::Output, JobError>>,
}

impl<J> LatestPendingTask<J>
where
    J: EditorJob,
{
    pub(super) fn new(job: J, sender: SyncSender<Result<J::Output, JobError>>) -> Self {
        Self { job, sender }
    }
}

impl<J> PendingTask for LatestPendingTask<J>
where
    J: EditorJob,
{
    fn run(self: Box<Self>, context: JobContext) {
        let Self { job, sender } = *self;
        let event_context = context.clone();
        let result = if context.is_cancelled() {
            Err(JobError::Cancelled)
        } else {
            catch_unwind(AssertUnwindSafe(|| job.run(context)))
                .unwrap_or_else(|payload| Err(JobError::Panicked(super::panic_message(payload))))
        };
        let kind = match &result {
            Ok(_) => JobEventKind::Completed,
            Err(JobError::Cancelled) => JobEventKind::Cancelled,
            Err(error) => JobEventKind::Failed {
                message: error.to_string(),
            },
        };
        event_context.emit(kind);
        let _ = sender.send(result);
    }

    fn replace_with(&mut self, latest: Box<dyn PendingTask>) -> bool {
        let Ok(latest) = latest.into_any().downcast::<Self>() else {
            return false;
        };
        self.job = latest.job;
        true
    }

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send> {
        self
    }
}

pub(super) const MAX_ADMISSION_BUCKET_PROBES_PER_PASS: usize =
    JobPriority::ALL.len() * JobCategory::ALL.len();

pub(super) struct PendingJob {
    pub(super) id: JobId,
    pub(super) spec: EditorJobSpec,
    pub(super) task: Box<dyn PendingTask>,
    pub(super) cancel_task: PendingCancelTask,
    admitted_at: Instant,
    estimated_bytes: usize,
}

impl PendingJob {
    pub(super) fn new(
        id: JobId,
        spec: EditorJobSpec,
        task: Box<dyn PendingTask>,
        cancel_task: PendingCancelTask,
        admitted_at: Instant,
    ) -> Self {
        let estimated_bytes = spec.estimated_pending_bytes;
        Self {
            id,
            spec,
            task,
            cancel_task,
            admitted_at,
            estimated_bytes,
        }
    }

    fn has_compatible_admission(&self, latest: &EditorJobSpec) -> bool {
        self.spec.category == latest.category
            && self.spec.priority == latest.priority
            && self.spec.mutex_group == latest.mutex_group
            && self.spec.after == latest.after
    }

    fn replace_with_latest(&mut self, latest: &EditorJobSpec, task: Box<dyn PendingTask>) -> bool {
        if !self.task.replace_with(task) {
            return false;
        }
        self.spec.estimated_pending_bytes = latest.estimated_pending_bytes;
        self.spec.max_pending_age = latest.max_pending_age;
        self.estimated_bytes = latest.estimated_pending_bytes;
        true
    }
}

#[derive(Default)]
pub(super) struct PendingJobQueue {
    jobs: BTreeMap<JobId, PendingJob>,
    ready: BTreeMap<(u8, JobCategory), BTreeSet<JobId>>,
    waiting_counts: BTreeMap<JobId, usize>,
    dependents_by_dependency: BTreeMap<JobId, BTreeSet<JobId>>,
    referenced_dependencies: BTreeMap<JobId, usize>,
    admission_keys: BTreeMap<EditorJobAdmissionKey, JobId>,
    estimated_bytes: usize,
    admission_probes: usize,
    merged_submissions: u64,
    cancelled_pending: u64,
    started_pending: u64,
}

impl PendingJobQueue {
    pub(super) fn insert(&mut self, pending: PendingJob, unscheduled: &[JobId]) {
        let id = pending.id;
        for dependency in &pending.spec.after {
            *self.referenced_dependencies.entry(*dependency).or_default() += 1;
        }
        if unscheduled.is_empty() {
            self.ready
                .entry(queue_key(&pending.spec))
                .or_default()
                .insert(id);
        } else {
            self.waiting_counts.insert(id, unscheduled.len());
            for dependency in unscheduled {
                self.dependents_by_dependency
                    .entry(*dependency)
                    .or_default()
                    .insert(id);
            }
        }
        self.estimated_bytes = self.estimated_bytes.saturating_add(pending.estimated_bytes);
        if let Some(key) = pending.spec.admission_key.as_ref() {
            let replaced = self.admission_keys.insert(key.clone(), id);
            debug_assert!(
                replaced.is_none(),
                "pending admission keys must remain unique"
            );
        }
        let replaced = self.jobs.insert(id, pending);
        debug_assert!(replaced.is_none(), "pending job ids must remain unique");
    }

    pub(super) fn remove(&mut self, id: JobId) -> Option<PendingJob> {
        let pending = self.jobs.remove(&id)?;
        self.estimated_bytes = self.estimated_bytes.saturating_sub(pending.estimated_bytes);
        if let Some(key) = pending.spec.admission_key.as_ref() {
            if self.admission_keys.get(key) == Some(&id) {
                self.admission_keys.remove(key);
            }
        }
        let key = queue_key(&pending.spec);
        remove_from_set_map(&mut self.ready, &key, id);
        self.waiting_counts.remove(&id);
        for dependency in &pending.spec.after {
            remove_from_set_map(&mut self.dependents_by_dependency, dependency, id);
            decrement_count(&mut self.referenced_dependencies, dependency);
        }
        Some(pending)
    }

    pub(super) fn take_next(
        &mut self,
        limits: &EditorJobLimits,
        running_by_category: &BTreeMap<JobCategory, usize>,
    ) -> Option<PendingJob> {
        for priority in JobPriority::ALL {
            let mut selected = None;
            for category in JobCategory::ALL {
                self.admission_probes = self.admission_probes.saturating_add(1);
                if running_by_category
                    .get(&category)
                    .copied()
                    .unwrap_or_default()
                    >= limits.limit(category)
                {
                    continue;
                }
                let key = (priority.admission_rank(), category);
                let Some(id) = self.ready.get(&key).and_then(|ids| ids.first().copied()) else {
                    continue;
                };
                if selected.is_none_or(|selected_id| id < selected_id) {
                    selected = Some(id);
                }
            }
            if let Some(id) = selected {
                let pending = self.remove(id);
                if pending.is_some() {
                    self.started_pending = self.started_pending.saturating_add(1);
                }
                return pending;
            }
        }
        None
    }

    pub(super) fn mark_dependency_schedulable(&mut self, dependency: JobId) {
        let Some(dependents) = self.dependents_by_dependency.remove(&dependency) else {
            return;
        };
        let mut ready = Vec::new();
        for dependent in dependents {
            let Some(remaining) = self.waiting_counts.get_mut(&dependent) else {
                continue;
            };
            *remaining = remaining.saturating_sub(1);
            if *remaining == 0 {
                ready.push(dependent);
            }
        }
        for id in ready {
            self.waiting_counts.remove(&id);
            if let Some(pending) = self.jobs.get(&id) {
                self.ready
                    .entry(queue_key(&pending.spec))
                    .or_default()
                    .insert(id);
            }
        }
    }

    pub(super) fn drain(&mut self) -> Vec<PendingJob> {
        let ids = self.jobs.keys().copied().collect::<Vec<_>>();
        let drained = ids
            .into_iter()
            .filter_map(|id| self.remove(id))
            .collect::<Vec<_>>();
        self.cancelled_pending = self.cancelled_pending.saturating_add(drained.len() as u64);
        drained
    }

    pub(super) fn depends_on(&self, dependency: JobId) -> bool {
        self.referenced_dependencies.contains_key(&dependency)
    }

    pub(super) fn len(&self) -> usize {
        self.jobs.len()
    }

    pub(super) fn ensure_admissible(
        &self,
        spec: &EditorJobSpec,
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        self.ensure_oldest_age(spec, limits, now)?;
        if self.jobs.len() >= limits.max_pending_entries {
            return Err(JobSubmitError::AdmissionEntryLimitExceeded {
                limit: limits.max_pending_entries,
            });
        }
        if self
            .estimated_bytes
            .saturating_add(spec.estimated_pending_bytes)
            > limits.max_pending_estimated_bytes
        {
            return Err(JobSubmitError::AdmissionByteLimitExceeded {
                limit: limits.max_pending_estimated_bytes,
                current: self.estimated_bytes,
                requested: spec.estimated_pending_bytes,
            });
        }
        Ok(())
    }

    pub(super) fn ensure_batch_admissible(
        &self,
        specs: &[&EditorJobSpec],
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        for spec in specs {
            self.ensure_oldest_age(spec, limits, now)?;
        }
        if self.jobs.len().saturating_add(specs.len()) > limits.max_pending_entries {
            return Err(JobSubmitError::AdmissionEntryLimitExceeded {
                limit: limits.max_pending_entries,
            });
        }
        let requested = specs.iter().fold(0_usize, |total, spec| {
            total.saturating_add(spec.estimated_pending_bytes)
        });
        if self.estimated_bytes.saturating_add(requested) > limits.max_pending_estimated_bytes {
            return Err(JobSubmitError::AdmissionByteLimitExceeded {
                limit: limits.max_pending_estimated_bytes,
                current: self.estimated_bytes,
                requested,
            });
        }
        Ok(())
    }

    pub(super) fn admission_snapshot(&self, now: Instant) -> EditorJobAdmissionSnapshot {
        EditorJobAdmissionSnapshot::new(
            self.jobs.len(),
            self.estimated_bytes,
            self.jobs
                .values()
                .next()
                .map(|pending| now.saturating_duration_since(pending.admitted_at)),
            self.merged_submissions,
            self.cancelled_pending,
            self.started_pending,
        )
    }

    pub(super) fn pending_admission_id(&self, spec: &EditorJobSpec) -> Option<JobId> {
        let key = spec.admission_key.as_ref()?;
        self.admission_keys.get(key).copied()
    }

    pub(super) fn merge_pending_admission(
        &mut self,
        existing_id: JobId,
        latest_spec: &EditorJobSpec,
        latest_task: Box<dyn PendingTask>,
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<JobId, JobSubmitError> {
        let Some(existing) = self.jobs.get(&existing_id) else {
            return Err(JobSubmitError::AdmissionKeyConflict {
                existing_job: existing_id,
            });
        };
        if !existing.has_compatible_admission(latest_spec) {
            return Err(JobSubmitError::AdmissionKeyConflict {
                existing_job: existing_id,
            });
        }
        self.ensure_replacement_admissible(existing, latest_spec, limits, now)?;
        let previous_bytes = existing.estimated_bytes;
        let Some(existing) = self.jobs.get_mut(&existing_id) else {
            return Err(JobSubmitError::AdmissionKeyConflict {
                existing_job: existing_id,
            });
        };
        if !existing.replace_with_latest(latest_spec, latest_task) {
            return Err(JobSubmitError::AdmissionKeyConflict {
                existing_job: existing_id,
            });
        }
        self.estimated_bytes = self
            .estimated_bytes
            .saturating_sub(previous_bytes)
            .saturating_add(latest_spec.estimated_pending_bytes);
        self.merged_submissions = self.merged_submissions.saturating_add(1);
        Ok(existing_id)
    }

    pub(super) fn record_cancelled_pending(&mut self) {
        self.cancelled_pending = self.cancelled_pending.saturating_add(1);
    }

    fn ensure_replacement_admissible(
        &self,
        existing: &PendingJob,
        latest: &EditorJobSpec,
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        self.ensure_oldest_age(latest, limits, now)?;
        let projected_bytes = self
            .estimated_bytes
            .saturating_sub(existing.estimated_bytes)
            .saturating_add(latest.estimated_pending_bytes);
        if projected_bytes > limits.max_pending_estimated_bytes {
            return Err(JobSubmitError::AdmissionByteLimitExceeded {
                limit: limits.max_pending_estimated_bytes,
                current: self
                    .estimated_bytes
                    .saturating_sub(existing.estimated_bytes),
                requested: latest.estimated_pending_bytes,
            });
        }
        Ok(())
    }

    fn ensure_oldest_age(
        &self,
        spec: &EditorJobSpec,
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        if self.jobs.values().next().is_some_and(|pending| {
            now.saturating_duration_since(pending.admitted_at)
                >= spec
                    .max_pending_age
                    .unwrap_or(limits.max_oldest_pending_age)
        }) {
            return Err(JobSubmitError::OldestPendingAgeExceeded {
                max_age_ms: duration_millis(
                    spec.max_pending_age
                        .unwrap_or(limits.max_oldest_pending_age),
                ),
            });
        }
        Ok(())
    }

    #[cfg(test)]
    pub(super) fn admission_probe_count(&self) -> usize {
        self.admission_probes
    }
}

fn duration_millis(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn queue_key(spec: &EditorJobSpec) -> (u8, JobCategory) {
    (spec.priority.admission_rank(), spec.category)
}

fn remove_from_set_map<K: Ord + Clone>(map: &mut BTreeMap<K, BTreeSet<JobId>>, key: &K, id: JobId) {
    let should_remove = map.get_mut(key).is_some_and(|ids| {
        ids.remove(&id);
        ids.is_empty()
    });
    if should_remove {
        map.remove(key);
    }
}

fn decrement_count<K: Ord + Clone>(map: &mut BTreeMap<K, usize>, key: &K) {
    let should_remove = map.get_mut(key).is_some_and(|count| {
        *count = count.saturating_sub(1);
        *count == 0
    });
    if should_remove {
        map.remove(key);
    }
}
