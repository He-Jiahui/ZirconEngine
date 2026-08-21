use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use super::super::{
    EditorJobAdmissionKey, EditorJobAdmissionLimits, EditorJobAdmissionRequest,
    EditorJobAdmissionSnapshot, EditorJobLimits, EditorJobSpec, JobCategory, JobId, JobPriority,
    JobSubmitError,
};
use super::admission_ledger::{PendingAdmissionLedger, PendingAdmissionReservation};
use super::pending_task::{PendingCancelTask, PendingTask};
use super::EditorJobAdmissionWindow;

const FAIR_ADMISSION_SLOTS: [JobPriority; 6] = [
    JobPriority::Interactive,
    JobPriority::Interactive,
    JobPriority::Normal,
    JobPriority::Interactive,
    JobPriority::Normal,
    JobPriority::Background,
];

pub(super) const MAX_ADMISSION_BUCKET_PROBES_PER_PASS: usize =
    FAIR_ADMISSION_SLOTS.len() * JobCategory::ALL.len();

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
        self.spec.cancel = latest.cancel.clone();
        self.spec.estimated_pending_bytes = latest.estimated_pending_bytes;
        self.spec.max_pending_age = latest.max_pending_age;
        self.estimated_bytes = latest.estimated_pending_bytes;
        true
    }
}

#[derive(Default)]
pub(super) struct PendingJobQueue {
    jobs: BTreeMap<JobId, PendingJob>,
    // This is the sole admission accounting index for executable jobs and
    // pre-materialization claims. Only `jobs` participates in promotion.
    admission: PendingAdmissionLedger,
    ready: BTreeMap<(u8, JobCategory), BTreeSet<JobId>>,
    waiting_counts: BTreeMap<JobId, usize>,
    dependents_by_dependency: BTreeMap<JobId, BTreeSet<JobId>>,
    referenced_dependencies: BTreeMap<JobId, usize>,
    admission_keys: BTreeMap<EditorJobAdmissionKey, JobId>,
    fairness_slot: usize,
    admission_probes: usize,
}

impl PendingJobQueue {
    pub(super) fn insert(&mut self, pending: PendingJob, unscheduled: &[JobId]) {
        let id = pending.id;
        self.admission.insert(
            id,
            pending.spec.category,
            pending.estimated_bytes,
            pending.admitted_at,
        );
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
        let admission = self
            .admission
            .remove(id)
            .expect("pending jobs must retain one admission entry");
        debug_assert_eq!(admission.0, pending.spec.category);
        debug_assert_eq!(admission.1, pending.estimated_bytes);
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
        for offset in 0..FAIR_ADMISSION_SLOTS.len() {
            let slot = (self.fairness_slot + offset) % FAIR_ADMISSION_SLOTS.len();
            let priority = FAIR_ADMISSION_SLOTS[slot];
            let Some(id) = self.first_ready_admissible_id(priority, limits, running_by_category)
            else {
                continue;
            };
            self.fairness_slot = (slot + 1) % FAIR_ADMISSION_SLOTS.len();
            let pending = self.remove(id);
            if let Some(pending) = pending.as_ref() {
                self.admission.record_started(pending.spec.category);
            }
            return pending;
        }
        None
    }

    fn first_ready_admissible_id(
        &mut self,
        priority: JobPriority,
        limits: &EditorJobLimits,
        running_by_category: &BTreeMap<JobCategory, usize>,
    ) -> Option<JobId> {
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
        selected
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
        for pending in &drained {
            self.admission.record_cancelled(pending.spec.category);
        }
        drained
    }

    pub(super) fn depends_on(&self, dependency: JobId) -> bool {
        self.referenced_dependencies.contains_key(&dependency)
    }

    pub(super) fn len(&self) -> usize {
        self.jobs.len()
    }

    pub(super) fn reserve_batch(
        &mut self,
        reservation_id: u64,
        reservations: Vec<PendingAdmissionReservation>,
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        self.admission
            .reserve_batch(reservation_id, reservations, limits, now)
    }

    pub(super) fn ensure_reservation_batch_admissible(
        &self,
        requests: &[&EditorJobAdmissionRequest],
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        self.admission
            .ensure_reservation_batch_admissible(requests, limits, now)
    }

    pub(super) fn commit_reservation(
        &mut self,
        reservation_id: u64,
        specs: &[&EditorJobSpec],
    ) -> Result<Vec<(JobId, Instant)>, JobSubmitError> {
        self.admission.commit_reservation(reservation_id, specs)
    }

    pub(super) fn release_reservation(&mut self, reservation_id: u64) -> bool {
        self.admission.release_reservation(reservation_id)
    }

    pub(super) fn release_all_reservations(&mut self) {
        self.admission.release_all_reservations();
    }

    pub(super) fn ensure_admissible(
        &self,
        spec: &EditorJobSpec,
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        self.admission.ensure_admissible(spec, limits, now)
    }

    pub(super) fn ensure_batch_admissible(
        &self,
        specs: &[&EditorJobSpec],
        limits: EditorJobAdmissionLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        self.admission.ensure_batch_admissible(specs, limits, now)
    }

    pub(super) fn pending_admission_window(
        &self,
        limits: &EditorJobLimits,
        now: Instant,
    ) -> Result<EditorJobAdmissionWindow, JobSubmitError> {
        self.admission.pending_admission_window(limits, now)
    }

    pub(super) fn admission_snapshot(&self, now: Instant) -> EditorJobAdmissionSnapshot {
        self.admission.admission_snapshot(now)
    }

    pub(super) fn category_admission_snapshot(
        &self,
        category: JobCategory,
        now: Instant,
    ) -> EditorJobAdmissionSnapshot {
        self.admission.category_admission_snapshot(category, now)
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
        self.admission.ensure_replacement_admissible(
            existing.estimated_bytes,
            latest_spec,
            limits,
            now,
        )?;
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
        self.admission.replace_bytes(
            existing_id,
            latest_spec.category,
            previous_bytes,
            latest_spec.estimated_pending_bytes,
        );
        self.admission.record_merged(latest_spec.category);
        Ok(existing_id)
    }

    pub(super) fn record_cancelled_pending(&mut self, category: JobCategory) {
        self.admission.record_cancelled(category);
    }

    #[cfg(test)]
    pub(super) fn admission_probe_count(&self) -> usize {
        self.admission_probes
    }
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

#[cfg(test)]
mod tests;
