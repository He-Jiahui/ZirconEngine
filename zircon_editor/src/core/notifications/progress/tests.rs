use crate::core::jobs::{EditorJobProgress, EditorJobProgressSnapshot, JobCategory, JobId};
use crate::core::notifications::{NotificationId, NotificationSource};

use super::{ProgressNotification, ProgressNotificationCenter, ProgressNotificationError};

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
