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
            EditorJobSpec::new(format!("blocked-{index}"), JobCategory::Export).after(dependency);
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
    let source = include_str!("../state.rs");
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
