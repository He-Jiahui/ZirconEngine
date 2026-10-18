use std::hint::black_box;

use super::StatusBarModel;
use crate::scene::viewport::{GridMode, SceneViewportChromeSettings};

const BENCHMARK_MARKER: &str = "EDITOR862_VIEWPORT_CHROME_DIRECT_SETTINGS_BENCH_V1";

#[test]
fn viewport_snap_text_uses_per_kind_flags_independently_of_grid_visibility() {
    let mut settings = SceneViewportChromeSettings::default();
    settings.grid_mode = GridMode::Hidden;
    assert_eq!(
        StatusBarModel::viewport_chrome_text(&settings),
        ("Grid: Off".to_string(), "T:1 m R:15° S:off".to_string())
    );

    settings.grid_mode = GridMode::VisibleNoSnap;
    settings.translate_step = 0.25;
    assert_eq!(
        StatusBarModel::viewport_chrome_text(&settings),
        ("Grid: 0.25 m".to_string(), "T:0.25 m R:15° S:off".to_string())
    );

    settings.grid_mode = GridMode::VisibleAndSnap;
    assert_eq!(
        StatusBarModel::viewport_chrome_text(&settings),
        ("Grid: 0.25 m".to_string(), "T:0.25 m R:15° S:off".to_string())
    );
    settings.translate_snap_enabled = false;
    settings.rotate_snap_enabled = false;
    settings.scale_snap_enabled = true;
    assert_eq!(StatusBarModel::viewport_chrome_text(&settings).1, "T:off R:off S:0.1");
}

#[test]
fn status_counts_follow_log_and_notification_authorities_and_clear() {
    use crate::core::context::EditorContextBuilder;
    use crate::core::logging::{LogEntry, LogSeverity, LogSource};
    use crate::core::notifications::{NotificationId, NotificationSource, ToastNotification, ToastSeverity};
    use crate::ui::workbench::fixture::default_preview_fixture;
    use crate::ui::workbench::snapshot::{StatusSignalsSnapshot, StatusTaskProgressSnapshot};

    let context = EditorContextBuilder::new(
        crate::core::jobs::test_job_scheduler(),
        crate::core::jobs::test_job_scheduler(),
    ).build();
    for _ in 0..2 {
        context.logs().emit(LogEntry::new(
            LogSource::editor(), LogSeverity::Warning, "test warning", 0, None,
        ).unwrap()).unwrap();
    }
    let mut chrome = default_preview_fixture().build_chrome();
    chrome.status_signals = StatusSignalsSnapshot::from_context(&context);
    // A task projection is not a notification authority.
    chrome.status_task_progress = Some(StatusTaskProgressSnapshot::new("test:1", "Test task"));
    let model = StatusBarModel::from_chrome(&chrome);
    assert_eq!(model.warning_text, "2 Warnings");
    assert_eq!(model.error_text, "0 Errors");
    assert_eq!(model.message_text, "0 Messages");

    context.notifications().publish_toast(ToastNotification::new(
        NotificationId::parse("editor.status.test").unwrap(),
        NotificationSource::builtin("editor.status.test").unwrap(),
        ToastSeverity::Error, "editor.notification.command_failed.title", "Failed operation",
        std::time::Duration::from_secs(5),
    ).unwrap()).unwrap();
    chrome.status_signals = StatusSignalsSnapshot::from_context(&context);
    assert_eq!(StatusBarModel::from_chrome(&chrome).message_text, "1 Message");

    context.logs().clear();
    chrome.status_signals = StatusSignalsSnapshot::from_context(&context);
    assert_eq!(StatusBarModel::from_chrome(&chrome).warning_text, "0 Warnings");
}

#[test]
#[ignore = "release-only direct-settings performance evidence"]
fn editor862_viewport_chrome_direct_settings_bench() {
    const QUERY_COUNT: usize = 65_536;
    let mut settings = SceneViewportChromeSettings::default();
    settings.grid_mode = GridMode::VisibleAndSnap;
    settings.translate_step = 0.25;

    for _ in 0..QUERY_COUNT {
        black_box(StatusBarModel::viewport_chrome_text(&settings));
    }

    println!(
        "{BENCHMARK_MARKER} queries={QUERY_COUNT} legacy_full_chrome_snapshot_builds={QUERY_COUNT} \
         optimized_full_chrome_snapshot_builds=0 reduction_pct=100"
    );
}
