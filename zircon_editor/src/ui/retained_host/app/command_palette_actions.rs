use zircon_runtime_interface::ui::component::UiValue;

use crate::core::commands::{CommandEvalCtx, EditorCommandPaletteEntry};

use super::callback_dispatch::WorkbenchCommandPaletteOpenState;
use super::{HostInvalidationMask, RetainedEditorHost};

const COMMAND_PALETTE_COMMAND_ID: &str = "editor.command.palette";

impl RetainedEditorHost {
    pub(super) fn open_workbench_command_palette(&mut self) {
        let context = self.runtime.context().command_eval().snapshot();
        let state = {
            let commands = self.runtime.commands().lock();
            workbench_command_palette_open_state(&commands, &context)
        };
        match self.workbench_window_bridge.open_command_palette(state) {
            Ok(true) => self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA),
            Ok(false) => {}
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}

fn workbench_command_palette_open_state(
    registry: &crate::core::commands::EditorCommandRegistry,
    context: &CommandEvalCtx,
) -> WorkbenchCommandPaletteOpenState {
    let entries = registry.command_palette_entries(context);
    let focused_index = focused_command_index(&entries);
    let selected_command_id = entries
        .get(usize::try_from(focused_index).unwrap_or(usize::MAX))
        .map(|entry| entry.id.clone())
        .unwrap_or_default();

    WorkbenchCommandPaletteOpenState {
        commands: UiValue::Array(
            entries
                .iter()
                .map(EditorCommandPaletteEntry::to_ui_value)
                .collect(),
        ),
        filtered_commands: UiValue::Array(
            entries
                .iter()
                .map(|entry| UiValue::String(entry.id.clone()))
                .collect(),
        ),
        selected_command_id,
        focused_index,
    }
}

fn focused_command_index(entries: &[EditorCommandPaletteEntry]) -> i64 {
    entries
        .iter()
        .enumerate()
        .find(|(_, entry)| entry.id != COMMAND_PALETTE_COMMAND_ID)
        .or_else(|| entries.iter().enumerate().next())
        .map(|(index, _)| index as i64)
        .unwrap_or(-1)
}
