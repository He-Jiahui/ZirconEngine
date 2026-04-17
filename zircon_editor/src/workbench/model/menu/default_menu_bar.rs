use crate::snapshot::EditorChromeSnapshot;

use super::super::menu_bar_model::MenuBarModel;
use super::edit_menu::build_edit_menu;
use super::file_menu::build_file_menu;
use super::help_menu::build_help_menu;
use super::selection_menu::build_selection_menu;
use super::view_menu::build_view_menu;
use super::window_menu::build_window_menu;

pub(crate) fn default_menu_bar(chrome: &EditorChromeSnapshot) -> MenuBarModel {
    MenuBarModel {
        menus: vec![
            build_file_menu(chrome),
            build_edit_menu(chrome),
            build_selection_menu(chrome),
            build_view_menu(),
            build_window_menu(),
            build_help_menu(),
        ],
    }
}
