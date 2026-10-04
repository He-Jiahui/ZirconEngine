use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use zircon_runtime::core::CoreRuntime;
use zircon_runtime::scene::DefaultLevelManager;
use zircon_runtime_interface::math::UVec2;

use crate::core::editing::engine::HistorySaveMarkOutcome;
use crate::core::gateway::DetachedEditorRuntimeGateway;
use crate::core::logging::{EditorLogService, LogFilter, LogSeverity, LogSource};
use crate::core::play::{PlayKind, PlayModeKind};
use crate::ui::host::{EditorHostEventController, EditorManager};
use crate::ui::workbench::state::EditorState;

use super::{
    execute_menu_action, project_save_completed_diagnostic, project_save_failed_diagnostic,
    project_save_started_diagnostic, MenuAction,
};

#[test]
fn stop_menu_detaches_play_gateway_without_relocking_the_shell_guard() {
    let (completed, result) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let core = CoreRuntime::new();
        let manager = std::sync::Arc::new(
            EditorManager::new(&core.handle())
                .expect("the host test should construct an editor manager"),
        );
        let state = EditorState::with_default_selection_with_context(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
            std::sync::Arc::clone(manager.context()),
        );
        let controller = EditorHostEventController::new(state, manager);
        let play_instance = controller
            .start_test_play_gateway(
                PlayKind::Play,
                std::sync::Arc::new(DetachedEditorRuntimeGateway),
            )
            .expect("the detached test gateway should attach to the Play domain");

        let mut shell = controller.shell().lock();
        shell
            .state
            .enter_play_mode()
            .expect("the retained shell should enter Play mode");
        let action_result = execute_menu_action(&controller, &mut shell, &MenuAction::ExitPlayMode);
        let stopped = action_result.is_ok()
            && controller.play_sessions().mode() == PlayModeKind::Edit
            && !shell.state.is_playing()
            && controller
                .gateway_for(crate::core::play::WorldDomain::Play(play_instance))
                .is_none();
        completed
            .send(stopped)
            .expect("the test thread should report Stop completion");
    });

    let stopped = result
        .recv_timeout(Duration::from_secs(10))
        .expect("Stop must finish while the menu already owns the shell guard");
    worker
        .join()
        .expect("the Stop menu regression test thread should complete");
    assert!(
        stopped,
        "Stop should restore Edit mode and detach the Play gateway"
    );
}

#[test]
fn project_save_diagnostics_record_the_save_generation_lifecycle() {
    let path = Path::new("C:/projects/f3 save#1");
    let started = project_save_started_diagnostic(path, true, 17, 17);
    let completed =
        project_save_completed_diagnostic(path, 17, 17, Some(17), HistorySaveMarkOutcome::Marked);

    assert!(started.contains("result=started"));
    assert!(started.contains("project=C%3A%2Fprojects%2Ff3%20save%231"));
    assert!(started.contains("pre_save_dirty=true"));
    assert!(started.contains("pre_save_dirty_generation=17"));
    assert!(started.contains("save_token_generation=17"));
    assert!(completed.contains("result=completed"));
    assert!(completed.contains("project=C%3A%2Fprojects%2Ff3%20save%231"));
    assert!(completed.contains("persisted_generation=17"));
    assert!(completed.contains("save_mark=Marked"));

    let failed = project_save_failed_diagnostic(path, "persist", "disk unavailable");
    assert!(failed.contains("result=failed"));
    assert!(failed.contains("project=C%3A%2Fprojects%2Ff3%20save%231"));
    assert!(failed.contains("phase=persist"));

    let resolve_failure = project_save_failed_diagnostic(path, "resolve_scene", "no project");
    assert!(resolve_failure.contains("phase=resolve_scene"));
}

#[test]
fn project_save_lifecycle_diagnostics_enter_the_editor_log_service() {
    let logs = EditorLogService::default();
    let diagnostic = project_save_started_diagnostic(Path::new("C:/projects/demo"), true, 9, 9);

    super::emit_project_save_log(&logs, LogSeverity::Info, diagnostic);

    let records = logs.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 1);
    let entry = records[0].entry();
    assert_eq!(entry.source(), &LogSource::editor());
    assert_eq!(entry.severity(), LogSeverity::Info);
    assert_eq!(entry.timestamp_frame(), 0);
    assert!(entry
        .message()
        .contains("editor_project_save result=started"));
    assert!(entry.message().contains("save_token_generation=9"));
}

#[test]
fn oversized_project_save_diagnostic_preserves_its_error_severity_in_the_fallback() {
    let logs = EditorLogService::default();

    super::emit_project_save_log(&logs, LogSeverity::Error, "x".repeat(9 * 1024));

    let records = logs.snapshot(&LogFilter::default());
    assert_eq!(records.len(), 1);
    let entry = records[0].entry();
    assert_eq!(entry.source(), &LogSource::editor());
    assert_eq!(entry.severity(), LogSeverity::Error);
    assert_eq!(
        entry.message(),
        "editor_project_save diagnostic exceeds the log-entry limit."
    );
}

#[cfg(windows)]
#[test]
fn project_save_diagnostics_expose_a_display_path_without_the_verbatim_prefix() {
    let diagnostic =
        project_save_started_diagnostic(Path::new(r"\\?\C:\projects\f3 save"), true, 17, 17);

    assert!(diagnostic.contains("project=C%3A%5Cprojects%5Cf3%20save"));
    assert!(!diagnostic.contains("%5C%5C%3F%5C"));
}
