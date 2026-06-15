use zircon_runtime_interface::ui::component::UiValue;

use crate::ui::host::{EditorCommandContext, EditorCommandPaletteEntry, EditorCommandRegistry};
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::startup::EditorSessionMode;

use super::callback_dispatch::WorkbenchCommandPaletteOpenState;
use super::{HostInvalidationMask, RetainedEditorHost};

const COMMAND_PALETTE_COMMAND_ID: &str = "editor.command_palette";

impl RetainedEditorHost {
    pub(super) fn open_workbench_command_palette(&mut self) {
        let state = workbench_command_palette_open_state(&self.build_chrome());
        match self.workbench_window_bridge.open_command_palette(state) {
            Ok(true) => self.invalidate_host(HostInvalidationMask::PRESENTATION_DATA),
            Ok(false) => {}
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}

fn workbench_command_palette_open_state(
    chrome: &EditorChromeSnapshot,
) -> WorkbenchCommandPaletteOpenState {
    let registry = EditorCommandRegistry::default_workbench();
    let context = command_context_from_chrome(chrome);
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
        disabled_commands: UiValue::Array(
            entries
                .iter()
                .filter(|entry| entry.disabled)
                .map(|entry| UiValue::String(entry.id.clone()))
                .collect(),
        ),
        selected_command_id,
        focused_index,
    }
}

fn command_context_from_chrome(chrome: &EditorChromeSnapshot) -> EditorCommandContext {
    EditorCommandContext {
        project_open: chrome.project_open,
        can_undo: chrome.can_undo,
        can_redo: chrome.can_redo,
        selection_present: chrome.inspector.is_some(),
        play_mode_active: chrome.session_mode == EditorSessionMode::Playing,
    }
}

fn focused_command_index(entries: &[EditorCommandPaletteEntry]) -> i64 {
    entries
        .iter()
        .enumerate()
        .find(|(_, entry)| !entry.disabled && entry.id != COMMAND_PALETTE_COMMAND_ID)
        .or_else(|| {
            entries
                .iter()
                .enumerate()
                .find(|(_, entry)| !entry.disabled)
        })
        .map(|(index, _)| index as i64)
        .unwrap_or(-1)
}
