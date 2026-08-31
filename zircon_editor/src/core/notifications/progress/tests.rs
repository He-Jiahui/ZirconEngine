use crate::core::jobs::{EditorJobProgress, EditorJobProgressSnapshot, JobCategory, JobId};
use crate::core::notifications::{NotificationId, NotificationSource};

use super::{
    AUTOMATIC_PROGRESS_SOURCE_ID, MAX_PROGRESS_NOTIFICATIONS, ProgressNotification,
    ProgressNotificationCenter, ProgressNotificationError,
};

fn notification(suffix: &str, job: JobId) -> ProgressNotification {
    ProgressNotification::new(
        NotificationId::parse(format!("editor.progress.{suffix}")).unwrap(),
        NotificationSource::builtin("editor17").unwrap(),
        job,
        "editor.progress.title",
    )
    .unwrap()
}

fn job(id: u64) -> EditorJobProgressSnapshot {
    EditorJobProgressSnapshot::new(
        JobId::new(id),
        "background task",
        JobCategory::Import,
        Some(EditorJobProgress::new(1, 3, "loading")),
        true,
    )
}

fn automatic_notification(job: JobId) -> ProgressNotification {
    ProgressNotification::new(
        NotificationId::parse(format!("editor.job.progress.{}", job.value())).unwrap(),
        NotificationSource::builtin(AUTOMATIC_PROGRESS_SOURCE_ID).unwrap(),
        job,
        "editor.notification.job_progress.title",
    )
    .unwrap()
}

#[test]
fn projection_tracks_one_bound_job_and_removes_terminal_entries() {
    let center = ProgressNotificationCenter::default();
    center
        .publish(notification("import", JobId::new(7)))
        .unwrap();
    assert_eq!(center.synchronize([job(7)]).len(), 1);
    assert!(
        center
            .synchronize(std::iter::empty::<EditorJobProgressSnapshot>())
            .is_empty()
    );
    assert!(center.synchronize([job(7)]).is_empty());
}

#[test]
fn one_job_has_one_progress_projection() {
    let center = ProgressNotificationCenter::default();
    center
        .publish(notification("first", JobId::new(7)))
        .unwrap();
    assert!(matches!(
        center.publish(notification("second", JobId::new(7))),
        Err(ProgressNotificationError::DuplicateJob { job }) if job == JobId::new(7)
    ));
}

#[test]
fn progress_content_key_distinguishes_empty_and_oversized_input() {
    let id = NotificationId::parse("editor.progress.invalid_content").unwrap();
    let source = NotificationSource::builtin("editor17").unwrap();
    assert!(matches!(
        ProgressNotification::new(id.clone(), source.clone(), JobId::new(7), ""),
        Err(ProgressNotificationError::EmptyField { field: "title key" })
    ));
    assert!(matches!(
        ProgressNotification::new(id, source, JobId::new(7), "a".repeat(257)),
        Err(ProgressNotificationError::FieldTooLong {
            field: "title key",
            maximum: 256,
            actual: 257,
        })
    ));
}

#[test]
fn progress_center_bounds_entries_and_releases_a_retired_job_slot() {
    let center = ProgressNotificationCenter::default();
    for index in 0..MAX_PROGRESS_NOTIFICATIONS {
        center
            .publish(notification(
                &format!("capacity_{index}"),
                JobId::new(index as u64),
            ))
            .unwrap();
    }

    assert!(matches!(
        center.publish(notification(
            "capacity_overflow",
            JobId::new(MAX_PROGRESS_NOTIFICATIONS as u64),
        )),
        Err(ProgressNotificationError::CapacityExceeded { maximum })
            if maximum == MAX_PROGRESS_NOTIFICATIONS
    ));

    center.retire_job(JobId::new(0));
    assert!(
        center
            .publish(notification(
                "capacity_released",
                JobId::new(MAX_PROGRESS_NOTIFICATIONS as u64),
            ))
            .is_ok()
    );
}

#[test]
fn manual_progress_producer_replaces_an_automatic_binding_for_its_job() {
    let center = ProgressNotificationCenter::default();
    let job_id = JobId::new(7);
    center.publish(automatic_notification(job_id)).unwrap();

    center
        .publish(
            ProgressNotification::new(
                NotificationId::parse("editor.import.progress").unwrap(),
                NotificationSource::builtin("editor.import").unwrap(),
                job_id,
                "editor.notification.import_model.title",
            )
            .unwrap(),
        )
        .unwrap();

    let snapshots = center.synchronize([job(7)]);
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].notification().source().id(), "editor.import");
    assert_eq!(
        snapshots[0].notification().title_key(),
        "editor.notification.import_model.title"
    );
}
