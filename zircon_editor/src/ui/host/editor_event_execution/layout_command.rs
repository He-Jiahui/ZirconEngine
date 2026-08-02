use crate::core::editor_event::EditorEventEffect;
use crate::core::editor_event::LayoutCommand;
use crate::ui::workbench::event::ui_layout_command_from_core;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;

use super::execution_outcome::ExecutionOutcome;
pub(super) fn execute_layout_command(
    shell: &mut WorkbenchShellStateData,
    command: &LayoutCommand,
) -> Result<ExecutionOutcome, String> {
    let ui_command = ui_layout_command_from_core(command);
    let changed = match ui_command {
        crate::ui::workbench::layout::LayoutCommand::CloseView { instance_id } => {
            shell.manager.close_view(&instance_id)
        }
        command => shell.manager.apply_layout_command(command),
    }
    .map_err(|error| error.to_string())?;
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
