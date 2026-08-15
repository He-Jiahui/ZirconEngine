use std::path::{Path, PathBuf};

use zircon_runtime::asset::project::ProjectPaths;

use crate::core::editing::engine::{HistoryContextId, HistorySaveMarkOutcome};
use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::{EditorEventEffect, MenuAction};
use crate::core::logging::{EditorLogService, LogEntry, LogSeverity, LogSource};
use crate::core::play::{PlayKind, PlaySceneSource, PlayStartRequest};
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::layout::LayoutCommand;
use crate::ui::workbench::project::project_root_path;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;

use super::super::project_access::percent_encode_diagnostic_token;
use super::common::{open_view, scene_effects, scene_intent_event};
use super::execution_outcome::ExecutionOutcome;
pub(super) fn execute_menu_action(
    controller: &EditorHostEventController,
    shell: &mut WorkbenchShellStateData,
    action: &MenuAction,
) -> Result<ExecutionOutcome, String> {
    match action {
        MenuAction::OpenProject => {
            shell
                .state
                .set_status_line("Open an existing project or create a renderable empty project.");
            Ok(ExecutionOutcome {
                changed: false,
                effects: vec![
                    EditorEventEffect::PresentWelcomeRequested,
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
        MenuAction::OpenScene => Ok(ExecutionOutcome {
            changed: false,
            effects: vec![
                EditorEventEffect::OpenScenePickerRequested,
                EditorEventEffect::PresentationChanged,
                EditorEventEffect::ReflectionChanged,
            ],
        }),
        MenuAction::CreateScene => Ok(ExecutionOutcome {
            changed: false,
            effects: vec![
                EditorEventEffect::CreateScenePickerRequested,
                EditorEventEffect::PresentationChanged,
                EditorEventEffect::ReflectionChanged,
            ],
        }),
        MenuAction::SaveProject => {
            let path = PathBuf::from(shell.state.snapshot().project_path);
            let transactions = shell.state.transactions();
            let pre_save_dirty = transactions
                .is_dirty(HistoryContextId::Global)
                .map_err(|error| error.to_string())?;
            let pre_save_dirty_generation = transactions
                .history_generation_snapshot(HistoryContextId::Global)
                .map_err(|error| error.to_string())?;
            let save_token = shell
                .state
                .transactions()
                .capture_save_token(HistoryContextId::Global)
                .map_err(|error| error.to_string())?;
            let save_token_generation = save_token.generation();
            emit_project_save_log(
                controller.context().logs(),
                LogSeverity::Info,
                project_save_started_diagnostic(
                    &path,
                    pre_save_dirty,
                    pre_save_dirty_generation,
                    save_token_generation,
                ),
            );
            let scene = match shell.state.project_scene() {
                Some(scene) => scene,
                None => {
                    let error = "No project open";
                    emit_project_save_log(
                        controller.context().logs(),
                        LogSeverity::Error,
                        project_save_failed_diagnostic(&path, "resolve_scene", error),
                    );
                    return Err(error.to_string());
                }
            };
            if let Err(error) = shell.manager.save_project(&path, &scene) {
                emit_project_save_log(
                    controller.context().logs(),
                    LogSeverity::Error,
                    project_save_failed_diagnostic(&path, "persist", &error),
                );
                return Err(error.to_string());
            }
            let save_mark =
                match transactions.mark_saved_if_unchanged(HistoryContextId::Global, save_token) {
                    Ok(outcome) => outcome,
                    Err(error) => {
                        emit_project_save_log(
                            controller.context().logs(),
                            LogSeverity::Error,
                            project_save_failed_diagnostic(&path, "mark_saved", &error),
                        );
                        return Err(error.to_string());
                    }
                };
            let persisted_generation = transactions
                .history_generation_snapshot(HistoryContextId::Global)
                .ok();
            emit_project_save_log(
                controller.context().logs(),
                LogSeverity::Info,
                project_save_completed_diagnostic(
                    &path,
                    pre_save_dirty_generation,
                    save_token_generation,
                    persisted_generation,
                    save_mark,
                ),
            );
            shell.state.mark_project_open();
            shell.state.set_status_line(format!(
                "Saved project to {}",
                ProjectPaths::display_path(&path).display()
            ));
            Ok(ExecutionOutcome {
                changed: true,
                effects: vec![
                    EditorEventEffect::ProjectSaveRequested,
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
        MenuAction::CloseProject => Ok(ExecutionOutcome {
            changed: false,
            effects: vec![
                EditorEventEffect::ProjectCloseRequested,
                EditorEventEffect::PresentationChanged,
                EditorEventEffect::ReflectionChanged,
            ],
        }),
        MenuAction::SaveLayout => {
            shell
                .manager
                .save_global_default_layout()
                .map_err(|error| error.to_string())?;
            shell.state.set_status_line("Saved global default layout");
            Ok(ExecutionOutcome {
                changed: false,
                effects: vec![
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
        MenuAction::ResetLayout => {
            let changed = shell
                .manager
                .apply_layout_command(LayoutCommand::ResetToDefault)
                .map_err(|error| error.to_string())?;
            shell.state.set_status_line("Reset layout");
            Ok(ExecutionOutcome {
                changed,
                effects: vec![
                    EditorEventEffect::LayoutChanged,
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
        MenuAction::ClearConsole => {
            let cleared_logs = controller.context().logs().clear();
            let cleared_legacy_output = shell.state.clear_console_history();
            Ok(ExecutionOutcome {
                changed: cleared_logs != 0 || cleared_legacy_output,
                effects: vec![EditorEventEffect::PresentationChanged],
            })
        }
        MenuAction::SetConsoleMessageFilter(filter) => Ok(ExecutionOutcome {
            changed: shell.set_console_message_filter(*filter),
            effects: vec![EditorEventEffect::PresentationChanged],
        }),
        MenuAction::SetConsoleSourceFilter(filter) => Ok(ExecutionOutcome {
            changed: shell.set_console_source_filter(*filter),
            effects: vec![EditorEventEffect::PresentationChanged],
        }),
        MenuAction::SelectPlayMode(kind) => {
            let changed = controller.play_sessions().set_preferred_kind(*kind);
            let label = match kind {
                PlayKind::Play => "Play In Editor",
                PlayKind::Simulate => "Simulate",
            };
            shell
                .state
                .set_status_line(format!("Run mode set to {label}"));
            Ok(ExecutionOutcome {
                changed,
                effects: vec![
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
        MenuAction::EnterPlayMode => {
            let project_root = project_root_path(&shell.state.snapshot().project_path).ok();
            let scene_source = shell
                .state
                .project_scene()
                .ok_or_else(|| "Cannot enter play without an open scene".to_string())
                .and_then(|scene| PlaySceneSource::from_world(&scene))?;
            let changed = shell.state.enter_play_mode()?;
            if changed {
                let play_kind = controller.play_sessions().preferred_kind();
                match controller.play_sessions().request_play(
                    PlayStartRequest::immediate(play_kind, project_root.as_deref())
                        .with_scene_source(scene_source),
                ) {
                    Ok(transition) => {
                        let backend_attachable = transition.backend_attachable;
                        let backend_diagnostics = transition.backend_diagnostics;
                        controller.log_play_backend_diagnostics(&backend_diagnostics);
                        let report = transition.activation;
                        let is_clean = report.is_clean();
                        shell
                            .state
                            .sync_bridge_diagnostics_matrix(report.bridge_diagnostics.as_ref());
                        if backend_attachable {
                            if let Err(error) = controller.begin_runtime_event_consumers() {
                                if controller.runtime_event_consumer_session_active() {
                                    shell.state.set_status_line(format!(
                                        "Runtime event consumer startup cleanup failed; play mode remains active for retry: {error}"
                                    ));
                                    return Err(format!(
                                        "Failed to bind runtime plugin event consumers; runtime remains active so Exit Play can retry cleanup: {error}"
                                    ));
                                }
                                if let Err(stop_error) = controller.play_sessions().request_stop() {
                                    shell.state.set_status_line(format!(
                                        "Runtime event consumer startup and play session cleanup failed; play mode remains active for retry: {stop_error}"
                                    ));
                                    return Err(format!(
                                        "Failed to bind runtime plugin event consumers: {error}; failed to stop play session, runtime remains active: {stop_error}"
                                    ));
                                }
                                if let Err(exit_error) = shell.state.exit_play_mode() {
                                    shell.state.set_status_line(format!(
                                        "Runtime event consumer startup failed after play session stopped; editor state remains in play mode for retry: {exit_error}"
                                    ));
                                    return Err(format!(
                                        "Failed to bind runtime plugin event consumers: {error}; play session stopped but failed to restore editor state: {exit_error}"
                                    ));
                                }
                                shell.state.sync_bridge_diagnostics_matrix(None);
                                return Err(format!(
                                    "Failed to bind runtime plugin event consumers: {error}"
                                ));
                            }
                        }
                        if !is_clean {
                            shell.state.set_status_line(format!(
                                "Entered play mode; native runtime diagnostics: {}",
                                report.diagnostics.join("; ")
                            ));
                        } else if !backend_diagnostics.is_empty() {
                            shell.state.set_status_line(format!(
                                "Entered play mode: {}",
                                backend_diagnostics.join("; ")
                            ));
                        }
                    }
                    Err(error) => {
                        if let Err(exit_error) = shell.state.exit_play_mode() {
                            shell.state.set_status_line(format!(
                                "Play session startup failed; editor state remains in play mode for retry: {exit_error}"
                            ));
                            return Err(format!(
                                "Failed to enter play session: {error}; backend startup failed but editor state could not be restored: {exit_error}"
                            ));
                        }
                        shell.state.sync_bridge_diagnostics_matrix(None);
                        return Err(format!("Failed to enter play session: {error}"));
                    }
                }
            }
            Ok(ExecutionOutcome {
                changed,
                effects: scene_effects(),
            })
        }
        MenuAction::ExitPlayMode => {
            if controller.runtime_event_consumer_session_active() {
                if let Err(error) = controller.end_runtime_event_consumers() {
                    shell.state.set_status_line(format!(
                        "Runtime event consumer cleanup failed; play mode remains active for retry: {error}"
                    ));
                    return Err(format!(
                        "Failed to clean up runtime event consumers; runtime remains active for retry: {error}"
                    ));
                }
            }
            let transition = controller.play_sessions().request_stop().map_err(|error| {
                shell.state.set_status_line(format!(
                    "Play session cleanup failed; play mode remains active for retry: {error}"
                ));
                format!("Failed to stop play session; runtime remains active: {error}")
            })?;
            let changed = shell.state.exit_play_mode()?;
            controller
                .publish_pending_edit_decision(transition.pending_edit_prompt.as_ref())
                .map_err(|error| {
                    format!("failed to publish pending play-edit decision: {error}")
                })?;
            if changed {
                let report = transition.activation;
                let is_clean = report.is_clean();
                shell
                    .state
                    .sync_bridge_diagnostics_matrix(report.bridge_diagnostics.as_ref());
                if !is_clean {
                    shell.state.set_status_line(format!(
                        "Exited play mode; native runtime diagnostics: {}",
                        report.diagnostics.join("; ")
                    ));
                }
            }
            Ok(ExecutionOutcome {
                changed,
                effects: scene_effects(),
            })
        }
        MenuAction::Undo => scene_intent_event(shell, EditorIntent::Undo),
        MenuAction::Redo => scene_intent_event(shell, EditorIntent::Redo),
        MenuAction::CreateNode(kind) => {
            scene_intent_event(shell, EditorIntent::CreateNode(kind.clone()))
        }
        MenuAction::DeleteSelected => {
            let changed = shell.state.delete_selected()?;
            Ok(ExecutionOutcome {
                changed,
                effects: scene_effects(),
            })
        }
        MenuAction::OpenView(descriptor_id) => open_view(
            shell,
            descriptor_id.0.as_str(),
            &format!("Opened view {}", descriptor_id.0),
        ),
    }
}

// Saving runs outside a retained-host render-frame callback, so no frame is available to record.
const UNKNOWN_PROJECT_SAVE_LOG_FRAME: u64 = 0;

fn emit_project_save_log(logs: &EditorLogService, severity: LogSeverity, message: String) {
    let entry = LogEntry::new(
        LogSource::editor(),
        severity,
        message,
        UNKNOWN_PROJECT_SAVE_LOG_FRAME,
        None,
    )
    .or_else(|_| {
        LogEntry::new(
            LogSource::editor(),
            severity,
            "editor_project_save diagnostic exceeds the log-entry limit.",
            UNKNOWN_PROJECT_SAVE_LOG_FRAME,
            None,
        )
    });
    if let Ok(entry) = entry {
        let _ = logs.emit(entry);
    }
}

fn project_save_started_diagnostic(
    path: &Path,
    pre_save_dirty: bool,
    pre_save_dirty_generation: u64,
    save_token_generation: u64,
) -> String {
    let project = project_save_display_token(path);
    format!(
        "editor_project_save result=started project={project} pre_save_dirty={pre_save_dirty} pre_save_dirty_generation={pre_save_dirty_generation} save_token_generation={save_token_generation}",
    )
}

fn project_save_completed_diagnostic(
    path: &Path,
    pre_save_dirty_generation: u64,
    save_token_generation: u64,
    persisted_generation: Option<u64>,
    save_mark: HistorySaveMarkOutcome,
) -> String {
    let persisted_generation = persisted_generation
        .map(|generation| generation.to_string())
        .unwrap_or_else(|| "unavailable".to_string());
    let project = project_save_display_token(path);
    format!(
        "editor_project_save result=completed project={project} pre_save_dirty_generation={pre_save_dirty_generation} save_token_generation={save_token_generation} persisted_generation={persisted_generation} save_mark={save_mark:?}",
    )
}

fn project_save_failed_diagnostic(
    path: &Path,
    phase: &str,
    error: &(impl std::fmt::Display + ?Sized),
) -> String {
    let project = project_save_display_token(path);
    format!("editor_project_save result=failed project={project} phase={phase} error={error}",)
}

fn project_save_display_token(path: &Path) -> String {
    let display_path = ProjectPaths::display_path(path);
    percent_encode_diagnostic_token(&display_path.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::core::editing::engine::HistorySaveMarkOutcome;
    use crate::core::logging::{EditorLogService, LogFilter, LogSeverity, LogSource};

    use super::{
        project_save_completed_diagnostic, project_save_failed_diagnostic,
        project_save_started_diagnostic,
    };

    #[test]
    fn project_save_diagnostics_record_the_save_generation_lifecycle() {
        let path = Path::new("C:/projects/f3 save#1");
        let started = project_save_started_diagnostic(path, true, 17, 17);
        let completed = project_save_completed_diagnostic(
            path,
            17,
            17,
            Some(17),
            HistorySaveMarkOutcome::Marked,
        );

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
}
