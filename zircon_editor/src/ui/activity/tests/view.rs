use std::time::Duration;

use crate::core::i18n::{EditorI18nService, EditorLocale};
use crate::core::jobs::{EditorJobProgress, EditorJobProgressSnapshot, JobCategory, JobId};
use crate::core::logging::{
    EditorLogService, LogEntry, LogFilter, LogJump, LogSeverity, LogSource,
};
use crate::core::notifications::{
    NotificationId, NotificationSource, ProgressNotification, ProgressNotificationCenter,
    ToastCenterConfig, ToastNotification, ToastNotificationCenter, ToastSeverity,
};

use super::{activity_log_views, activity_progress_views, activity_toast_views};

#[test]
fn activity_toast_views_localize_immutable_core_snapshots() {
    let center = ToastNotificationCenter::new(ToastCenterConfig::default());
    center
        .publish_at(
            ToastNotification::new(
                NotificationId::parse("editor.activity.save").unwrap(),
                NotificationSource::builtin("editor.activity").unwrap(),
                ToastSeverity::Success,
                "editor.notification.project_saved.title",
                "editor.notification.project_saved.message",
                Duration::from_secs(3),
            )
            .unwrap(),
            Duration::ZERO,
        )
        .unwrap();
    let i18n = EditorI18nService::default();
    i18n.set_active_locale(EditorLocale::parse("zh-CN").unwrap())
        .unwrap();

    let views = activity_toast_views(&center.snapshot_at(Duration::ZERO), &i18n, Duration::ZERO);

    assert_eq!(views.len(), 1);
    assert_eq!(views[0].id(), "editor.activity.save");
    assert_eq!(views[0].title(), "项目已保存");
    assert_eq!(views[0].message(), "项目状态已写入磁盘。");
    assert_eq!(views[0].severity(), ToastSeverity::Success);
    assert_eq!(views[0].expires_at(), Duration::from_secs(3));
    assert_eq!(views[0].remaining_lifetime(), Duration::from_secs(3));
}

#[test]
fn activity_progress_views_localize_active_core_job_snapshots() {
    let center = ProgressNotificationCenter::default();
    let job = JobId::new(7);
    center
        .publish(
            ProgressNotification::new(
                NotificationId::parse("editor.activity.import-progress").unwrap(),
                NotificationSource::builtin("editor.activity").unwrap(),
                job,
                "editor.notification.import_completed.title",
            )
            .unwrap(),
        )
        .unwrap();
    let i18n = EditorI18nService::default();
    i18n.set_active_locale(EditorLocale::parse("zh-CN").unwrap())
        .unwrap();

    let views = activity_progress_views(
        &center.synchronize([EditorJobProgressSnapshot::new(
            job,
            "Importing terrain",
            JobCategory::Import,
            Some(EditorJobProgress::new(3, 4, "Converting materials")),
            true,
        )]),
        &i18n,
    );

    assert_eq!(views.len(), 1);
    assert_eq!(views[0].id(), "editor.activity.import-progress");
    assert_eq!(views[0].job_id(), job);
    assert_eq!(views[0].title(), "模型已导入");
    assert_eq!(views[0].detail(), "Converting materials");
    assert_eq!(views[0].percent(), Some(75));
}

#[test]
fn activity_log_views_preserve_the_core_record_and_jump_target() {
    let logs = EditorLogService::default();
    let jump = LogJump::asset("assets/terrain.material")
        .expect("a nonempty asset locator should be a valid jump target");
    logs.emit(
        LogEntry::new(
            LogSource::editor(),
            LogSeverity::Warning,
            "material fallback selected",
            42,
            Some(jump.clone()),
        )
        .expect("a bounded log entry should construct"),
    )
    .expect("the activity fixture should enter the log store");

    let records = logs.snapshot(&LogFilter::default());
    let views = activity_log_views(&records);

    assert_eq!(views.len(), 1);
    assert_eq!(views[0].sequence(), records[0].sequence());
    assert_eq!(views[0].source(), &LogSource::editor());
    assert_eq!(views[0].severity(), LogSeverity::Warning);
    assert_eq!(views[0].message(), "material fallback selected");
    assert_eq!(views[0].timestamp_frame(), 42);
    assert_eq!(views[0].jump(), Some(&jump));
}
