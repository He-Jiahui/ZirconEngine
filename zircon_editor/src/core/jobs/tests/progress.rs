use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::core::notifications::{
    NotificationId, NotificationSource, ProgressNotification, ProgressNotificationCenter,
};

use super::super::{EditorJobSpec, JobCategory, JobEventKind, JobId, JobPriority};
use super::{EditorJobProgressSnapshot, EditorJobProgressSource};

#[test]
fn terminal_visibility_hides_ui_before_lifecycle_completion_removes_the_entry() {
    let progress = EditorJobProgressSource::default();
    let id = JobId::new(7);
    progress.register(id, &EditorJobSpec::new("terminal", JobCategory::Compile));

    progress.apply_event(id, &JobEventKind::Completed);

    assert!(progress.snapshot().is_empty());
    assert!(progress.has_active());
    assert_eq!(progress.unfinished_jobs()[0].id(), id);
    assert!(!progress.request_cancel(id));

    progress.complete(id);
    assert!(!progress.has_active());
    assert!(progress.unfinished_jobs().is_empty());
}

#[test]
fn primary_snapshot_clones_only_the_smallest_visible_job() {
    let progress = EditorJobProgressSource::default();
    let first = JobId::new(2);
    let hidden = JobId::new(1);
    let later = JobId::new(9);
    progress.register(later, &EditorJobSpec::new("later", JobCategory::Thumbnail));
    progress.register(first, &EditorJobSpec::new("first", JobCategory::Compile));
    progress.register(hidden, &EditorJobSpec::new("hidden", JobCategory::Import));
    progress.apply_event(hidden, &JobEventKind::Completed);

    let primary = progress.primary_snapshot().unwrap();

    assert_eq!(primary.id(), first);
    assert_eq!(primary.label(), "first");
    assert_eq!(progress.snapshot().len(), 2);
}

#[test]
fn selected_snapshots_exclude_unrequested_and_terminal_jobs() {
    let progress = EditorJobProgressSource::default();
    let first = JobId::new(2);
    let later = JobId::new(9);
    let unrequested = JobId::new(3);
    let terminal = JobId::new(4);
    progress.register(first, &EditorJobSpec::new("first", JobCategory::Import));
    progress.register(later, &EditorJobSpec::new("later", JobCategory::Thumbnail));
    progress.register(
        unrequested,
        &EditorJobSpec::new("unrequested", JobCategory::Compile),
    );
    progress.register(
        terminal,
        &EditorJobSpec::new("terminal", JobCategory::Index),
    );
    progress.apply_event(terminal, &JobEventKind::Completed);

    let snapshots = progress.snapshot_for_ids([later, terminal, first, later]);

    assert_eq!(
        snapshots
            .iter()
            .map(EditorJobProgressSnapshot::id)
            .collect::<Vec<_>>(),
        vec![first, later]
    );
}

#[test]
fn trusted_selected_snapshots_preserve_unique_caller_order() {
    let progress = EditorJobProgressSource::default();
    let first = JobId::new(2);
    let later = JobId::new(9);
    let terminal = JobId::new(4);
    progress.register(first, &EditorJobSpec::new("first", JobCategory::Import));
    progress.register(later, &EditorJobSpec::new("later", JobCategory::Thumbnail));
    progress.register(
        terminal,
        &EditorJobSpec::new("terminal", JobCategory::Index),
    );
    progress.apply_event(terminal, &JobEventKind::Completed);

    let snapshots = progress.snapshot_for_unique_ids([later, terminal, first]);

    assert_eq!(
        snapshots
            .iter()
            .map(EditorJobProgressSnapshot::id)
            .collect::<Vec<_>>(),
        vec![later, first]
    );
}

