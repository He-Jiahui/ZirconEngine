use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

use zircon_runtime::asset::project::ProjectPaths;

use crate::core::editing::engine::HistorySaveMarkOutcome;
use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::{EditorEventEffect, MenuAction};
use crate::core::logging::{EditorLogService, LogEntry, LogSeverity, LogSource};
use crate::core::play::{
    PlayKind, PlayModeKind, PlaySceneSource, PlayStartRequest, PlayTransitionCause, WorldDomain,
};
use crate::ui::host::{EditorHostEventController, RuntimeEventConsumerShutdownDisposition};
use crate::ui::workbench::layout::LayoutCommand;
use crate::ui::workbench::project::project_root_path;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;

use super::super::project_access::percent_encode_diagnostic_token;
use super::common::{effects_when, open_view, scene_effects, scene_intent_event};
use super::error::MenuActionExecutionError;
use super::execution_outcome::ExecutionOutcome;
pub(super) fn execute_menu_action(
    controller: &EditorHostEventController,
    shell: &mut WorkbenchShellStateData,
    action: &MenuAction,
) -> Result<ExecutionOutcome, MenuActionExecutionError> {
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
            let history_context = shell.state.scene_history_context()?;
            let transactions = shell.state.transactions();
            let pre_save_dirty = transactions.is_dirty(history_context).map_err(|source| {
                MenuActionExecutionError::Transaction {
                    phase: "query dirtiness",
                    source,
                }
            })?;
            let pre_save_dirty_generation = transactions
                .history_generation_snapshot(history_context)
                .map_err(|source| MenuActionExecutionError::Transaction {
                    phase: "query generation",
                    source,
                })?;
            let save_token = shell
                .state
                .transactions()
                .capture_save_token(history_context)
                .map_err(|source| MenuActionExecutionError::Transaction {
                    phase: "capture save token",
                    source,
                })?;
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
            let scene = match shell
                .state
                .project_scene()
                .map_err(crate::ui::workbench::state::EditorStateOperationError::from)?
            {
                Some(scene) => scene,
                None => {
                    emit_project_save_log(
                        controller.context().logs(),
                        LogSeverity::Error,
                        project_save_failed_diagnostic(
                            &path,
                            "resolve_scene",
                            &MenuActionExecutionError::NoProjectOpen,
                        ),
                    );
                    return Err(MenuActionExecutionError::NoProjectOpen);
                }
            };
            let live_scene_views = shell
                .manager
                .view_instance_ids_for_descriptor_key("editor.scene")
                .into_iter()
                .map(|instance_id| crate::core::editor_event::ViewInstanceId::new(instance_id.0))
                .collect::<std::collections::BTreeSet<_>>();
            let mut workspace = shell.manager.project_workspace();
            workspace.scene_viewport_sessions = shell
                .state
                .viewport_controller
                .snapshot_workspace_sessions(&live_scene_views)
                .into_iter()
                .map(|(instance_id, snapshot)| {
                    (
                        crate::ui::workbench::view::ViewInstanceId::new(instance_id.0),
                        snapshot,
                    )
                })
                .collect();
            if let Err(source) = shell
                .manager
                .save_active_scene_with_workspace(&path, &scene, &workspace)
            {
                emit_project_save_log(
                    controller.context().logs(),
                    LogSeverity::Error,
                    project_save_failed_diagnostic(&path, "persist", &source),
                );
                return Err(source.into());
            }
            let save_mark = match transactions.mark_saved_if_unchanged(history_context, save_token)
            {
                Ok(outcome) => outcome,
                Err(source) => {
                    emit_project_save_log(
                        controller.context().logs(),
                        LogSeverity::Error,
                        project_save_failed_diagnostic(&path, "mark_saved", &source),
                    );
                    return Err(MenuActionExecutionError::Transaction {
                        phase: "mark saved",
                        source,
                    });
                }
            };
            let persisted_generation = transactions
                .history_generation_snapshot(history_context)
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
        MenuAction::SaveAllDocuments => Ok(ExecutionOutcome {
            changed: false,
            effects: vec![
                EditorEventEffect::DocumentSaveAllRequested,
                EditorEventEffect::PresentationChanged,
                EditorEventEffect::ReflectionChanged,
            ],
        }),
        MenuAction::CloseProject => Ok(ExecutionOutcome {
            changed: false,
            effects: vec![
                EditorEventEffect::ProjectCloseRequested,
                EditorEventEffect::PresentationChanged,
                EditorEventEffect::ReflectionChanged,
            ],
        }),
        MenuAction::SaveLayout => {
            shell.manager.save_global_default_layout()?;
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
                .apply_layout_command(LayoutCommand::ResetToDefault)?;
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
            let changed = cleared_logs != 0 || cleared_legacy_output;
            Ok(ExecutionOutcome {
                changed,
                effects: effects_when(changed, [EditorEventEffect::PresentationChanged]),
            })
        }
        MenuAction::SetConsoleMessageFilter(filter) => {
            let changed = shell.set_console_message_filter(*filter);
            Ok(ExecutionOutcome {
                changed,
                effects: effects_when(changed, [EditorEventEffect::PresentationChanged]),
            })
        }
        MenuAction::SetConsoleSourceFilter(filter) => {
            let changed = shell.set_console_source_filter(*filter);
            Ok(ExecutionOutcome {
                changed,
                effects: effects_when(changed, [EditorEventEffect::PresentationChanged]),
            })
        }
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
            let scene = shell
                .state
                .project_scene()
                .map_err(crate::ui::workbench::state::EditorStateOperationError::from)?
                .ok_or(MenuActionExecutionError::NoProjectOpen)?;
            let scene_source = PlaySceneSource::from_world(&scene)?;
            let changed = shell.state.enter_play_mode()?;
            if changed {
                let play_kind = controller.play_sessions().preferred_kind();
                match controller.play_sessions().request_play(
                    PlayStartRequest::immediate(play_kind, project_root.as_deref())
                        .with_scene_source(scene_source),
                ) {
                    Ok(transition) => {
                        if let Some(WorldDomain::Play(instance)) =
                            controller.play_sessions().attached_world_domain()
                        {
                            shell.state.activate_play_selection_domain(instance);
                        }
                        let backend_attachable = transition.backend_attachable;
                        let backend_diagnostics = transition.backend_diagnostics;
                        controller.log_play_backend_diagnostics(&backend_diagnostics);
                        let report = transition.activation;
                        let is_clean = report.is_clean();
                        shell
                            .state
                            .sync_bridge_diagnostics_matrix(report.bridge_diagnostics.as_ref());
                        if backend_attachable {
                            let enabled_capabilities = shell
                                .manager
                                .capability_snapshot()
                                .enabled_capabilities()
                                .to_vec();
                            if let Err(source) = controller
                                .begin_runtime_event_consumers_with_capabilities(
                                    &enabled_capabilities,
                                )
                            {
                                if controller.runtime_event_consumer_session_active() {
                                    shell.state.set_status_line(format!(
                                        "Runtime event consumer startup cleanup failed; play mode remains active for retry: {source}"
                                    ));
                                    return Err(MenuActionExecutionError::RuntimeConsumerStart {
                                        source,
                                    });
                                }
                                if let Err(stop_error) = controller.play_sessions().request_stop() {
                                    shell.state.set_status_line(format!(
                                        "Runtime event consumer startup and play session cleanup failed; play mode remains active for retry: {stop_error}"
                                    ));
                                    return Err(
                                        MenuActionExecutionError::RuntimeConsumerStartStopFailed {
                                            source,
                                            stop: stop_error,
                                        },
                                    );
                                }
                                if let Err(detach) =
                                    controller.detach_terminal_play_gateway_with_shell(shell)
                                {
                                    return Err(MenuActionExecutionError::RuntimeConsumerStartGatewayDetachFailed {
                                        source,
                                        detach,
                                    });
                                }
                                let retirement = match controller
                                    .play_sessions()
                                    .retire_terminal_backend()
                                {
                                    Ok(retirement) => retirement,
                                    Err(stop) => {
                                        return Err(
                                            MenuActionExecutionError::RuntimeConsumerStartStopFailed {
                                                source,
                                                stop,
                                            },
                                        );
                                    }
                                };
                                controller
                                    .log_play_backend_diagnostics(&retirement.backend_diagnostics);
                                if let Err(exit_error) = shell.state.exit_play_mode() {
                                    shell.state.set_status_line(format!(
                                        "Runtime event consumer startup failed after play session stopped; editor state remains in play mode for retry: {exit_error}"
                                    ));
                                    return Err(
                                        MenuActionExecutionError::RuntimeConsumerStartRestoreStateFailed {
                                            source,
                                            restore: exit_error,
                                        },
                                    );
                                }
                                shell.state.sync_bridge_diagnostics_matrix(None);
                                return Err(MenuActionExecutionError::RuntimeConsumerStart {
                                    source,
                                });
                            }
                        }
                        let preview_focus_error =
                            if backend_attachable && play_kind == PlayKind::Play {
                                shell.focus_play_preview_view().err()
                            } else {
                                None
                            };
                        if let Some(error) = preview_focus_error {
                            shell.state.set_status_line(format!(
                                "Entered play mode, but the Game view could not be focused: {error}"
                            ));
                        } else if !is_clean {
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
                    Err(source) => {
                        if controller.play_sessions().mode().has_active_runtime() {
                            if let Some(WorldDomain::Play(instance)) =
                                controller.play_sessions().attached_world_domain()
                            {
                                shell.state.activate_play_selection_domain(instance);
                            }
                            shell.state.set_status_line(format!(
                                "Play session startup did not complete, but its runtime is still active for stop retry: {source}"
                            ));
                            return Err(MenuActionExecutionError::PlayStart { source });
                        }
                        if let Err(exit_error) = shell.state.exit_play_mode() {
                            shell.state.set_status_line(format!(
                                "Play session startup failed; editor state remains in play mode for retry: {exit_error}"
                            ));
                            return Err(MenuActionExecutionError::PlayStartRestoreStateFailed {
                                source,
                                restore: exit_error,
                            });
                        }
                        shell.state.sync_bridge_diagnostics_matrix(None);
                        return Err(MenuActionExecutionError::PlayStart { source });
                    }
                }
            }
            Ok(ExecutionOutcome {
                changed,
                effects: scene_effects(),
            })
        }
        MenuAction::ExitPlayMode => {
            let trace_sequence = next_play_stop_trace_sequence();
            trace_play_stop_stage(trace_sequence, "stop_menu_entry", None, None);
            let consumer_shutdown_started =
                trace_play_stop_stage(trace_sequence, "consumer_shutdown_enter", None, None);
            let consumer_shutdown = controller.shutdown_runtime_event_consumers();
            let consumer_shutdown_result = match &consumer_shutdown {
                RuntimeEventConsumerShutdownDisposition::NotActive => "not_active",
                RuntimeEventConsumerShutdownDisposition::Retired => "retired",
                RuntimeEventConsumerShutdownDisposition::RetiredWithCleanupFailure { .. } => {
                    "cleanup_failure"
                }
                RuntimeEventConsumerShutdownDisposition::RetirementDeferred { .. } => "deferred",
            };
            trace_play_stop_stage(
                trace_sequence,
                "consumer_shutdown_return",
                Some(consumer_shutdown_result),
                consumer_shutdown_started,
            );
            let remote_consumer_cleanup_failure = match consumer_shutdown {
                RuntimeEventConsumerShutdownDisposition::NotActive
                | RuntimeEventConsumerShutdownDisposition::Retired => None,
                RuntimeEventConsumerShutdownDisposition::RetiredWithCleanupFailure { error } => {
                    Some(error.to_string())
                }
                RuntimeEventConsumerShutdownDisposition::RetirementDeferred { error } => {
                    shell.state.set_status_line(format!(
                            "Runtime event consumer retirement is deferred; play mode remains active for retry: {error}"
                        ));
                    return Err(MenuActionExecutionError::RuntimeConsumerStop { source: error });
                }
            };
            let stop_request_started =
                trace_play_stop_stage(trace_sequence, "play_session_stop_enter", None, None);
            let transition = match controller.play_sessions().request_stop() {
                Ok(transition) => {
                    trace_play_stop_stage(
                        trace_sequence,
                        "play_session_stop_return",
                        Some("ok"),
                        stop_request_started,
                    );
                    transition
                }
                Err(source) => {
                    trace_play_stop_stage(
                        trace_sequence,
                        "play_session_stop_return",
                        Some("error"),
                        stop_request_started,
                    );
                    shell.state.set_status_line(format!(
                        "Play session cleanup failed; play mode remains active for retry: {source}"
                    ));
                    return Err(MenuActionExecutionError::PlayStop { source });
                }
            };
            let gateway_detach_started =
                trace_play_stop_stage(trace_sequence, "gateway_detach_enter", None, None);
            if let Err(source) = controller.detach_terminal_play_gateway_with_shell(shell) {
                trace_play_stop_stage(
                    trace_sequence,
                    "gateway_detach_return",
                    Some("error"),
                    gateway_detach_started,
                );
                return Err(MenuActionExecutionError::PlayGatewayDetach { source });
            }
            trace_play_stop_stage(
                trace_sequence,
                "gateway_detach_return",
                Some("ok"),
                gateway_detach_started,
            );
            let backend_retire_started =
                trace_play_stop_stage(trace_sequence, "backend_retire_enter", None, None);
            let retirement = match controller.play_sessions().retire_terminal_backend() {
                Ok(retirement) => {
                    trace_play_stop_stage(
                        trace_sequence,
                        "backend_retire_return",
                        Some("ok"),
                        backend_retire_started,
                    );
                    retirement
                }
                Err(source) => {
                    trace_play_stop_stage(
                        trace_sequence,
                        "backend_retire_return",
                        Some("error"),
                        backend_retire_started,
                    );
                    return Err(MenuActionExecutionError::PlayStop { source });
                }
            };
            controller.log_play_backend_diagnostics(&retirement.backend_diagnostics);
            let changed = shell.state.exit_play_mode().map_err(|source| {
                shell.state.sync_bridge_diagnostics_matrix(None);
                shell.state.set_status_line(format!(
                    "Play session stopped, but editor state remains in play mode for retry: {source}"
                ));
                MenuActionExecutionError::PlayStopRestoreStateFailed { source }
            })?;
            let preview_restore_error = shell.restore_pre_play_view().err();
            controller.reconcile_pending_play_decision_from_controller()?;
            let report = transition.activation.clone();
            let is_clean = report.is_clean();
            shell
                .state
                .sync_bridge_diagnostics_matrix(report.bridge_diagnostics.as_ref());
            let remote_cleanup_suffix = remote_consumer_cleanup_failure
                .as_deref()
                .map(|error| format!("; runtime event subscription cleanup is pending: {error}"))
                .unwrap_or_default();
            let terminal_transition =
                if matches!(retirement.cause, PlayTransitionCause::CleanupFailed { .. }) {
                    &retirement
                } else {
                    &transition
                };
            match &terminal_transition.cause {
                PlayTransitionCause::CleanupFailed { failure } => {
                    shell.state.set_status_line(format!(
                        "Runtime preview stopped, but cleanup is pending: {failure}{remote_cleanup_suffix}"
                    ));
                }
                _ if changed && !is_clean => {
                    shell.state.set_status_line(format!(
                        "Exited play mode; native runtime diagnostics: {}{remote_cleanup_suffix}",
                        report.diagnostics.join("; "),
                    ));
                }
                _ if changed => {
                    if let Some(error) = remote_consumer_cleanup_failure {
                        shell.state.set_status_line(format!(
                            "Exited play mode; runtime event subscription cleanup is pending: {error}"
                        ));
                    }
                }
                _ if transition.changed && transition.mode == PlayModeKind::Edit => {
                    shell
                        .state
                        .set_status_line("Runtime preview cleanup completed");
                }
                _ => {}
            }
            if let Some(error) = preview_restore_error {
                shell.state.set_status_line(format!(
                    "Exited play mode, but the previous editor view could not be restored: {error}"
                ));
            }
            Ok(ExecutionOutcome {
                changed,
                effects: scene_effects(),
            })
        }
        MenuAction::KeepPlayChanges => {
            let changed = shell.state.keep_play_changes()?;
            Ok(ExecutionOutcome {
                changed,
                effects: scene_effects(),
            })
        }
        MenuAction::Undo => replay_focused_history_or_scene(shell, true),
        MenuAction::Redo => replay_focused_history_or_scene(shell, false),
        MenuAction::CreateNode(kind) => Ok(scene_intent_event(
            shell,
            EditorIntent::CreateNode(kind.clone()),
        )?),
        MenuAction::DeleteSelected => {
            let changed = shell.state.delete_selected()?;
            Ok(ExecutionOutcome {
                changed,
                effects: scene_effects(),
            })
        }
        MenuAction::OpenView(descriptor_id) => Ok(open_view(
            shell,
            descriptor_id.0.as_str(),
            &format!("Opened view {}", descriptor_id.0),
        )?),
    }
}

