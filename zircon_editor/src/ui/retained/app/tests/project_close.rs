use std::path::Path;

use crate::ui::workbench::startup::{EditorSessionMode, EditorStartupSessionDocument};

use super::{welcome_session_after_project_close, RetainedProjectCloseError};

#[test]
fn successful_close_returns_to_welcome_without_retaining_project_navigation() {
    let mut session = EditorStartupSessionDocument::default();
    session.mode = EditorSessionMode::Project;
    session.open_builtin_view = Some("editor.scene".to_string());

    let welcome =
        welcome_session_after_project_close(session, Some(Path::new("C:/projects/forest")));

    assert_eq!(welcome.mode, EditorSessionMode::Welcome);
    assert!(welcome.project.is_none());
    assert!(welcome.open_builtin_view.is_none());
    assert_eq!(welcome.status_message, "Closed project C:/projects/forest");
}

#[test]
fn close_enters_durable_closing_and_retires_focus_before_quiescing_consumers() {
    let source = include_str!("../project_close.rs");
    let close_start = source
        .find("fn commit_project_close(")
        .expect("retained close entry point");
    let close_end = source[close_start..]
        .find("#[cfg(test)]")
        .map(|offset| close_start + offset)
        .expect("retained close test boundary");
    let close = &source[close_start..close_end];
    let begin = close
        .find(".begin_project_close()")
        .expect("durable close admission");
    let focus_sync = close
        .find("self.sync_hub_focus_binding()")
        .expect("focus binding retirement");
    let asset_quiesce = close
        .find("self.cancel_pending_asset_deletion()")
        .expect("asset quiescence");
    let manager_close = close
        .find("self.editor_manager.commit_project_close(&operation)")
        .expect("manager close result");

    assert!(begin < focus_sync);
    assert!(focus_sync < asset_quiesce);
    assert!(asset_quiesce < manager_close);
    assert!(close.contains("ProjectSessionEffect::FocusBinding"));
}

#[cfg(windows)]
#[test]
fn successful_close_projects_operation_roots_to_a_display_path() {
    let welcome = welcome_session_after_project_close(
        EditorStartupSessionDocument::default(),
        Some(Path::new(r"\\?\C:\projects\forest")),
    );

    assert_eq!(welcome.status_message, r"Closed project C:\projects\forest");
}

#[test]
fn retry_after_committed_runtime_close_still_repairs_the_welcome_surface() {
    let mut session = EditorStartupSessionDocument::default();
    session.mode = EditorSessionMode::Project;
    session.open_builtin_view = Some("editor.asset_browser".to_string());

    let welcome = welcome_session_after_project_close(session, None);

    assert_eq!(welcome.mode, EditorSessionMode::Welcome);
    assert!(welcome.project.is_none());
    assert!(welcome.open_builtin_view.is_none());
    assert_eq!(
        welcome.status_message,
        "Project was already closed; restored the welcome workspace."
    );
}

#[test]
fn project_close_runs_play_teardown_before_releasing_the_project_session() {
    let source = include_str!("../project_close.rs");
    let close = source
        .split("fn commit_project_close(&mut self)")
        .nth(1)
        .expect("project-close commit implementation");
    let play_shutdown = close
        .find("shutdown_play_session_for_project_close()")
        .expect("project close must retire Play before project release");
    let project_commit = close
        .find(".commit_project_close(&operation)")
        .expect("project close must commit the manager release");

    assert!(play_shutdown < project_commit);
    assert!(close.contains("is_ready_for_project_close()"));
    assert!(close.contains("pending_edit_decision_prompt()"));
}

#[test]
fn terminal_entry_points_delegate_project_close_to_retained_host() {
    let direct_manager_close = ["editor_manager", "commit_project_close(&"].join(".");

    for source in [
        include_str!("../../app.rs"),
        include_str!("../automation.rs"),
    ] {
        assert!(source.contains("host\n        .borrow_mut()\n        .commit_project_close()"));
        assert!(!source.contains(&direct_manager_close));
    }
}

#[test]
fn project_close_errors_keep_terminal_failure_classes_typed() {
    assert!(matches!(
        RetainedProjectCloseError::PendingPlayEditDecision,
        RetainedProjectCloseError::PendingPlayEditDecision
    ));
    assert!(matches!(
        RetainedProjectCloseError::WelcomeWorkspace {
            message: "welcome failed".to_string(),
        },
        RetainedProjectCloseError::WelcomeWorkspace { .. }
    ));
}
