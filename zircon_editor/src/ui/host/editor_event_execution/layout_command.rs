use crate::core::editor_event::EditorEventEffect;
use crate::core::editor_event::LayoutCommand;
use crate::ui::host::EditorError;
use crate::ui::workbench::event::ui_layout_command_from_core;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;

use super::execution_outcome::ExecutionOutcome;
pub(super) fn execute_layout_command(
    shell: &mut WorkbenchShellStateData,
    manager: &crate::ui::host::EditorManager,
    command: &LayoutCommand,
) -> Result<ExecutionOutcome, EditorError> {
    let ui_command = ui_layout_command_from_core(command);
    let changed = match ui_command {
        crate::ui::workbench::layout::LayoutCommand::CloseView { instance_id } => {
            manager.close_view(&instance_id)
        }
        crate::ui::workbench::layout::LayoutCommand::CloseViews {
            window_id,
            instance_ids,
        } => manager.close_views_with_discard(&window_id, &instance_ids, &[]),
        command => manager.apply_layout_command(command),
    }?;
    match command {
        LayoutCommand::SavePreset { name } => shell
            .state
            .set_status_line(format!("Saved layout preset asset {name}")),
        LayoutCommand::LoadPreset { name } => shell
            .state
            .set_status_line(format!("Loaded layout preset {name}")),
        _ => {}
    }
    Ok(ExecutionOutcome {
        changed,
        effects: vec![
            EditorEventEffect::LayoutChanged,
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    })
}