#[test]
fn notification_snapshot_uses_unique_job_lookup_without_reordering_rows() {
    let jobs = EditorJobProgressSource::default();
    let center = ProgressNotificationCenter::default();
    let later_notification = JobId::new(9);
    let first_notification = JobId::new(1);
    jobs.register(
        later_notification,
        &EditorJobSpec::new("later", JobCategory::Thumbnail),
    );
    jobs.register(
        first_notification,
        &EditorJobSpec::new("first", JobCategory::Import),
    );
    center
        .publish(
            ProgressNotification::new(
                NotificationId::parse("editor.progress.b").unwrap(),
                NotificationSource::builtin("editor.progress.test").unwrap(),
                later_notification,
                "editor.progress.title",
            )
            .unwrap(),
        )
        .unwrap();
    center
        .publish(
            ProgressNotification::new(
                NotificationId::parse("editor.progress.a").unwrap(),
                NotificationSource::builtin("editor.progress.test").unwrap(),
                first_notification,
                "editor.progress.title",
            )
            .unwrap(),
        )
        .unwrap();

    let snapshots = center.snapshot(&jobs);

    assert_eq!(
        snapshots
            .iter()
            .map(|snapshot| snapshot.notification().id().as_str())
            .collect::<Vec<_>>(),
        ["editor.progress.a", "editor.progress.b"]
    );

    jobs.apply_event(first_notification, &JobEventKind::Completed);
    let snapshots = center.snapshot(&jobs);
    assert_eq!(snapshots.len(), 1);
    assert_eq!(
        snapshots[0].notification().id().as_str(),
        "editor.progress.b"
    );
}

#[test]
fn selected_snapshot_lookup_uses_requested_ids_without_active_scan() {
    let source = include_str!("../progress.rs");
    let method_start = source
        .find("pub fn snapshot_for_ids(")
        .expect("snapshot_for_ids should remain available");
    let method_end = method_start
        + source[method_start..]
            .find("\n    pub(super) fn register")
            .expect("snapshot_for_ids should end before registration");
    let method = &source[method_start..method_end];

    assert!(method.contains("ids.into_iter()"));
    assert!(method.contains("active.get(&id)"));
    for active_scan in [
        "active.iter()",
        "active.values()",
        "active.keys()",
        "active.range(",
        "active.into_iter()",
    ] {
        assert!(
            !method.contains(active_scan),
            "selected snapshot lookup must not scan active entries through {active_scan}"
        );
    }
}

#[test]
fn limited_snapshots_preserve_stable_order_without_cloning_the_tail() {
    let progress = EditorJobProgressSource::default();
    for id in [JobId::new(4), JobId::new(1), JobId::new(7)] {
        progress.register(id, &EditorJobSpec::new("limited", JobCategory::Import));
    }

    let snapshots = progress.snapshot_limit(2);

    assert_eq!(
        snapshots
            .iter()
            .map(EditorJobProgressSnapshot::id)
            .collect::<Vec<_>>(),
        vec![JobId::new(1), JobId::new(4)]
    );
}

