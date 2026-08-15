use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use super::super::{EditorJobSpec, JobCategory, JobEventKind, JobId};
use super::{EditorJobProgress, EditorJobProgressSnapshot, EditorJobProgressSource};

const TEST_TIMEOUT: Duration = Duration::from_secs(5);

#[test]
fn primary_generation_initializes_once_and_skips_stable_reads() {
    let progress = EditorJobProgressSource::default();

    let initial = progress
        .primary_snapshot_if_changed(None)
        .expect("the first read must initialize the retained cursor");
    assert!(initial.primary().is_none());
    assert!(
        progress
            .primary_snapshot_if_changed(Some(initial.generation()))
            .is_none()
    );

    let first = JobId::new(2);
    progress.register(first, &EditorJobSpec::new("first", JobCategory::Compile));

    let changed = progress
        .primary_snapshot_if_changed(Some(initial.generation()))
        .expect("a newly visible primary must advance the generation");
    assert_eq!(changed.generation(), initial.generation() + 1);
    assert_eq!(
        changed.primary().map(EditorJobProgressSnapshot::id),
        Some(first)
    );
    assert!(
        progress
            .primary_snapshot_if_changed(Some(changed.generation()))
            .is_none()
    );
}

#[test]
fn future_observation_reads_the_authoritative_primary_generation() {
    let progress = EditorJobProgressSource::default();
    let initial = progress
        .primary_snapshot_if_changed(None)
        .expect("the initial generation must be observable");

    let authoritative = progress
        .primary_snapshot_if_changed(Some(initial.generation() + 1))
        .expect("a future cursor must not suppress the current source state");

    assert_eq!(authoritative.generation(), initial.generation());
    assert!(authoritative.primary().is_none());
}

#[test]
fn mirror_mismatch_rechecks_the_authoritative_generation_under_lock() {
    let progress = EditorJobProgressSource::default();
    let observed_generation = progress
        .primary_snapshot_if_changed(None)
        .expect("the initial generation must be observable")
        .generation();
    progress
        .published_primary_generation
        .store(observed_generation + 1, Ordering::Release);

    assert!(
        progress
            .primary_snapshot_if_changed(Some(observed_generation))
            .is_none(),
        "an atomic mirror mismatch must not fabricate an authoritative change"
    );
}

#[test]
fn stable_generation_does_not_wait_for_the_progress_state_lock() {
    let progress = EditorJobProgressSource::default();
    let observed_generation = progress
        .primary_snapshot_if_changed(None)
        .expect("the initial generation must be observable")
        .generation();
    let state_guard = progress.lock_state();
    let reader = progress.clone();
    let start = Arc::new(Barrier::new(2));
    let worker_start = Arc::clone(&start);
    let (result_sender, result_receiver) = mpsc::channel();
    let worker = thread::spawn(move || {
        worker_start.wait();
        let result = reader.primary_snapshot_if_changed(Some(observed_generation));
        let _ = result_sender.send(result);
    });

    start.wait();
    let result_while_locked = result_receiver.recv_timeout(TEST_TIMEOUT);
    drop(state_guard);
    worker
        .join()
        .expect("the stable generation reader must not panic");

    assert!(matches!(result_while_locked, Ok(None)));
}

#[test]
fn generation_overflow_panics_before_each_primary_projection_mutation() {
    let progress = EditorJobProgressSource::default();
    {
        let mut state = progress.lock_state();
        state.primary_generation = u64::MAX;
    }
    progress
        .published_primary_generation
        .store(u64::MAX, Ordering::Release);

    let overflow = catch_unwind(AssertUnwindSafe(|| {
        progress.register(
            JobId::new(1),
            &EditorJobSpec::new("overflow", JobCategory::Compile),
        );
    }));

    assert!(overflow.is_err());
    assert!(progress.primary_snapshot().is_none());
    assert_eq!(
        progress
            .published_primary_generation
            .load(Ordering::Acquire),
        u64::MAX
    );

    let (progress, id) = primary_source_at_max_generation();
    let overflow = catch_unwind(AssertUnwindSafe(|| {
        progress.apply_event(
            id,
            &JobEventKind::Progress {
                completed: 1,
                total: 2,
                message: "step".to_owned(),
            },
        );
    }));
    assert!(overflow.is_err());
    assert!(
        progress
            .primary_snapshot()
            .expect("the primary entry must remain visible")
            .progress()
            .is_none()
    );

    let (progress, id) = primary_source_at_max_generation();
    let overflow = catch_unwind(AssertUnwindSafe(|| {
        progress.apply_event(id, &JobEventKind::Completed);
    }));
    assert!(overflow.is_err());
    assert!(progress.primary_snapshot().is_some());

    let (progress, id) = primary_source_at_max_generation();
    let overflow = catch_unwind(AssertUnwindSafe(|| progress.complete(id)));
    assert!(overflow.is_err());
    assert!(progress.primary_snapshot().is_some());
}

