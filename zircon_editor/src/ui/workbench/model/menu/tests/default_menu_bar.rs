use std::collections::BTreeMap;

use super::*;
use crate::core::commands::EditorKeyChord;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::settings::EditorKeymapOverrides;

#[test]
fn menu_bar_shortcuts_follow_effective_overrides_and_unbindings() {
    let registry = EditorCommandRegistry::default_workbench();
    let keymap = EditorKeymap::default_workbench().with_overrides(&EditorKeymapOverrides::new(
        BTreeMap::from([
            (
                EditorOperationPath::parse("file.project.open").unwrap(),
                Some("Alt+O".parse::<EditorKeyChord>().unwrap()),
            ),
            (
                EditorOperationPath::parse("file.project.save").unwrap(),
                None,
            ),
        ]),
    ));
    let menu_bar = default_menu_bar_with_sources(
        &registry,
        &keymap,
        &EditorI18nService::default(),
        &EditorLocale::english(),
        &ContributionSnapshot::default(),
        &CapabilitySet::default(),
        None,
        &CommandEvalCtx::interactive().with_project_open(true),
    );

    assert_eq!(
        item(&menu_bar, "file.project.open").shortcut.as_deref(),
        Some("Alt+O")
    );
    assert_eq!(item(&menu_bar, "file.project.save").shortcut, None);
}

fn item<'a>(menu_bar: &'a MenuBarModel, operation: &str) -> &'a MenuItemModel {
    menu_bar
        .menus
        .iter()
        .flat_map(|menu| &menu.items)
        .find_map(|item| find_item(item, operation))
        .unwrap_or_else(|| panic!("menu item {operation} should exist"))
}

fn find_item<'a>(item: &'a MenuItemModel, operation: &str) -> Option<&'a MenuItemModel> {
    if item.operation_path.as_ref().map(|path| path.as_str()) == Some(operation) {
        return Some(item);
    }
    item.children
        .iter()
        .find_map(|child| find_item(child, operation))
}