#[test]
fn progress_projections_reserve_known_output_upper_bounds() {
    let production = include_str!("../progress.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("progress implementation");

    assert!(production.contains("Vec::with_capacity(state.active.len())"));
    assert!(production.contains("Vec::with_capacity(limit.min(state.active.len()))"));
    assert!(production.contains("Vec::with_capacity(ids.len())"));
    assert!(production.contains("snapshots.extend("));
    assert!(production.contains("unfinished.extend("));
}

#[test]
#[ignore = "managed Editor09 performance evidence"]
fn editor09_progress_projection_capacity_evidence() {
    const ACTIVE_JOBS: usize = 10_000;
    let legacy_growths = geometric_growth_events(ACTIVE_JOBS, 0);
    let optimized_growths = geometric_growth_events(ACTIVE_JOBS, ACTIVE_JOBS);

    println!(
        "EDITOR09_PROGRESS_PROJECTION_CAPACITY_BENCH_V1 legacy_growths={} optimized_growths={} active_jobs={}",
        legacy_growths, optimized_growths, ACTIVE_JOBS,
    );
    assert!(legacy_growths > 0);
    assert_eq!(optimized_growths, 0);
}

fn geometric_growth_events(target_len: usize, initial_capacity: usize) -> usize {
    let mut capacity = initial_capacity;
    let mut growths = 0;
    for length in 0..target_len {
        if length == capacity {
            capacity = capacity.max(1).saturating_mul(2);
            growths += 1;
        }
    }
    growths
}

#[test]
fn repeated_snapshots_share_label_and_progress_message_allocations() {
    let progress = EditorJobProgressSource::default();
    let id = JobId::new(11);
    progress.register(id, &EditorJobSpec::new("shared-label", JobCategory::Import));
    progress.apply_event(
        id,
        &JobEventKind::Progress {
            completed: 1,
            total: 2,
            message: "shared-progress-message".to_string(),
        },
    );

    let first = progress.snapshot().pop().unwrap();
    let second = progress.snapshot().pop().unwrap();

    assert!(Arc::ptr_eq(&first.label, &second.label));
    assert!(Arc::ptr_eq(
        &first.progress.as_ref().unwrap().message,
        &second.progress.as_ref().unwrap().message,
    ));
}

#[test]
fn interactive_progress_preempts_an_older_background_job() {
    let progress = EditorJobProgressSource::default();
    let background = JobId::new(1);
    let interactive = JobId::new(99);
    progress.register(
        background,
        &EditorJobSpec::new("background", JobCategory::Index)
            .with_priority(JobPriority::Background),
    );
    let observed = progress
        .primary_snapshot_if_changed(None)
        .unwrap()
        .generation();
    progress.register(
        interactive,
        &EditorJobSpec::new("interactive", JobCategory::InteractiveSave)
            .with_priority(JobPriority::Interactive),
    );

    let preempted = progress
        .primary_snapshot_if_changed(Some(observed))
        .expect("interactive registration must advance the retained primary");
    assert_eq!(preempted.primary().unwrap().id(), interactive);

    progress.apply_event(interactive, &JobEventKind::Completed);

    let resumed = progress
        .primary_snapshot_if_changed(Some(preempted.generation()))
        .expect("terminal interactive work must restore the background primary");
    assert_eq!(resumed.primary().unwrap().id(), background);
}

#[test]
fn primary_lookup_uses_the_maintained_visibility_index() {
    let source = include_str!("../progress.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("progress implementation");
    let primary_entry = implementation
        .split("fn primary_entry(&self)")
        .nth(1)
        .and_then(|source| source.split("fn primary_id(&self)").next())
        .expect("primary entry implementation");

    assert!(implementation.contains("visible_by_priority: BTreeSet<(u8, JobId)>"));
    assert!(primary_entry.contains("visible_by_priority.first()"));
    assert!(!primary_entry.contains("active.values()"));
    assert!(!primary_entry.contains("min_by_key"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn editor10_progress_snapshot_unique_id_evidence() {
    const BINDINGS: usize = 64;
    const POLLS: usize = 100_000;
    const MAX_ELAPSED_NS: u128 = 5_000_000_000;

    let jobs = EditorJobProgressSource::default();
    let center = ProgressNotificationCenter::default();
    for index in 0..BINDINGS {
        let job_id = JobId::new(index as u64 + 1);
        jobs.register(
            job_id,
            &EditorJobSpec::new(format!("progress-bench-{index:02}"), JobCategory::Import),
        );
        center
            .publish(
                ProgressNotification::new(
                    NotificationId::parse(format!("editor.progress.bench.{index:02}")).unwrap(),
                    NotificationSource::builtin("editor.progress.bench").unwrap(),
                    job_id,
                    "editor.progress.title",
                )
                .unwrap(),
            )
            .unwrap();
    }

    let started = Instant::now();
    for _ in 0..POLLS {
        std::hint::black_box(center.snapshot(&jobs));
    }
    let elapsed_ns = started.elapsed().as_nanos();
    let legacy_linear_pass_visits = POLLS.saturating_mul(BINDINGS).saturating_mul(4);
    let optimized_linear_pass_visits = POLLS.saturating_mul(BINDINGS).saturating_mul(2);
    let modeled_reduction_bps = legacy_linear_pass_visits
        .saturating_sub(optimized_linear_pass_visits)
        .saturating_mul(10_000)
        / legacy_linear_pass_visits;

    println!(
        "EDITOR_PROGRESS_SNAPSHOT_UNIQUE_ID_BENCH_V1 bindings={BINDINGS} polls={POLLS} legacy_linear_pass_visits={legacy_linear_pass_visits} optimized_linear_pass_visits={optimized_linear_pass_visits} modeled_reduction_bps={modeled_reduction_bps} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}"
    );

    assert_eq!(modeled_reduction_bps, 5_000);
    assert!(elapsed_ns <= MAX_ELAPSED_NS);
}

#[test]
#[ignore = "managed Editor09 performance evidence"]
fn editor09_progress_snapshot_shared_string_evidence() {
    const ACTIVE_JOBS: usize = 10_000;
    const REPEATS: usize = 8;
    const MAX_NON_PRIMARY_UPDATE_LATENCY: Duration = Duration::from_secs(2);

    let progress = EditorJobProgressSource::default();
    for index in 1..=ACTIVE_JOBS {
        let id = JobId::new(index as u64);
        progress.register(
            id,
            &EditorJobSpec::new(
                format!("background-progress-job-{index:05}"),
                JobCategory::Import,
            ),
        );
        progress.apply_event(
            id,
            &JobEventKind::Progress {
                completed: 1,
                total: 100,
                message: format!("processing background artifact {index:05}"),
            },
        );
    }

    let update_started = Instant::now();
    for index in 2..=ACTIVE_JOBS {
        progress.apply_event(
            JobId::new(index as u64),
            &JobEventKind::Progress {
                completed: 2,
                total: 100,
                message: format!("updated background artifact {index:05}"),
            },
        );
    }
    let update_elapsed = update_started.elapsed();
    assert!(update_elapsed <= MAX_NON_PRIMARY_UPDATE_LATENCY);
    let primary_candidates_scanned_before = (ACTIVE_JOBS - 1).saturating_mul(ACTIVE_JOBS);
    let primary_index_reads_after = ACTIVE_JOBS - 1;
    let primary_lookup_reduction_percent =
        (1.0 - primary_index_reads_after as f64 / primary_candidates_scanned_before as f64) * 100.0;
    let baseline = progress.snapshot();
    let copied_string_bytes_before_per_snapshot = baseline
        .iter()
        .map(|snapshot| {
            snapshot.label().len()
                + snapshot
                    .progress()
                    .map(|progress| progress.message().len())
                    .unwrap_or_default()
        })
        .sum::<usize>();
    let mut samples = Vec::with_capacity(REPEATS);
    for _ in 0..REPEATS {
        let started = Instant::now();
        let snapshot = progress.snapshot();
        samples.push(started.elapsed().as_nanos());
        assert_eq!(snapshot.len(), ACTIVE_JOBS);
        assert!(snapshot
            .iter()
            .zip(&baseline)
            .all(|(next, previous)| Arc::ptr_eq(&next.label, &previous.label)
                && Arc::ptr_eq(
                    &next.progress.as_ref().unwrap().message,
                    &previous.progress.as_ref().unwrap().message,
                )));
    }
    samples.sort_unstable();

    println!(
        "EDITOR_JOB_BENCH_V1 kind=progress_snapshot_shared_strings active_jobs={} repeats={} copied_string_bytes_before={} copied_string_bytes_after=0 copied_string_reduction_percent=100.0000 non_primary_updates={} primary_candidates_scanned_before={} primary_index_reads_after={} primary_lookup_reduction_percent={:.4} update_elapsed_ns={} update_target_ns={} p50_ns={} p95_ns={} max_ns={}",
        ACTIVE_JOBS,
        REPEATS,
        copied_string_bytes_before_per_snapshot.saturating_mul(REPEATS),
        ACTIVE_JOBS - 1,
        primary_candidates_scanned_before,
        primary_index_reads_after,
        primary_lookup_reduction_percent,
        update_elapsed.as_nanos(),
        MAX_NON_PRIMARY_UPDATE_LATENCY.as_nanos(),
        samples[REPEATS / 2],
        samples[REPEATS - 1],
        samples[REPEATS - 1],
    );
}