fn primary_source_at_max_generation() -> (EditorJobProgressSource, JobId) {
    let progress = EditorJobProgressSource::default();
    let id = JobId::new(1);
    progress.register(id, &EditorJobSpec::new("overflow", JobCategory::Compile));
    {
        let mut state = progress.lock_state();
        state.primary_generation = u64::MAX;
    }
    progress
        .published_primary_generation
        .store(u64::MAX, Ordering::Release);
    (progress, id)
}

#[test]
fn smaller_job_registered_after_observation_advances_the_primary_generation() {
    let progress = EditorJobProgressSource::default();
    let later = JobId::new(9);
    let first = JobId::new(2);
    progress.register(later, &EditorJobSpec::new("later", JobCategory::Thumbnail));
    let observed = progress
        .primary_snapshot_if_changed(None)
        .expect("the initial primary must be observable")
        .generation();

    progress.register(first, &EditorJobSpec::new("first", JobCategory::Compile));

    let changed = progress
        .primary_snapshot_if_changed(Some(observed))
        .expect("a lower JobId must replace the visible primary");
    assert_eq!(changed.generation(), observed + 1);
    assert_eq!(
        changed.primary().map(EditorJobProgressSnapshot::id),
        Some(first)
    );
}

#[test]
fn replacing_the_primary_advances_only_when_its_visible_snapshot_changes() {
    let progress = EditorJobProgressSource::default();
    let primary = JobId::new(2);
    let unchanged = EditorJobSpec::new("primary", JobCategory::Compile);
    progress.register(primary, &unchanged);
    let observed = progress
        .primary_snapshot_if_changed(None)
        .expect("the initial primary must be observable")
        .generation();

    progress.register(primary, &unchanged);
    assert!(
        progress
            .primary_snapshot_if_changed(Some(observed))
            .is_none()
    );

    progress.register(
        primary,
        &EditorJobSpec::new("renamed primary", JobCategory::Compile),
    );
    let changed = progress
        .primary_snapshot_if_changed(Some(observed))
        .expect("a changed primary snapshot must advance the generation");
    assert_eq!(changed.generation(), observed + 1);
    assert_eq!(
        changed.primary().map(EditorJobProgressSnapshot::label),
        Some("renamed primary")
    );
}

#[test]
fn non_primary_progress_does_not_advance_the_visible_primary_generation() {
    let progress = EditorJobProgressSource::default();
    let first = JobId::new(2);
    let later = JobId::new(9);
    progress.register(first, &EditorJobSpec::new("first", JobCategory::Compile));
    progress.register(later, &EditorJobSpec::new("later", JobCategory::Thumbnail));

    let observed = progress
        .primary_snapshot_if_changed(None)
        .expect("first observation must return the primary")
        .generation();
    progress.apply_event(
        later,
        &JobEventKind::Progress {
            completed: 1,
            total: 2,
            message: "later".to_owned(),
        },
    );
    assert!(
        progress
            .primary_snapshot_if_changed(Some(observed))
            .is_none()
    );

    progress.apply_event(
        first,
        &JobEventKind::Progress {
            completed: 3,
            total: 4,
            message: "first".to_owned(),
        },
    );
    let changed = progress
        .primary_snapshot_if_changed(Some(observed))
        .expect("primary progress must advance the generation");
    assert_eq!(changed.generation(), observed + 1);
    assert_eq!(
        changed
            .primary()
            .and_then(EditorJobProgressSnapshot::progress)
            .map(EditorJobProgress::message),
        Some("first")
    );

    progress.apply_event(
        first,
        &JobEventKind::Progress {
            completed: 3,
            total: 4,
            message: "first".to_owned(),
        },
    );
    assert!(
        progress
            .primary_snapshot_if_changed(Some(changed.generation()))
            .is_none()
    );
}

#[test]
fn terminal_and_completion_do_not_double_advance_the_primary_generation() {
    let progress = EditorJobProgressSource::default();
    let first = JobId::new(2);
    let later = JobId::new(9);
    progress.register(first, &EditorJobSpec::new("first", JobCategory::Compile));
    progress.register(later, &EditorJobSpec::new("later", JobCategory::Thumbnail));

    let observed = progress
        .primary_snapshot_if_changed(None)
        .expect("first observation must return the primary")
        .generation();
    progress.apply_event(first, &JobEventKind::Completed);
    let after_terminal = progress
        .primary_snapshot_if_changed(Some(observed))
        .expect("terminal primary must expose the next primary");
    assert_eq!(after_terminal.generation(), observed + 1);
    assert_eq!(
        after_terminal.primary().map(EditorJobProgressSnapshot::id),
        Some(later)
    );

    progress.complete(first);
    assert!(
        progress
            .primary_snapshot_if_changed(Some(after_terminal.generation()))
            .is_none()
    );

    progress.complete(later);
    let after_completion = progress
        .primary_snapshot_if_changed(Some(after_terminal.generation()))
        .expect("direct completion of the visible primary must clear it");
    assert_eq!(
        after_completion.generation(),
        after_terminal.generation() + 1
    );
    assert!(after_completion.primary().is_none());
}
