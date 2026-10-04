use std::cell::RefCell;
use std::path::Path;

use crate::core::logging::{EditorLogService, LogFilter, LogSeverity, LogSource};

use super::{
    emit_project_log, finish_project_runtime_retirement, post_persist_project_save_sync,
    preflight_project_runtime_retirement, project_close_post_commit_sync_diagnostic,
    project_opened_diagnostic, project_save_post_persist_sync_diagnostic, ProjectRuntimeCloseError,
    ProjectRuntimeRetirementDisposition, ProjectRuntimeRetirementRequirement,
};

#[test]
fn project_open_diagnostic_records_the_catalog_generation() {
    let diagnostic = project_opened_diagnostic(
        Path::new("C:\\F1 Project"),
        "F1 Save",
        1,
        "res://scenes/main.scene.toml",
        7,
        7,
        0,
        2,
        3,
        9,
        7,
        "persisted-v1",
    );

    assert!(diagnostic.starts_with("editor_project_open result=completed"));
    assert!(diagnostic.contains("project_root=C%3A%5CF1%20Project"));
    assert!(diagnostic.contains("manifest_identity=F1%20Save%40v1"));
    assert!(diagnostic.contains("scene_uri=res%3A%2F%2Fscenes%2Fmain.scene.toml"));
    assert!(diagnostic.contains("registry_asset_count=7"));
    assert!(diagnostic.contains("registry_ready_asset_count=7"));
    assert!(diagnostic.contains("registry_failed_asset_count=0"));
    assert!(diagnostic.contains("registry_diagnostic_count=2"));
    assert!(diagnostic.contains("project_generation=3"));
    assert!(diagnostic.contains("project_generation_publish_epoch=9"));
    assert!(diagnostic.contains("catalog_asset_count=7"));
    assert!(diagnostic.contains("settings_source=persisted-v1"));
}

#[cfg(windows)]
#[test]
fn project_open_diagnostic_uses_a_display_path_for_verbatim_roots() {
    let diagnostic = project_opened_diagnostic(
        Path::new(r"\\?\C:\F1 Project"),
        "F1 Save",
        1,
        "res://scenes/main.scene.toml",
        7,
        7,
        0,
        2,
        3,
        9,
        7,
        "persisted-v1",
    );

    assert!(diagnostic.contains("project_root=C%3A%5CF1%20Project"));
    assert!(!diagnostic.contains("%3F%5C"));
}

#[test]
fn project_open_diagnostic_marks_failed_asset_imports_as_degraded() {
    let diagnostic = project_opened_diagnostic(
        Path::new("C:\\F1 Project"),
        "Broken F1 Asset",
        1,
        "res://scenes/main.scene.toml",
        7,
        6,
        1,
        2,
        3,
        9,
        7,
        "persisted-v1",
    );

    assert!(diagnostic.starts_with("editor_project_open result=degraded"));
    assert!(diagnostic.contains("registry_ready_asset_count=6"));
    assert!(diagnostic.contains("registry_failed_asset_count=1"));
}

#[test]
fn project_open_diagnostic_marks_incomplete_asset_registry_as_degraded() {
    let diagnostic = project_opened_diagnostic(
        Path::new("C:\\F1 Project"),
        "Incomplete F1 Registry",
        1,
        "res://scenes/main.scene.toml",
        7,
        6,
        0,
        1,
        3,
        9,
        7,
        "persisted-v1",
    );

    assert!(diagnostic.starts_with("editor_project_open result=degraded"));
    assert!(diagnostic.contains("registry_asset_count=7"));
    assert!(diagnostic.contains("registry_ready_asset_count=6"));
    assert!(diagnostic.contains("registry_failed_asset_count=0"));
}

#[test]
fn project_open_diagnostic_marks_fallback_settings_as_degraded() {
    let diagnostic = project_opened_diagnostic(
        Path::new("C:\\F1 Project"),
        "Fallback F1 Settings",
        1,
        "res://scenes/main.scene.toml",
        7,
        7,
        0,
        0,
        3,
        9,
        7,
        "degraded-missing",
    );

    assert!(diagnostic.starts_with("editor_project_open result=degraded"));
    assert!(diagnostic.contains("settings_source=degraded-missing"));
}

