use crate::core::editor_extension::EditorExtensionRegistry;
use crate::ui::host::{EditorCommandContext, EditorCommandRegistry};
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use crate::ui::workbench::startup::EditorSessionMode;

use super::super::menu_bar_model::MenuBarModel;
use super::extension_menu::append_extension_menus;

pub(crate) fn default_menu_bar_with_extensions(
    chrome: &EditorChromeSnapshot,
    extensions: &[EditorExtensionRegistry],
    enabled_capabilities: &[String],
) -> MenuBarModel {
    let mut menu_bar = EditorCommandRegistry::default_workbench()
        .menu_bar_model(command_context_from_chrome(chrome));
    append_extension_menus(&mut menu_bar, extensions, enabled_capabilities);
    menu_bar
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
