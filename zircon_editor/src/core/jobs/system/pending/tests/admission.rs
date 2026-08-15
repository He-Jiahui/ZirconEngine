use std::time::{Duration, Instant};

use super::super::*;
use super::ReplaceablePendingTask;

#[test]
fn category_snapshot_keeps_entry_bytes_and_oldest_age_in_sync_through_merge_and_removal() {
    let mut pending = PendingJobQueue::default();
    let admitted_at = Instant::now();
    let export_key = EditorJobAdmissionKey::new("category-snapshot:export").unwrap();
    let export_first = EditorJobSpec::new("export-first", JobCategory::Export)
        .with_admission_key(export_key.clone())
        .with_estimated_bytes(5);
    pending.insert(
        PendingJob::new(
            JobId::new(1),
            export_first,
            Box::new(ReplaceablePendingTask),
            Box::new(|_| {}),
            admitted_at,
        ),
        &[],
    );
    pending.insert(
        PendingJob::new(
            JobId::new(2),
            EditorJobSpec::new("index", JobCategory::Index).with_estimated_bytes(7),
            Box::new(ReplaceablePendingTask),
            Box::new(|_| {}),
            admitted_at + Duration::from_secs(1),
        ),
        &[],
    );
    pending.insert(
        PendingJob::new(
            JobId::new(3),
            EditorJobSpec::new("export-second", JobCategory::Export).with_estimated_bytes(11),
            Box::new(ReplaceablePendingTask),
            Box::new(|_| {}),
            admitted_at + Duration::from_secs(2),
        ),
        &[],
    );

    let now = admitted_at + Duration::from_secs(10);
    let export = pending.category_admission_snapshot(JobCategory::Export, now);
    let index = pending.category_admission_snapshot(JobCategory::Index, now);
    assert_eq!(export.pending_entries(), 2);
    assert_eq!(export.pending_estimated_bytes(), 16);
    assert_eq!(export.oldest_pending_age(), Some(Duration::from_secs(10)));
    assert_eq!(index.pending_entries(), 1);
    assert_eq!(index.pending_estimated_bytes(), 7);
    assert_eq!(index.oldest_pending_age(), Some(Duration::from_secs(9)));

    let latest_export = EditorJobSpec::new("export-latest", JobCategory::Export)
        .with_admission_key(export_key)
        .with_estimated_bytes(13);
    assert_eq!(
        pending
            .merge_pending_admission(
                JobId::new(1),
                &latest_export,
                Box::new(ReplaceablePendingTask),
                EditorJobAdmissionLimits::new(4, 64, Duration::from_secs(60)),
                now,
            )
            .unwrap(),
        JobId::new(1)
    );
    let export_after_merge = pending.category_admission_snapshot(JobCategory::Export, now);
    assert_eq!(export_after_merge.pending_entries(), 2);
    assert_eq!(export_after_merge.pending_estimated_bytes(), 24);
    assert_eq!(
        export_after_merge.oldest_pending_age(),
        Some(Duration::from_secs(10))
    );

    pending.remove(JobId::new(1));
    let export_after_remove = pending.category_admission_snapshot(JobCategory::Export, now);
    assert_eq!(export_after_remove.pending_entries(), 1);
    assert_eq!(export_after_remove.pending_estimated_bytes(), 11);
    assert_eq!(
        export_after_remove.oldest_pending_age(),
        Some(Duration::from_secs(8))
    );
}

#[test]
fn drain_updates_global_and_category_cancelled_counters_together() {
    let mut pending = PendingJobQueue::default();
    for (id, category) in [
        (JobId::new(1), JobCategory::Export),
        (JobId::new(2), JobCategory::Index),
        (JobId::new(3), JobCategory::Index),
    ] {
        pending.insert(
            PendingJob::new(
                id,
                EditorJobSpec::new(format!("drain-{id:?}"), category),
                Box::new(|_| {}),
                Box::new(|_| {}),
                Instant::now(),
            ),
            &[],
        );
    }

    assert_eq!(pending.drain().len(), 3);

    let now = Instant::now();
    let global = pending.admission_snapshot(now);
    let export = pending.category_admission_snapshot(JobCategory::Export, now);
    let index = pending.category_admission_snapshot(JobCategory::Index, now);
    assert_eq!(global.pending_entries(), 0);
    assert_eq!(global.cancelled_pending(), 3);
    assert_eq!(export.cancelled_pending(), 1);
    assert_eq!(index.cancelled_pending(), 2);
    assert_eq!(
        global.cancelled_pending(),
        export.cancelled_pending() + index.cancelled_pending()
    );
}
