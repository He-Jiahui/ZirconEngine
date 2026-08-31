use std::collections::BTreeMap;
use std::time::Instant;

use super::super::*;
use super::ReplaceablePendingTask;

#[test]
fn ready_background_job_is_selected_within_one_weighted_fairness_round() {
    let mut pending = PendingJobQueue::default();
    let admitted_at = Instant::now();
    pending.insert(
        PendingJob::new(
            JobId::new(1),
            EditorJobSpec::new("background", JobCategory::Index)
                .with_priority(JobPriority::Background),
            Box::new(ReplaceablePendingTask),
            Box::new(|_| {}),
            admitted_at,
        ),
        &[],
    );
    for id in 2..=8 {
        pending.insert(
            PendingJob::new(
                JobId::new(id),
                EditorJobSpec::new(format!("interactive-{id}"), JobCategory::Index)
                    .with_priority(JobPriority::Interactive),
                Box::new(ReplaceablePendingTask),
                Box::new(|_| {}),
                admitted_at,
            ),
            &[],
        );
    }

    let limits = EditorJobLimits::default();
    let running = BTreeMap::new();
    let selected = (0..4)
        .map(|_| {
            pending
                .take_next(&limits, &running)
                .expect("ready job must be selected")
                .id
        })
        .collect::<Vec<_>>();

    assert_eq!(
        selected,
        vec![JobId::new(2), JobId::new(3), JobId::new(4), JobId::new(1)]
    );
}

#[test]
fn empty_priority_slots_do_not_probe_every_category() {
    let mut pending = PendingJobQueue::default();
    pending.insert(
        PendingJob::new(
            JobId::new(1),
            EditorJobSpec::new("background", JobCategory::Index)
                .with_priority(JobPriority::Background),
            Box::new(ReplaceablePendingTask),
            Box::new(|_| {}),
            Instant::now(),
        ),
        &[],
    );

    let baseline = pending.admission_probe_count();
    assert_eq!(
        pending
            .take_next(&EditorJobLimits::default(), &BTreeMap::new())
            .expect("background job must remain reachable")
            .id,
        JobId::new(1)
    );
    assert_eq!(
        pending.admission_probe_count() - baseline,
        JobCategory::ALL.len(),
        "empty weighted priority slots must be rejected from the maintained inventory"
    );
}

#[test]
#[ignore = "managed Editor09 performance evidence"]
fn editor09_empty_priority_slot_probe_evidence() {
    const SELECTIONS: usize = 4_096;

    let mut pending = PendingJobQueue::default();
    for id in 1..=SELECTIONS as u64 {
        pending.insert(
            PendingJob::new(
                JobId::new(id),
                EditorJobSpec::new("background", JobCategory::Index)
                    .with_priority(JobPriority::Background),
                Box::new(ReplaceablePendingTask),
                Box::new(|_| {}),
                Instant::now(),
            ),
            &[],
        );
    }

    let baseline = pending.admission_probe_count();
    let started = Instant::now();
    for _ in 0..SELECTIONS {
        assert!(pending
            .take_next(&EditorJobLimits::default(), &BTreeMap::new())
            .is_some());
    }
    let elapsed = started.elapsed();
    let probes_after = pending.admission_probe_count() - baseline;
    let probes_before = SELECTIONS * MAX_ADMISSION_BUCKET_PROBES_PER_PASS;
    let expected_probes_after = SELECTIONS * JobCategory::ALL.len();

    assert_eq!(probes_after, expected_probes_after);
    let reduction_percent = (1.0 - probes_after as f64 / probes_before as f64) * 100.0;
    println!(
        "EDITOR_JOB_BENCH_V1 kind=empty_priority_slot_probe selections={} category_probes_before={} category_probes_after={} probe_reduction_percent={:.4} elapsed_ns={}",
        SELECTIONS,
        probes_before,
        probes_after,
        reduction_percent,
        elapsed.as_nanos(),
    );
}

#[test]
fn category_blocked_background_reenters_fair_selection_after_capacity_recovers() {
    let mut pending = PendingJobQueue::default();
    pending.insert(
        PendingJob::new(
            JobId::new(1),
            EditorJobSpec::new("background", JobCategory::Index)
                .with_priority(JobPriority::Background),
            Box::new(ReplaceablePendingTask),
            Box::new(|_| {}),
            Instant::now(),
        ),
        &[],
    );
    let limits = EditorJobLimits::default().with_limit(JobCategory::Index, 1);
    let mut running = BTreeMap::new();
    running.insert(JobCategory::Index, 1);

    assert!(pending.take_next(&limits, &running).is_none());
    assert_eq!(pending.fairness_slot, 0);

    running.clear();
    assert_eq!(
        pending
            .take_next(&limits, &running)
            .expect("recovered category capacity admits the background job")
            .id,
        JobId::new(1)
    );
}

#[test]
fn dependency_blocked_background_reenters_fair_selection_after_dependency_completes() {
    let mut pending = PendingJobQueue::default();
    let dependency = JobId::new(99);
    pending.insert(
        PendingJob::new(
            JobId::new(1),
            EditorJobSpec::new("background", JobCategory::Index)
                .with_priority(JobPriority::Background),
            Box::new(ReplaceablePendingTask),
            Box::new(|_| {}),
            Instant::now(),
        ),
        &[dependency],
    );
    let limits = EditorJobLimits::default();
    let running = BTreeMap::new();

    assert!(pending.take_next(&limits, &running).is_none());
    pending.mark_dependency_schedulable(dependency);
    assert_eq!(
        pending
            .take_next(&limits, &running)
            .expect("completed dependency admits the background job")
            .id,
        JobId::new(1)
    );
}

#[test]
fn nonzero_fairness_cursor_still_selects_the_next_weighted_background_slot() {
    let mut pending = PendingJobQueue::default();
    pending.fairness_slot = 4;
    let admitted_at = Instant::now();
    pending.insert(
        PendingJob::new(
            JobId::new(1),
            EditorJobSpec::new("background", JobCategory::Index)
                .with_priority(JobPriority::Background),
            Box::new(ReplaceablePendingTask),
            Box::new(|_| {}),
            admitted_at,
        ),
        &[],
    );
    pending.insert(
        PendingJob::new(
            JobId::new(2),
            EditorJobSpec::new("interactive", JobCategory::Index)
                .with_priority(JobPriority::Interactive),
            Box::new(ReplaceablePendingTask),
            Box::new(|_| {}),
            admitted_at,
        ),
        &[],
    );

    assert_eq!(
        pending
            .take_next(&EditorJobLimits::default(), &BTreeMap::new())
            .expect("next weighted slot must be selected")
            .id,
        JobId::new(1)
    );
}
