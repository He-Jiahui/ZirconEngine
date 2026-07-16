use crate::core::editor_event::{EditorEvent, EditorEventTransient};

use super::{
    CommandEvalCtx, EditorCommandDescriptor, EditorCommandMenuProjection, EditorCommandRegistry,
    MenuBarModel, MenuItemModel, MenuModel,
};

pub(super) fn menu_bar_model(
    registry: &EditorCommandRegistry,
    context: &CommandEvalCtx,
) -> MenuBarModel {
    const MENU_ORDER: [&str; 7] = [
        "File",
        "Edit",
        "Selection",
        "Play",
        "View",
        "Window",
        "Help",
    ];

    MenuBarModel {
        menus: MENU_ORDER
            .into_iter()
            .filter_map(|label| menu_model(registry, label, context))
            .collect(),
    }
}

pub(super) fn menu_model(
    registry: &EditorCommandRegistry,
    label: &str,
    context: &CommandEvalCtx,
) -> Option<MenuModel> {
    let items = registry
        .commands()
        .filter_map(|descriptor| command_menu_item(descriptor, label, context))
        .collect::<Vec<_>>();

    (!items.is_empty()).then(|| MenuModel {
        label: label.to_string(),
        items,
    })
}

fn command_menu_item(
    descriptor: &EditorCommandDescriptor,
    menu_label: &str,
    context: &CommandEvalCtx,
) -> Option<MenuItemModel> {
    if matches!(
        descriptor.event(),
        Some(EditorEvent::Transient(
            EditorEventTransient::OpenCommandPalette
        ))
    ) {
        return None;
    }
    if descriptor.menu_projection() != EditorCommandMenuProjection::CommandRegistry {
        return None;
    }
    let menu_path = descriptor.menu_path()?;
    let (top_level, item_label) = menu_path.split_once('/')?;
    if top_level != menu_label {
        return None;
    }

    let label = item_label
        .rsplit('/')
        .next()
        .filter(|label| !label.is_empty())
        .unwrap_or(descriptor.display_name());
    let shortcut = descriptor.default_chord().map(ToString::to_string);

    Some(MenuItemModel::leaf(
        label,
        None,
        Some(descriptor.id().clone()),
        shortcut,
        descriptor.is_enabled(context),
    ))
}
