use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::core::runtime::tasks::JobHandle;

use std::time::Instant;

use super::super::{
    EditorJobAdmissionSnapshot, EditorJobLimits, EditorJobSpec, JobCategory, JobId, JobSubmitError,
    MutexGroup,
};
use super::admission_ledger::PendingAdmissionReservation;
use super::pending::{PendingJob, PendingJobQueue};
use super::pending_task::PendingTask;
use super::EditorJobAdmissionWindow;

// Completed dependencies only need a bounded late-submission history, not runtime handles.
pub(super) const TERMINAL_RECORD_RETENTION_LIMIT: usize = 256;

#[derive(Default)]
pub(super) struct EditorJobSystemState {
    next_id: u64,
    next_admission_reservation: u64,
    next_terminal_order: u64,
    closed: bool,
    records: BTreeMap<JobId, EditorJobRecord>,
    terminal_records: BTreeSet<(u64, JobId)>,
    terminal_orders: BTreeMap<JobId, u64>,
    evictable_terminal_records: BTreeSet<(u64, JobId)>,
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
        let pending = self.pending.drain();
        self.pending.release_all_reservations();
        for job in &pending {
            self.release_terminal_dependencies(job);
        }
        self.prune_terminal_records();
        pending
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
        // A dependency can already be terminal when this pending job is
        // admitted. Keep that terminal record until this job has captured its
        // completed runtime handle during promotion.
        self.pin_terminal_dependencies(&pending);
        self.pending.insert(pending, &unscheduled);
    }

    pub(super) fn ensure_pending_admissible(
        &self,
        spec: &EditorJobSpec,
        limits: &EditorJobLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        self.pending
            .ensure_admissible(spec, limits.admission_limits(), now)
    }

    pub(super) fn ensure_batch_pending_admissible(
        &self,
        specs: &[&EditorJobSpec],
        limits: &EditorJobLimits,
        now: Instant,
    ) -> Result<(), JobSubmitError> {
        self.pending
            .ensure_batch_admissible(specs, limits.admission_limits(), now)
    }

    pub(super) fn reserve_batch_admission(
        &mut self,
        requests: Vec<super::super::EditorJobAdmissionRequest>,
        limits: &EditorJobLimits,
        admitted_at: Instant,
    ) -> Result<u64, JobSubmitError> {
        self.ensure_accepting_submissions()?;
        let requests_for_preflight = requests.iter().collect::<Vec<_>>();
        self.pending.ensure_reservation_batch_admissible(
            &requests_for_preflight,
            limits.admission_limits(),
            admitted_at,
        )?;
        self.next_admission_reservation = self.next_admission_reservation.saturating_add(1);
        let reservation_id = self.next_admission_reservation;
        let reservations = requests
            .into_iter()
            .map(|request| PendingAdmissionReservation {
                id: self.allocate_id(),
                request,
                admitted_at,
            })
            .collect::<Vec<_>>();
        self.pending.reserve_batch(
            reservation_id,
            reservations,
            limits.admission_limits(),
            admitted_at,
        )?;
        Ok(reservation_id)
    }

    pub(super) fn commit_batch_admission_reservation(
        &mut self,
        reservation_id: u64,
        specs: &[&EditorJobSpec],
    ) -> Result<Vec<(JobId, Instant)>, JobSubmitError> {
        self.ensure_accepting_submissions()?;
        for spec in specs {
            for dependency in &spec.after {
                self.validate_dependency(*dependency)?;
            }
        }
        self.pending.commit_reservation(reservation_id, specs)
    }

    pub(super) fn release_batch_admission_reservation(&mut self, reservation_id: u64) -> bool {
        self.pending.release_reservation(reservation_id)
    }

    pub(super) fn pending_admission_id(&self, spec: &EditorJobSpec) -> Option<JobId> {
        self.pending.pending_admission_id(spec)
    }

    pub(super) fn merge_pending_admission(
        &mut self,
        existing_id: JobId,
        spec: &EditorJobSpec,
        task: Box<dyn PendingTask>,
        limits: &EditorJobLimits,
        now: Instant,
    ) -> Result<JobId, JobSubmitError> {
        self.pending.merge_pending_admission(
            existing_id,
            spec,
            task,
            limits.admission_limits(),
            now,
        )
    }

    pub(super) fn admission_snapshot(&self, now: Instant) -> EditorJobAdmissionSnapshot {
        self.pending.admission_snapshot(now)
    }

    pub(super) fn pending_admission_window(
        &self,
        limits: &EditorJobLimits,
        now: Instant,
    ) -> Result<EditorJobAdmissionWindow, JobSubmitError> {
        self.ensure_accepting_submissions()?;
        self.pending.pending_admission_window(limits, now)
    }

    pub(super) fn category_admission_snapshot(
        &self,
        category: JobCategory,
        now: Instant,
    ) -> EditorJobAdmissionSnapshot {
        self.pending.category_admission_snapshot(category, now)
    }

    pub(super) fn remove_pending(&mut self, id: JobId) -> Option<PendingJob> {
        let pending = self.pending.remove(id);
        if let Some(pending) = pending.as_ref() {
            self.pending.record_cancelled_pending(pending.spec.category);
            self.release_terminal_dependencies(pending);
            self.prune_terminal_records();
        }
        pending
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

    pub(super) fn scheduling_dependencies(&self, pending: &PendingJob) -> Vec<JobHandle> {
        let mut dependencies = pending
            .spec
            .after
            .iter()
            .map(|id| {
                self.dependency_handle(*id)
                    .expect("pending dependency records stay pinned until scheduling")
            })
            .collect::<Vec<_>>();
        if let Some(group) = pending.spec.mutex_group.as_ref() {
            if let Some(group_tail) = self.mutex_group_tail(group) {
                dependencies.push(group_tail);
            }
        }
        dependencies
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
        // The scheduler handle for every dependency is already captured before
        // this point. The accepted task no longer needs terminal history.
        self.release_terminal_dependencies(pending);
        self.prune_terminal_records();
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
        if !matches!(self.records.get(&id), Some(EditorJobRecord::Scheduled(_))) {
            return;
        }
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
        self.next_terminal_order = self.next_terminal_order.saturating_add(1);
        let terminal = (self.next_terminal_order, id);
        self.terminal_records.insert(terminal);
        self.terminal_orders.insert(id, self.next_terminal_order);
        self.pending.mark_dependency_schedulable(id);
        self.mutex_group_tails.retain(|_, tail| tail.id != id);
        self.mark_terminal_evictable_if_unpinned(id);
        self.prune_terminal_records();
    }

    fn release_terminal_dependencies(&mut self, pending: &PendingJob) {
        for dependency in &pending.spec.after {
            self.mark_terminal_evictable_if_unpinned(*dependency);
        }
    }

    fn pin_terminal_dependencies(&mut self, pending: &PendingJob) {
        for dependency in &pending.spec.after {
            let Some(order) = self.terminal_orders.get(dependency).copied() else {
                continue;
            };
            self.evictable_terminal_records
                .remove(&(order, *dependency));
        }
    }

    fn mark_terminal_evictable_if_unpinned(&mut self, id: JobId) {
        if self.pending.depends_on(id)
            || !matches!(self.records.get(&id), Some(EditorJobRecord::Terminal))
        {
            return;
        }
        if let Some(order) = self.terminal_orders.get(&id).copied() {
            self.evictable_terminal_records.insert((order, id));
        }
    }

    fn prune_terminal_records(&mut self) {
        while self.terminal_records.len() > TERMINAL_RECORD_RETENTION_LIMIT {
            let Some(terminal) = self.evictable_terminal_records.pop_first() else {
                // Accepted pending submissions pin their dependencies until scheduling.
                break;
            };
            let (_, id) = terminal;
            self.terminal_records.remove(&terminal);
            self.terminal_orders.remove(&id);
            if matches!(self.records.get(&id), Some(EditorJobRecord::Terminal)) {
                self.records.remove(&id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::core::jobs::{EditorJobLimits, EditorJobSpec, JobCategory, MutexGroup};
    use zircon_runtime::core::runtime::tasks::JobHandle;

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
                Instant::now(),
            ));
            state.mark_cancelled(dependency);
        }

        assert!(state.take_next_admissible(&limits).is_none());
        assert!(state.terminal_records.len() > TERMINAL_RECORD_RETENTION_LIMIT);
        assert!(state.retained_record_count() > TERMINAL_RECORD_RETENTION_LIMIT);

        for pending in state.begin_shutdown() {
            state.mark_cancelled(pending.id);
        }

        assert!(state.terminal_records.len() <= TERMINAL_RECORD_RETENTION_LIMIT);
        assert!(state.retained_record_count() <= TERMINAL_RECORD_RETENTION_LIMIT);
    }

    #[test]
    fn terminal_history_eviction_uses_indexed_candidates_not_a_linear_queue_scan() {
        let source = include_str!("state.rs");
        let prune = source
            .split("fn prune_terminal_records")
            .nth(1)
            .expect("terminal history prune implementation");

        assert!(source.contains("evictable_terminal_records: BTreeSet"));
        assert!(prune.contains("evictable_terminal_records.pop_first()"));
        assert!(!prune.contains(".position("));
        assert!(!prune.contains(".remove(index)"));
    }

    #[test]
    fn scheduling_dependencies_include_the_previous_mutex_owner_tail() {
        let mut state = EditorJobSystemState::default();
        let explicit_dependency = state.allocate_id();
        state.register(explicit_dependency);
        state.store_scheduled_handle(explicit_dependency, JobHandle::completed());
        let group = MutexGroup::parse("welcome_project_probe_test").unwrap();
        let previous_owner = state.allocate_id();
        state.register(previous_owner);
        state.store_scheduled_handle(previous_owner, JobHandle::completed());
        state.update_mutex_group_tail(group.clone(), previous_owner, JobHandle::completed());
        let pending_id = state.allocate_id();
        let pending = PendingJob::new(
            pending_id,
            EditorJobSpec::new("latest-probe", JobCategory::Index)
                .after(explicit_dependency)
                .with_mutex_group(group),
            Box::new(|_| {}),
            Box::new(|_| {}),
            Instant::now(),
        );

        let dependencies = state.scheduling_dependencies(&pending);

        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.iter().all(JobHandle::is_complete));
    }

    #[test]
    fn terminal_record_cannot_reinstall_a_mutex_tail_after_fast_completion() {
        let mut state = EditorJobSystemState::default();
        let id = state.allocate_id();
        state.register(id);
        state.mark_finished(id, JobCategory::Export);
        let group = MutexGroup::parse("terminal_before_tail").unwrap();

        state.store_scheduled_handle(id, JobHandle::completed());
        state.update_mutex_group_tail(group.clone(), id, JobHandle::completed());

        assert!(state.mutex_group_tail(&group).is_none());
        assert_eq!(state.mutex_group_tail_count(), 0);
    }

    #[test]
    fn a_late_pending_dependency_pins_its_terminal_record_through_retention_pruning() {
        let mut state = EditorJobSystemState::default();
        let dependency = state.allocate_id();
        state.register(dependency);
        state.mark_cancelled(dependency);

        let dependent = state.allocate_id();
        state.register(dependent);
        state.enqueue_pending(PendingJob::new(
            dependent,
            EditorJobSpec::new("late-dependent", JobCategory::Export).after(dependency),
            Box::new(|_| {}),
            Box::new(|_| {}),
            Instant::now(),
        ));

        for _ in 0..=TERMINAL_RECORD_RETENTION_LIMIT {
            let terminal = state.allocate_id();
            state.register(terminal);
            state.mark_cancelled(terminal);
        }

        assert!(state.is_terminal_record(dependency));
        assert!(state.dependency_handle(dependency).is_some());
        assert!(state
            .take_next_admissible(&EditorJobLimits::default())
            .is_some());
    }
}
