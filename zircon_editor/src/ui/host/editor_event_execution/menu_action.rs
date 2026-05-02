use std::path::PathBuf;

use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::{EditorEventEffect, MenuAction};
use crate::ui::workbench::layout::LayoutCommand;
use crate::ui::workbench::project::project_root_path;

use super::common::{open_view, scene_effects, scene_intent_event};
use super::execution_outcome::ExecutionOutcome;
use crate::core::editor_event::runtime::editor_event_runtime_inner::EditorEventRuntimeInner;

pub(super) fn execute_menu_action(
    inner: &mut EditorEventRuntimeInner,
    action: &MenuAction,
) -> Result<ExecutionOutcome, String> {
    match action {
        MenuAction::OpenProject => {
            inner
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
            inner
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
            let path = PathBuf::from(inner.state.snapshot().project_path);
            let scene = inner
                .state
                .project_scene()
                .ok_or_else(|| "No project open".to_string())?;
            inner
                .manager
                .save_project(&path, &scene)
                .map_err(|error| error.to_string())?;
            inner.state.mark_project_open();
            inner
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
            inner
                .manager
                .save_global_default_layout()
                .map_err(|error| error.to_string())?;
            inner.state.set_status_line("Saved global default layout");
            Ok(ExecutionOutcome {
                changed: false,
                effects: vec![
                    EditorEventEffect::PresentationChanged,
                    EditorEventEffect::ReflectionChanged,
                ],
            })
        }
        MenuAction::ResetLayout => {
            let changed = inner
                .manager
                .apply_layout_command(LayoutCommand::ResetToDefault)
                .map_err(|error| error.to_string())?;
            inner.state.set_status_line("Reset layout");
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
            let project_root = project_root_path(&inner.state.snapshot().project_path).ok();
            let changed = inner.state.enter_play_mode()?;
            if changed {
                match inner
                    .runtime_play_mode_backend
                    .enter_play_mode(project_root.as_deref())
                {
                    Ok(report) if report.is_clean() => {}
                    Ok(report) => inner.state.set_status_line(format!(
                        "Entered play mode; native runtime diagnostics: {}",
                        report.diagnostics.join("; ")
                    )),
                    Err(error) => {
                        let _ = inner.state.exit_play_mode();
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
            let changed = inner.state.exit_play_mode()?;
            if changed {
                match inner.runtime_play_mode_backend.exit_play_mode() {
                    Ok(report) if report.is_clean() => {}
                    Ok(report) => inner.state.set_status_line(format!(
                        "Exited play mode; native runtime diagnostics: {}",
                        report.diagnostics.join("; ")
                    )),
                    Err(error) => inner.state.set_status_line(format!(
                        "Exited play mode; native runtime exit failed: {error}"
                    )),
                }
            }
            Ok(ExecutionOutcome {
                changed,
                effects: scene_effects(),
            })
        }
        MenuAction::Undo => scene_intent_event(inner, EditorIntent::Undo),
        MenuAction::Redo => scene_intent_event(inner, EditorIntent::Redo),
        MenuAction::CreateNode(kind) => {
            scene_intent_event(inner, EditorIntent::CreateNode(kind.clone()))
        }
        MenuAction::DeleteSelected => {
            let changed = inner.state.delete_selected()?;
            Ok(ExecutionOutcome {
                changed,
                effects: scene_effects(),
            })
        }
        MenuAction::OpenView(descriptor_id) => open_view(
            inner,
            descriptor_id.0.as_str(),
            &format!("Opened view {}", descriptor_id.0),
        ),
    }
}