#[test]
fn post_persist_save_sync_failure_is_diagnostic_only() {
    let logs = EditorLogService::default();
    assert_eq!(
        post_persist_project_save_sync(&logs, "reimport_active_scene", Ok::<_, &str>(7)),
        Some(7)
    );
    assert_eq!(
        post_persist_project_save_sync(
            &logs,
            "refresh_editor_assets",
            Err::<(), _>("catalog stale"),
        ),
        None
    );

    let diagnostic =
        project_save_post_persist_sync_diagnostic("refresh_editor_assets", "catalog stale");
    assert!(diagnostic.contains("result=post_persist_sync_failed"));
    assert!(diagnostic.contains("phase=refresh_editor_assets"));
    assert!(diagnostic.contains("error=catalog stale"));
}

#[test]
fn committed_project_failures_enter_the_shared_editor_log() {
    let logs = EditorLogService::default();

    assert_eq!(
        post_persist_project_save_sync(
            &logs,
            "refresh_editor_assets",
            Err::<(), _>("catalog stale"),
        ),
        None
    );
    emit_project_log(
        &logs,
        LogSeverity::Error,
        project_close_post_commit_sync_diagnostic(
            "stop_ui_asset_workspace_watcher",
            &"watcher unavailable",
        ),
    );

    let records = logs.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 2);
    assert!(records.iter().all(|record| {
        record.entry().source() == &LogSource::editor()
            && record.entry().severity() == LogSeverity::Error
            && record.entry().timestamp_frame() == 0
    }));
    assert!(records[0]
        .entry()
        .message()
        .starts_with("editor_project_save result=post_persist_sync_failed"));
    assert!(records[1]
        .entry()
        .message()
        .starts_with("editor_project_close result=post_commit_sync_failed"));
}

#[test]
fn oversized_project_open_diagnostic_preserves_its_info_severity_in_the_fallback() {
    let logs = EditorLogService::default();

    emit_project_log(&logs, LogSeverity::Info, "x".repeat(9 * 1024));

    let records = logs.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 1);
    let entry = records[0].entry();
    assert_eq!(entry.source(), &LogSource::editor());
    assert_eq!(entry.severity(), LogSeverity::Info);
    assert_eq!(
        entry.message(),
        "editor_project_access diagnostic exceeds the log-entry limit."
    );
}

#[test]
fn committed_project_close_deactivates_projection_before_stopping_the_watcher() {
    let calls = RefCell::new(Vec::new());

    let receipt = finish_project_runtime_retirement(
        Some(Path::new("C:/projects/forest").to_path_buf()),
        Path::new("C:/projects/forest"),
        ProjectRuntimeRetirementRequirement::ExactActiveRoot,
        || {
            calls.borrow_mut().push("deactivate");
            true
        },
        || {
            calls.borrow_mut().push("watcher");
            Ok::<_, &str>(())
        },
    )
    .expect("all runtime close owners should commit");

    assert_eq!(receipt.closed_root(), Some(Path::new("C:/projects/forest")));
    assert_eq!(
        receipt.disposition(),
        ProjectRuntimeRetirementDisposition::ClosedActive
    );
    assert_eq!(calls.borrow().as_slice(), ["deactivate", "watcher"]);
}

#[test]
fn project_close_without_an_active_runtime_project_is_not_a_terminal_receipt() {
    let calls = RefCell::new(Vec::new());

    let error = finish_project_runtime_retirement(
        None,
        Path::new("C:/projects/forest"),
        ProjectRuntimeRetirementRequirement::ExactActiveRoot,
        || {
            calls.borrow_mut().push("deactivate");
            true
        },
        || {
            calls.borrow_mut().push("watcher");
            Ok::<_, &str>(())
        },
    )
    .expect_err("a guarded close must retire one exact runtime project");

    assert!(matches!(
        error,
        ProjectRuntimeCloseError::RuntimeProjectMissing
    ));
    assert!(calls.borrow().is_empty());
}

#[test]
fn committed_project_close_rejects_watcher_failure_as_non_terminal() {
    let calls = RefCell::new(Vec::new());

    let error = finish_project_runtime_retirement(
        Some(Path::new("C:/projects/forest").to_path_buf()),
        Path::new("C:/projects/forest"),
        ProjectRuntimeRetirementRequirement::ExactActiveRoot,
        || {
            calls.borrow_mut().push("deactivate");
            true
        },
        || {
            calls.borrow_mut().push("watcher");
            Err::<(), _>("watcher unavailable")
        },
    )
    .expect_err("watcher transition is part of the runtime close owner");

    assert!(matches!(
        error,
        ProjectRuntimeCloseError::WatcherTransitionFailed { .. }
    ));
    assert_eq!(calls.borrow().as_slice(), ["deactivate", "watcher"]);
}

