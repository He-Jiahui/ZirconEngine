use std::collections::{BTreeMap, VecDeque};

use zircon_runtime::core::runtime::tasks::JobHandle;

use super::super::{EditorJobLimits, JobCategory, JobId, JobSubmitError, MutexGroup};
use super::pending::{PendingJob, PendingJobQueue};

// Completed dependencies only need a bounded late-submission history, not runtime handles.
pub(super) const TERMINAL_RECORD_RETENTION_LIMIT: usize = 256;

#[derive(Default)]
pub(super) struct EditorJobSystemState {
    next_id: u64,
    closed: bool,
    records: BTreeMap<JobId, EditorJobRecord>,
    terminal_records: VecDeque<JobId>,
    pending: PendingJobQueue,
    running_by_category: BTreeMap<JobCategory, usize>,
    mutex_group_tails: BTreeMap<MutexGroup, MutexGroupTail>,
}

#[derive(Default)]
enum EditorJobRecord {
    #[default]
    AwaitingSchedule,
    Scheduled(JobHandle),
    Terminal,
}

struct MutexGroupTail {
    id: JobId,
    handle: JobHandle,
}

impl EditorJobSystemState {
    pub(super) fn allocate_id(&mut self) -> JobId {
        self.next_id = self.next_id.saturating_add(1);
        JobId::new(self.next_id)
    }

    pub(super) fn validate_dependency(&self, dependency: JobId) -> Result<(), JobSubmitError> {
        if self.records.contains_key(&dependency) {
            return Ok(());
        }
        if dependency.value() > 0 && dependency.value() <= self.next_id {
            return Err(JobSubmitError::ExpiredDependency { dependency });
        }
        Err(JobSubmitError::UnknownDependency { dependency })
    }

    pub(super) fn register(&mut self, id: JobId) {
        self.records.insert(id, EditorJobRecord::default());
    }

    pub(super) fn ensure_accepting_submissions(&self) -> Result<(), JobSubmitError> {
        if self.closed {
            Err(JobSubmitError::ShuttingDown)
        } else {
            Ok(())
        }
    }

    pub(super) fn begin_shutdown(&mut self) -> Vec<PendingJob> {
        self.closed = true;
        self.pending.drain()
    }

    pub(super) fn enqueue_pending(&mut self, pending: PendingJob) {
        let unscheduled = pending
            .spec
            .after
            .iter()
            .copied()
            .filter(|dependency| {
                matches!(
                    self.records.get(dependency),
                    Some(EditorJobRecord::AwaitingSchedule)
                )
            })
            .collect::<Vec<_>>();
        self.pending.insert(pending, &unscheduled);
    }

    pub(super) fn remove_pending(&mut self, id: JobId) -> Option<PendingJob> {
        self.pending.remove(id)
    }

    pub(super) fn pending_len(&self) -> usize {
        self.pending.len()
    }

    pub(super) fn dependency_handle(&self, id: JobId) -> Option<JobHandle> {
        match self.records.get(&id)? {
            EditorJobRecord::AwaitingSchedule => None,
            EditorJobRecord::Scheduled(handle) => Some(handle.clone()),
            EditorJobRecord::Terminal => Some(JobHandle::completed()),
        }
    }

    pub(super) fn store_scheduled_handle(&mut self, id: JobId, handle: JobHandle) {
        let Some(record) = self.records.get_mut(&id) else {
            return;
        };
        if !matches!(record, EditorJobRecord::Terminal) {
            *record = EditorJobRecord::Scheduled(handle);
        }
        self.pending.mark_dependency_schedulable(id);
        self.prune_terminal_records();
    }

    pub(super) fn take_next_admissible(&mut self, limits: &EditorJobLimits) -> Option<PendingJob> {
        self.pending.take_next(limits, &self.running_by_category)
    }

    pub(super) fn mark_started(&mut self, pending: &PendingJob) {
        *self
            .running_by_category
            .entry(pending.spec.category)
            .or_default() += 1;
    }

    pub(super) fn mutex_group_tail(&self, group: &MutexGroup) -> Option<JobHandle> {
        self.mutex_group_tails
            .get(group)
            .map(|tail| tail.handle.clone())
    }

