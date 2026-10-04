use crate::core::commands::{
    CommandEvalCtx, EditorCommandRegistry, EditorKeymap, MenuBarModel, MenuItemModel,
};
use crate::core::extension::{CapabilitySet, ContributionSnapshot, DocumentToolkitDescriptor};
use crate::core::i18n::{EditorI18nService, EditorLocale};

use super::extension_menu::append_contributed_menus;
/// 用本轮命令评估、有效keymap及能力集合组合三来源菜单；展示可用性不替代执行时验证。
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
#[path = "tests/default_menu_bar.rs"]
mod tests;
