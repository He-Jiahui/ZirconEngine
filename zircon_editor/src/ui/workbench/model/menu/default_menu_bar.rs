use crate::core::editor_extension::EditorExtensionRegistry;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::menu_bar_model::MenuBarModel;
use super::edit_menu::build_edit_menu;
use super::extension_menu::append_extension_menus;
use super::file_menu::build_file_menu;
use super::help_menu::build_help_menu;
use super::play_menu::build_play_menu;
use super::selection_menu::build_selection_menu;
use super::view_menu::build_view_menu;
use super::window_menu::build_window_menu;

pub(crate) fn default_menu_bar_with_extensions(
    chrome: &EditorChromeSnapshot,
    extensions: &[EditorExtensionRegistry],
    enabled_capabilities: &[String],
) -> MenuBarModel {
    let mut menu_bar = MenuBarModel {
        menus: vec![
            build_file_menu(chrome),
            build_edit_menu(chrome),
            build_selection_menu(chrome),
            build_play_menu(chrome),
            build_view_menu(),
            build_window_menu(),
            build_help_menu(),
        ],
    };
    append_extension_menus(&mut menu_bar, extensions, enabled_capabilities);
    menu_bar
}