#[test]
fn committed_project_close_accepts_an_already_empty_editor_projection() {
    let calls = RefCell::new(Vec::new());
    let receipt = finish_project_runtime_retirement(
        Some(Path::new("C:/projects/forest").to_path_buf()),
        Path::new("C:/projects/forest"),
        ProjectRuntimeRetirementRequirement::ExactActiveRoot,
        || {
            calls.borrow_mut().push("deactivate");
            false
        },
        || {
            calls.borrow_mut().push("watcher");
            Ok::<_, &str>(())
        },
    )
    .expect("an already empty projection is a terminal retirement state");

    assert_eq!(receipt.closed_root(), Some(Path::new("C:/projects/forest")));
    assert_eq!(
        receipt.disposition(),
        ProjectRuntimeRetirementDisposition::AlreadyEmpty
    );
    assert_eq!(calls.borrow().as_slice(), ["deactivate", "watcher"]);
}

#[test]
fn activation_rollback_accepts_an_already_absent_runtime_owner() {
    let calls = RefCell::new(Vec::new());
    let receipt = finish_project_runtime_retirement(
        None,
        Path::new("C:/projects/forest"),
        ProjectRuntimeRetirementRequirement::AllowAlreadyAbsent,
        || {
            calls.borrow_mut().push("deactivate");
            false
        },
        || {
            calls.borrow_mut().push("watcher");
            Ok::<_, &str>(())
        },
    )
    .expect("activation compensation is terminal when every owner is already absent");

    assert_eq!(receipt.closed_root(), None);
    assert_eq!(
        receipt.disposition(),
        ProjectRuntimeRetirementDisposition::AlreadyAbsent
    );
    assert_eq!(calls.borrow().as_slice(), ["deactivate", "watcher"]);
}

#[test]
fn committed_project_close_rejects_a_different_runtime_root_before_projection_teardown() {
    let calls = RefCell::new(Vec::new());
    let error = finish_project_runtime_retirement(
        Some(Path::new("C:/projects/other").to_path_buf()),
        Path::new("C:/projects/forest"),
        ProjectRuntimeRetirementRequirement::ExactActiveRoot,
        || {
            calls.borrow_mut().push("deactivate");
            true
        },
        || {
            calls.borrow_mut().push("watcher");
            Ok::<_, &str>(())
        },
    )
    .expect_err("a close capability cannot consume another project root");

    assert!(matches!(
        error,
        ProjectRuntimeCloseError::ClosedRootMismatch { .. }
    ));
    assert!(calls.borrow().is_empty());
}

#[test]
fn runtime_retirement_preflight_rejects_missing_and_mismatched_active_roots() {
    assert_eq!(
        preflight_project_runtime_retirement(
            None,
            Path::new("C:/projects/forest"),
            ProjectRuntimeRetirementRequirement::ExactActiveRoot,
        ),
        Err(ProjectRuntimeCloseError::RuntimeProjectMissing)
    );
    assert!(matches!(
        preflight_project_runtime_retirement(
            Some(Path::new("C:/projects/other")),
            Path::new("C:/projects/forest"),
            ProjectRuntimeRetirementRequirement::ExactActiveRoot,
        ),
        Err(ProjectRuntimeCloseError::ClosedRootMismatch { .. })
    ));
    assert_eq!(
        preflight_project_runtime_retirement(
            None,
            Path::new("C:/projects/forest"),
            ProjectRuntimeRetirementRequirement::AllowAlreadyAbsent,
        ),
        Ok(())
    );
}

#[test]
fn project_close_entry_points_preflight_before_mutating_the_runtime_owner() {
    let source = include_str!("../project_access.rs");
    for (entry, next_entry) in [
        (
            "pub(super) fn close_project(",
            "pub(super) fn roll_back_project_activation(",
        ),
        (
            "pub(super) fn roll_back_project_activation(",
            "pub(super) fn save_active_scene(",
        ),
    ] {
        let start = source.find(entry).expect("project close entry point");
        let end = source[start..]
            .find(next_entry)
            .map(|offset| start + offset)
            .expect("project close entry boundary");
        let body = &source[start..end];
        let preflight = body
            .find("preflight_project_runtime_retirement(")
            .expect("runtime retirement preflight");
        let destructive_close = body
            .find("asset_manager.close_project()?")
            .expect("runtime close mutation");
        assert!(preflight < destructive_close);
    }
}
