use crate::core::commands::{
    CommandEvalCtx, EditorCommandRegistry, EditorKeymap, MenuBarModel, MenuItemModel,
};
use crate::core::extension::{CapabilitySet, ContributionSnapshot, DocumentToolkitDescriptor};
use crate::core::i18n::{EditorI18nService, EditorLocale};

use super::extension_menu::append_contributed_menus;
pub(crate) fn default_menu_bar_with_sources(
    command_registry: &EditorCommandRegistry,
    keymap: &EditorKeymap,
    i18n: &EditorI18nService,
    locale: &EditorLocale,
    contributions: &ContributionSnapshot,
    capabilities: &CapabilitySet,
    focused_toolkit: Option<&DocumentToolkitDescriptor>,
    context: &CommandEvalCtx,
) -> MenuBarModel {
    let mut menu_bar = command_registry.menu_bar_model(i18n, locale, context);
    apply_effective_shortcuts(&mut menu_bar, keymap);
    append_contributed_menus(
        &mut menu_bar,
        command_registry,
        keymap,
        i18n,
        locale,
        contributions,
        capabilities,
        focused_toolkit,
        context,
    );
    menu_bar
}

fn apply_effective_shortcuts(menu_bar: &mut MenuBarModel, keymap: &EditorKeymap) {
    for menu in &mut menu_bar.menus {
        for item in &mut menu.items {
            apply_effective_item_shortcut(item, keymap);
        }
    }
}

fn apply_effective_item_shortcut(item: &mut MenuItemModel, keymap: &EditorKeymap) {
    if let Some(operation) = item.operation_path.as_ref() {
        item.shortcut = keymap
            .chord_for_command(operation.as_str())
            .map(ToString::to_string);
    }
    for child in &mut item.children {
        apply_effective_item_shortcut(child, keymap);
    }
}

#[cfg(test)]
mod tests {
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
}
