use crate::core::commands::{CommandEvalCtx, EditorCommandRegistry};
use crate::core::extension::{CapabilitySet, ContributionSnapshot};

use super::extension_menu::append_extension_menus;
use crate::core::commands::MenuBarModel;

pub(crate) fn default_menu_bar_with_extensions(
    command_registry: &EditorCommandRegistry,
    contributions: &ContributionSnapshot,
    capabilities: &CapabilitySet,
    context: &CommandEvalCtx,
) -> MenuBarModel {
    let mut menu_bar = command_registry.menu_bar_model(context);
    append_extension_menus(
        &mut menu_bar,
        command_registry,
        contributions,
        capabilities,
        context,
    );
    menu_bar
}
