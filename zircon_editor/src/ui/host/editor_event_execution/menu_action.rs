use std::path::PathBuf;

use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::{EditorEventEffect, MenuAction};
use crate::ui::host::EditorHostEventController;
use crate::ui::workbench::layout::LayoutCommand;
use crate::ui::workbench::project::project_root_path;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;

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
        MenuAction::OpenScene | MenuAction::CreateScene => {
            shell
                .state
                .set_status_line("Scene open/create workflow is not wired yet");
            Ok(ExecutionOutcome {
                changed: false,
                effects: vec![
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
        MenuAction::SaveProject => {
            let path = PathBuf::from(shell.state.snapshot().project_path);
            let scene = shell
                .state
                .project_scene()
                .ok_or_else(|| "No project open".to_string())?;
            shell
                .manager
                .save_project(&path, &scene)
                .map_err(|error| error.to_string())?;
            shell.state.mark_project_open();
            shell
                .state
                .set_status_line(format!("Saved project to {}", path.display()));
            Ok(ExecutionOutcome {
                changed: true,
                effects: vec![
                    EditorEventEffect::ProjectSaveRequested,
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
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
        MenuAction::EnterPlayMode => {
            let project_root = project_root_path(&shell.state.snapshot().project_path).ok();
            let changed = shell.state.enter_play_mode()?;
            if changed {
                match controller
                    .play_bridge()
                    .backend()
                    .enter_play_mode(project_root.as_deref())
                {
                    Ok(report) => {
                        let is_clean = report.is_clean();
                        shell
                            .state
                            .sync_bridge_diagnostics_matrix(report.bridge_diagnostics.as_ref());
                        if let Err(error) = controller.begin_runtime_event_consumers() {
                            if controller.runtime_event_consumer_session_active() {
                                shell.state.set_status_line(format!(
                                    "Runtime event consumer startup cleanup failed; play mode remains active for retry: {error}"
                                ));
                                return Err(format!(
                                    "Failed to bind runtime plugin event consumers; runtime remains active so Exit Play can retry cleanup: {error}"
                                ));
                            }
                            let _ = controller.play_bridge().backend().exit_play_mode();
                            let _ = shell.state.exit_play_mode();
                            shell.state.sync_bridge_diagnostics_matrix(None);
                            return Err(format!(
                                "Failed to bind runtime plugin event consumers: {error}"
                            ));
                        }
                        if !is_clean {
                            shell.state.set_status_line(format!(
                                "Entered play mode; native runtime diagnostics: {}",
                                report.diagnostics.join("; ")
                            ));
                        }
                    }
                    Err(error) => {
                        let _ = shell.state.exit_play_mode();
                        shell.state.sync_bridge_diagnostics_matrix(None);
                        return Err(format!("Failed to enter runtime play mode: {error}"));
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
            let changed = shell.state.exit_play_mode()?;
            if changed {
                match controller.play_bridge().backend().exit_play_mode() {
                    Ok(report) => {
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
                    Err(error) => {
                        shell.state.sync_bridge_diagnostics_matrix(None);
                        shell.state.set_status_line(format!(
                            "Exited play mode; native runtime exit failed: {error}"
                        ));
                    }
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
