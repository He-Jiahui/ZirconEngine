use crate::LayoutCommand;

use super::super::execution_outcome::ExecutionOutcome;
use super::super::runtime_inner::EditorEventRuntimeInner;

pub(super) fn execute_layout_command(
    inner: &mut EditorEventRuntimeInner,
    command: &LayoutCommand,
) -> Result<ExecutionOutcome, String> {
    let changed = inner
        .manager
        .apply_layout_command(command.clone())
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
            crate::editor_event::EditorEventEffect::LayoutChanged,
            crate::editor_event::EditorEventEffect::PresentationChanged,
            crate::editor_event::EditorEventEffect::ReflectionChanged,
        ],
    })
}