fn replay_focused_history_or_scene(
    shell: &mut WorkbenchShellStateData,
    undo: bool,
) -> Result<ExecutionOutcome, MenuActionExecutionError> {
    // Play mode freezes both scene and document authoring. Preserve the shared guard before
    // resolving a document-specific history context.
    if shell.state.is_playing() {
        return Ok(scene_intent_event(
            shell,
            if undo {
                EditorIntent::Undo
            } else {
                EditorIntent::Redo
            },
        )?);
    }
    let Some(changed) = shell
        .manager
        .replay_focused_animation_document_history(undo)?
    else {
        return Ok(scene_intent_event(
            shell,
            if undo {
                EditorIntent::Undo
            } else {
                EditorIntent::Redo
            },
        )?);
    };
    shell.state.set_status_line(if changed {
        if undo {
            "Undo"
        } else {
            "Redo"
        }
    } else if undo {
        "Nothing to undo"
    } else {
        "Nothing to redo"
    });
    Ok(ExecutionOutcome {
        changed,
        effects: vec![
            crate::core::editor_event::EditorEventEffect::PresentationChanged,
            crate::core::editor_event::EditorEventEffect::ReflectionChanged,
        ],
    })
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

fn next_play_stop_trace_sequence() -> Option<u64> {
    static TRACE_ENABLED: OnceLock<bool> = OnceLock::new();
    let enabled = *TRACE_ENABLED.get_or_init(|| {
        std::env::var("ZIRCON_TRACE_PLAY_STOP")
            .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true"))
    });
    if !enabled {
        return None;
    }
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    Some(SEQUENCE.fetch_add(1, Ordering::Relaxed) + 1)
}

fn trace_play_stop_stage(
    sequence: Option<u64>,
    stage: &str,
    result: Option<&str>,
    started: Option<Instant>,
) -> Option<Instant> {
    let Some(sequence) = sequence else {
        return None;
    };
    match result {
        Some(result) => {
            if let Some(started) = started {
                eprintln!(
                    "mvp_play_trace component=editor_stop_menu seq={sequence} stage={stage} result={result} elapsed_us={}",
                    started.elapsed().as_micros()
                );
            } else {
                eprintln!(
                    "mvp_play_trace component=editor_stop_menu seq={sequence} stage={stage} result={result}"
                );
            }
            None
        }
        None => {
            eprintln!("mvp_play_trace component=editor_stop_menu seq={sequence} stage={stage}");
            Some(Instant::now())
        }
    }
}

#[cfg(test)]
#[path = "tests/menu_action.rs"]
mod tests;
