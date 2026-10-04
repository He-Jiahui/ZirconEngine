use std::hint::black_box;
use std::time::Instant;

use crate::core::jobs::{EditorJobProgressSnapshot, JobCategory, JobId};
use crate::core::notifications::{NotificationId, NotificationSource};

use super::{
    ProgressNotification, ProgressNotificationCenter, AUTOMATIC_PROGRESS_SOURCE_ID,
    MAX_PROGRESS_NOTIFICATIONS,
};

fn notification(id: &str, job: JobId) -> ProgressNotification {
    ProgressNotification::new(
        NotificationId::parse(id).unwrap(),
        NotificationSource::builtin("editor.progress.test").unwrap(),
        job,
        "editor.progress.title",
    )
    .unwrap()
}

fn job(id: JobId) -> EditorJobProgressSnapshot {
    EditorJobProgressSnapshot::new(id, "job", JobCategory::Import, None, true)
}

#[test]
fn snapshot_unique_lookup_preserves_notification_order_and_visibility() {
    let center = ProgressNotificationCenter::default();
    let first = JobId::new(9);
    let second = JobId::new(1);
    let first_id = NotificationId::parse("editor.progress.b").unwrap();
    let second_id = NotificationId::parse("editor.progress.a").unwrap();
    center
        .publish(notification(first_id.as_str(), first))
        .expect("first progress binding should publish");
    center
        .publish(notification(second_id.as_str(), second))
        .expect("second progress binding should publish");
    let captured = vec![(second_id.clone(), second), (first_id.clone(), first)];
    let snapshots = center.synchronize_captured(&captured, [job(first), job(second)]);

    assert_eq!(
        snapshots
            .iter()
            .map(|snapshot| snapshot.notification().id().as_str())
            .collect::<Vec<_>>(),
        ["editor.progress.a", "editor.progress.b"]
    );

    let snapshots = center.synchronize_captured(&captured, [job(first)]);
    assert_eq!(snapshots.len(), 1);
    assert_eq!(
        snapshots[0].notification().id().as_str(),
        "editor.progress.b"
    );
}

#[test]
fn captured_synchronization_does_not_remove_bindings_added_after_capture() {
    let center = ProgressNotificationCenter::default();
    let captured_id = NotificationId::parse("editor.progress.captured").unwrap();
    let captured_job = JobId::new(1);
    let later_job = JobId::new(2);
    center
        .publish(notification(captured_id.as_str(), captured_job))
        .unwrap();
    let captured = vec![(captured_id, captured_job)];
    center
        .publish(notification("editor.progress.later", later_job))
        .unwrap();

    let projected = center.synchronize_captured(&captured, [job(captured_job)]);

    assert_eq!(projected.len(), 1);
    assert_eq!(projected[0].job().id(), captured_job);
    assert_eq!(center.synchronize([job(later_job)]).len(), 1);
}

#[test]
fn captured_synchronization_preserves_a_reused_id_bound_to_a_new_job() {
    let center = ProgressNotificationCenter::default();
    let id = NotificationId::parse("editor.progress.reused").unwrap();
    let retired_job = JobId::new(1);
    let replacement_job = JobId::new(2);
    center
        .publish(notification(id.as_str(), retired_job))
        .unwrap();
    let captured = vec![(id.clone(), retired_job)];
    center.retire_job(retired_job);
    center
        .publish(notification(id.as_str(), replacement_job))
        .unwrap();

    assert!(center
        .synchronize_captured(&captured, std::iter::empty::<EditorJobProgressSnapshot>(),)
        .is_empty());
    assert_eq!(center.synchronize([job(replacement_job)]).len(), 1);
}

#[test]
fn retiring_a_manual_replacement_releases_its_job_index_entry() {
    let center = ProgressNotificationCenter::default();
    let job_id = JobId::new(7);
    let automatic = ProgressNotification::new(
        NotificationId::parse("editor.progress.automatic").unwrap(),
        NotificationSource::builtin(AUTOMATIC_PROGRESS_SOURCE_ID).unwrap(),
        job_id,
        "editor.progress.title",
    )
    .unwrap();
    center.publish(automatic).unwrap();
    center
        .publish(notification("editor.progress.manual", job_id))
        .unwrap();

    center.retire_job(job_id);

    assert!(center
        .publish(notification("editor.progress.reused", job_id))
        .is_ok());
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_wave_20260825_editor10_progress_job_index_evidence() {
    const LOOKUPS: usize = 100_000;
    const MAX_ELAPSED_NS: u128 = 3_000_000_000;

    let center = ProgressNotificationCenter::default();
    for index in 0..MAX_PROGRESS_NOTIFICATIONS {
        center
            .publish(notification(
                &format!("editor.progress.bench.{index:02}"),
                JobId::new(index as u64),
            ))
            .unwrap();
    }
    let candidate = notification(
        "editor.progress.bench.duplicate",
        JobId::new((MAX_PROGRESS_NOTIFICATIONS - 1) as u64),
    );
    let probes_before = center.job_lookup_probe_count();
    let started = Instant::now();
    for _ in 0..LOOKUPS {
        black_box(center.publish(candidate.clone()).unwrap_err());
    }
    let elapsed_ns = started.elapsed().as_nanos();
    let indexed_job_probes = center
        .job_lookup_probe_count()
        .saturating_sub(probes_before);
    let legacy_candidate_checks = LOOKUPS * MAX_PROGRESS_NOTIFICATIONS;
    let probe_reduction_bps = legacy_candidate_checks
        .saturating_sub(indexed_job_probes)
        .saturating_mul(10_000)
        / legacy_candidate_checks;

    println!(
        "EDITOR_PROGRESS_JOB_INDEX_BENCH_V1 entries={MAX_PROGRESS_NOTIFICATIONS} lookups={LOOKUPS} legacy_candidate_checks={legacy_candidate_checks} indexed_job_probes={indexed_job_probes} probe_reduction_bps={probe_reduction_bps} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}"
    );

    assert_eq!(indexed_job_probes, LOOKUPS);
    assert_eq!(probe_reduction_bps, 9_843);
    assert!(elapsed_ns <= MAX_ELAPSED_NS);
}