    pub(super) fn update_mutex_group_tail(
        &mut self,
        group: MutexGroup,
        id: JobId,
        handle: JobHandle,
    ) {
        self.mutex_group_tails
            .insert(group, MutexGroupTail { id, handle });
    }

    pub(super) fn mark_finished(&mut self, id: JobId, category: JobCategory) {
        if let Some(running) = self.running_by_category.get_mut(&category) {
            *running = running.saturating_sub(1);
        }
        self.mark_terminal(id);
    }

    pub(super) fn mark_cancelled(&mut self, id: JobId) {
        self.mark_terminal(id);
    }

    #[cfg(test)]
    pub(super) fn retained_record_count(&self) -> usize {
        self.records.len()
    }

    #[cfg(test)]
    pub(super) fn is_terminal_record(&self, id: JobId) -> bool {
        matches!(self.records.get(&id), Some(EditorJobRecord::Terminal))
    }

    #[cfg(test)]
    pub(super) fn scheduled_record_count(&self) -> usize {
        self.records
            .values()
            .filter(|record| matches!(record, EditorJobRecord::Scheduled(_)))
            .count()
    }

    #[cfg(test)]
    pub(super) fn running_job_count(&self) -> usize {
        self.running_by_category.values().sum()
    }

    #[cfg(test)]
    pub(super) fn mutex_group_tail_count(&self) -> usize {
        self.mutex_group_tails.len()
    }

    #[cfg(test)]
    pub(super) fn admission_probe_count(&self) -> usize {
        self.pending.admission_probe_count()
    }

    fn mark_terminal(&mut self, id: JobId) {
        let Some(record) = self.records.get_mut(&id) else {
            return;
        };
        if matches!(record, EditorJobRecord::Terminal) {
            return;
        }

        // Replacing the scheduled handle first releases runtime task state at completion.
        *record = EditorJobRecord::Terminal;
        self.terminal_records.push_back(id);
        self.pending.mark_dependency_schedulable(id);
        self.mutex_group_tails.retain(|_, tail| tail.id != id);
        self.prune_terminal_records();
    }

    fn prune_terminal_records(&mut self) {
        while self.terminal_records.len() > TERMINAL_RECORD_RETENTION_LIMIT {
            let Some(index) = self
                .terminal_records
                .iter()
                .position(|candidate| !self.pending.depends_on(*candidate))
            else {
                // Accepted pending submissions pin their dependencies until scheduling.
                break;
            };
            let id = self
                .terminal_records
                .remove(index)
                .expect("terminal record index came from the same queue");
            if matches!(self.records.get(&id), Some(EditorJobRecord::Terminal)) {
                self.records.remove(&id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::jobs::{EditorJobLimits, EditorJobSpec, JobCategory};

    use super::{EditorJobSystemState, PendingJob, TERMINAL_RECORD_RETENTION_LIMIT};

    #[test]
    fn category_blocked_pending_dependencies_pin_history_until_cancelled() {
        let mut state = EditorJobSystemState::default();
        let limits = EditorJobLimits::default().with_limit(JobCategory::Export, 1);
        state.running_by_category.insert(JobCategory::Export, 1);

        for index in 0..=TERMINAL_RECORD_RETENTION_LIMIT {
            let dependency = state.allocate_id();
            state.register(dependency);
            let dependent = state.allocate_id();
            let dependent_spec =
                EditorJobSpec::new(format!("blocked-{index}"), JobCategory::Export)
                    .after(dependency);
            state.register(dependent);
            state.enqueue_pending(PendingJob::new(
                dependent,
                dependent_spec,
                Box::new(|_| {}),
                Box::new(|_| {}),
            ));
            state.mark_cancelled(dependency);
        }

        assert!(state.take_next_admissible(&limits).is_none());
        assert!(state.terminal_records.len() > TERMINAL_RECORD_RETENTION_LIMIT);
        assert!(state.retained_record_count() > TERMINAL_RECORD_RETENTION_LIMIT);

        for pending in state.pending.drain() {
            state.mark_cancelled(pending.id);
        }

        assert!(state.terminal_records.len() <= TERMINAL_RECORD_RETENTION_LIMIT);
        assert!(state.retained_record_count() <= TERMINAL_RECORD_RETENTION_LIMIT);
    }
}
