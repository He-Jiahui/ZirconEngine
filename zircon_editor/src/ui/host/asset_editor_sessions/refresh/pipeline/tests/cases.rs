use super::queue::UiAssetRefreshQueue;
use super::service::UiAssetWorkspaceRefreshPipeline;
use crate::core::jobs::test_job_system;
use std::collections::BTreeSet;
use std::path::Path;
use std::time::{Duration, Instant};

#[test]
fn newer_change_supersedes_active_generation() {
    let mut queue = UiAssetRefreshQueue::default();
    assert!(queue.enqueue(["res://ui/a.zui".to_string()]));
    let first = queue.start_next().expect("first generation");
    assert!(queue.enqueue(["res://ui/a.zui".to_string()]));

    assert!(queue.finish(&first));
    let second = queue.start_next().expect("replacement generation");
    assert!(second.generation > first.generation);
    assert_eq!(second.changed_asset_ids.len(), 1);
}

#[test]
fn stable_generation_does_not_schedule_work() {
    let mut queue = UiAssetRefreshQueue::default();

    assert!(!queue.enqueue(Vec::<String>::new()));
    assert!(queue.start_next().is_none());
    assert!(!queue.snapshot().active);
}

#[test]
fn fresh_file_event_resets_a_deferred_retry_generation() {
    let now = Instant::now();
    let mut queue = UiAssetRefreshQueue::default();
    queue.enqueue(["res://ui/a.zui".to_string()]);
    let request = queue.start_next_at(now).expect("request");
    assert!(!queue.finish(&request));
    assert!(queue.defer_retry_at(
        request.changed_asset_ids,
        request.retry_attempt,
        request.generation,
        now,
    ));
    assert!(queue.enqueue(["res://ui/a.zui".to_string(), "res://ui/b.zui".to_string(),]));

    let retry = queue
        .start_next_at(now)
        .expect("fresh event bypasses retry delay");
    assert_eq!(retry.retry_attempt, 0);
    assert_eq!(retry.changed_asset_ids.len(), 2);
}

#[test]
fn transient_retry_waits_for_backoff_and_stops_at_the_bound() {
    let now = Instant::now();
    let mut queue = UiAssetRefreshQueue::default();
    queue.enqueue(["res://ui/a.zui".to_string()]);

    let mut request = queue.start_next_at(now).expect("initial request");
    for expected_attempt in 1..=6 {
        assert!(!queue.finish(&request));
        assert!(queue.defer_retry_at(
            request.changed_asset_ids,
            request.retry_attempt,
            request.generation,
            now,
        ));
        assert!(queue.start_next_at(now).is_none());
        request = queue
            .start_next_at(now + Duration::from_secs(3))
            .expect("bounded retry request");
        assert_eq!(request.retry_attempt, expected_attempt);
    }

    assert!(!queue.finish(&request));
    assert!(!queue.defer_retry_at(
        request.changed_asset_ids,
        request.retry_attempt,
        request.generation,
        now,
    ));
    assert_eq!(queue.snapshot().exhausted_retry_count, 1);
    assert!(queue.start_next_at(now + Duration::from_secs(30)).is_none());
}

#[test]
fn project_epoch_reset_discards_active_pending_and_deferred_work() {
    let now = Instant::now();
    let mut queue = UiAssetRefreshQueue::default();
    queue.enqueue(["res://ui/active.zui".to_string()]);
    let active = queue.start_next_at(now).expect("active request");
    queue.enqueue(["res://ui/pending.zui".to_string()]);
    assert!(queue.finish(&active));
    let pending = queue.start_next_at(now).expect("pending request");
    assert!(!queue.finish(&pending));
    assert!(queue.defer_retry_at(
        pending.changed_asset_ids,
        pending.retry_attempt,
        pending.generation,
        now,
    ));

    queue.reset_project_epoch();

    let snapshot = queue.snapshot();
    assert!(!snapshot.active);
    assert_eq!(snapshot.pending_asset_count, 0);
    assert!(queue.start_next_at(now + Duration::from_secs(30)).is_none());
}

#[test]
fn retry_attempts_remain_asset_local_cohorts() {
    let now = Instant::now();
    let mut queue = UiAssetRefreshQueue::default();
    assert!(queue.defer_retry_at(
        ["res://ui/early.zui".to_string()].into_iter().collect(),
        0,
        1,
        now,
    ));
    assert!(queue.defer_retry_at(
        ["res://ui/exhausted.zui".to_string()].into_iter().collect(),
        5,
        1,
        now,
    ));

    let early = queue
        .start_next_at(now + Duration::from_secs(3))
        .expect("first retry cohort");
    assert_eq!(early.retry_attempt, 1);
    assert_eq!(
        early.changed_asset_ids,
        ["res://ui/early.zui".to_string()].into_iter().collect()
    );
    assert!(!queue.finish(&early));

    let exhausted = queue
        .start_next_at(now + Duration::from_secs(3))
        .expect("second retry cohort");
    assert_eq!(exhausted.retry_attempt, 6);
    assert_eq!(
        exhausted.changed_asset_ids,
        ["res://ui/exhausted.zui".to_string()].into_iter().collect()
    );
}

#[test]
fn newer_generation_replaces_older_retry_state_for_the_same_asset() {
    let now = Instant::now();
    let asset_ids: BTreeSet<String> = ["res://ui/recovered.zui".to_string()].into_iter().collect();
    let mut queue = UiAssetRefreshQueue::default();
    assert!(queue.defer_retry_at(asset_ids.clone(), 5, 1, now));
    assert!(queue.defer_retry_at(asset_ids, 0, 2, now));

    let retry = queue
        .start_next_at(now + Duration::from_secs(3))
        .expect("newer retry generation");
    assert_eq!(retry.retry_attempt, 1);
}

#[test]
fn same_project_watcher_restart_preserves_refresh_work() {
    let mut pipeline = UiAssetWorkspaceRefreshPipeline::new(test_job_system());
    assert!(pipeline.transition_project(Some(Path::new(r"C:\project-a"))));
    assert!(pipeline.enqueue(["res://ui/pending.zui".to_string()]));

    assert!(!pipeline.transition_project(Some(Path::new(r"C:\project-a"))));
    assert_eq!(pipeline.snapshot().pending_asset_count, 1);

    assert!(pipeline.transition_project(Some(Path::new(r"C:\project-b"))));
    assert_eq!(pipeline.snapshot().pending_asset_count, 0);
}
