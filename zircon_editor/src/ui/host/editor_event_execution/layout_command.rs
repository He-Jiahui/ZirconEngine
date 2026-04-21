use crate::core::editor_event::EditorEventEffect;
use crate::core::editor_event::LayoutCommand;
use crate::ui::workbench::event::ui_layout_command_from_core;

use super::execution_outcome::ExecutionOutcome;
use crate::core::editor_event::runtime::editor_event_runtime_inner::EditorEventRuntimeInner;

pub(super) fn execute_layout_command(
    inner: &mut EditorEventRuntimeInner,
    command: &LayoutCommand,
) -> Result<ExecutionOutcome, String> {
    let changed = inner
        .manager
        .apply_layout_command(ui_layout_command_from_core(command))
        .map_err(|error| error.to_string())?;
    match command {
        LayoutCommand::SavePreset { name } => inner
            .state
            .set_status_line(format!("Saved layout preset asset {name}")),
        LayoutCommand::LoadPreset { name } => inner
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
