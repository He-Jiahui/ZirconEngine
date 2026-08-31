use std::collections::HashSet;

use crate::core::commands::{
    CommandEvalCtx, EditorCommandRegistry, EditorKeymap, MenuBarModel, MenuItemModel, MenuModel,
};
use crate::core::editor_extension::EditorMenuItemDescriptor;
use crate::core::editor_operation::EditorOperationPath;
use crate::core::extension::{CapabilitySet, ContributionSnapshot, DocumentToolkitDescriptor};
use crate::core::i18n::{EditorI18nService, EditorLocale};

pub(super) fn append_contributed_menus(
    menu_bar: &mut MenuBarModel,
    command_registry: &EditorCommandRegistry,
    keymap: &EditorKeymap,
    i18n: &EditorI18nService,
    locale: &EditorLocale,
    contributions: &ContributionSnapshot,
    capabilities: &CapabilitySet,
    focused_toolkit: Option<&DocumentToolkitDescriptor>,
    context: &CommandEvalCtx,
) {
    let mut menu_items = contributions.menu_items(capabilities).collect::<Vec<_>>();
    if let Some(toolkit) = focused_toolkit {
        menu_items.extend(toolkit.menu_items().iter().filter(|descriptor| {
            descriptor
                .required_capabilities()
                .iter()
                .all(|capability| capabilities.contains(capability))
        }));
    }
    menu_items.sort_by(|left, right| {
        left.priority()
            .cmp(&right.priority())
            .then_with(|| left.path().cmp(right.path()))
            .then_with(|| left.operation().cmp(right.operation()))
    });

    let mut operation_paths = menu_operation_paths(menu_bar);
    for descriptor in menu_items {
        if command_registry
            .command(descriptor.operation().as_str())
            .is_none()
        {
            continue;
        }
        if !operation_paths.insert(descriptor.operation().clone()) {
            continue;
        }
        append_contributed_menu_item(
            menu_bar,
            command_registry,
            keymap,
            i18n,
            locale,
            descriptor,
            context,
        );
    }

    let mut views = contributions.views(capabilities).collect::<Vec<_>>();
    views.sort_by(|left, right| {
        left.display_name()
            .cmp(right.display_name())
            .then_with(|| left.id().cmp(right.id()))
    });
    for view in views {
        let Ok(operation_path) = view.open_operation_path() else {
            continue;
        };
        let Some(command) = command_registry.command(operation_path.as_str()) else {
            continue;
        };
        if !operation_paths.insert(operation_path.clone()) {
            continue;
        }
        let item = MenuItemModel::leaf(
            view.display_name(),
            None,
            Some(operation_path.clone()),
            keymap
                .chord_for_command(operation_path.as_str())
                .map(ToString::to_string),
            command.is_enabled(context),
        );
        if let Some(menu) = menu_bar
            .menus
            .iter_mut()
            .find(|menu| menu.label.eq_ignore_ascii_case("View"))
        {
            menu.items.push(item);
        } else {
            menu_bar.menus.push(MenuModel {
                label: "View".to_string(),
                items: vec![item],
            });
        }
    }
}

fn append_contributed_menu_item(
    menu_bar: &mut MenuBarModel,
    command_registry: &EditorCommandRegistry,
    keymap: &EditorKeymap,
    i18n: &EditorI18nService,
    locale: &EditorLocale,
    descriptor: &EditorMenuItemDescriptor,
    context: &CommandEvalCtx,
) {
    let Some(command) = command_registry.command(descriptor.operation().as_str()) else {
        return;
    };
    let menu_path = descriptor.menu_path();
    let menu_label = command
        .presentation()
        .resolve_key(i18n, locale, menu_path.root().label_key());
    let item_label = command
        .presentation()
        .resolve_key(i18n, locale, menu_path.leaf().label_key());

    let item = MenuItemModel::leaf(
        item_label.as_ref(),
        None,
        Some(descriptor.operation().clone()),
        keymap
            .chord_for_command(descriptor.operation().as_str())
            .map(ToString::to_string),
        descriptor.enabled()
            && command_registry
                .command(descriptor.operation().as_str())
                .is_some_and(|command| command.is_enabled(context)),
    );
    let branch_path = menu_path
        .groups()
        .iter()
        .map(|group| {
            command
                .presentation()
                .resolve_key(i18n, locale, group.label_key())
        })
        .collect::<Vec<_>>();

    let menu = if let Some(index) = menu_bar
        .menus
        .iter()
        .position(|menu| menu.label.eq_ignore_ascii_case(menu_label.as_ref()))
    {
        &mut menu_bar.menus[index]
    } else {
        menu_bar.menus.push(MenuModel {
            label: menu_label.as_ref().to_owned(),
            items: Vec::new(),
        });
        let index = menu_bar.menus.len() - 1;
        &mut menu_bar.menus[index]
    };
    insert_menu_item(&mut menu.items, &branch_path, item);
}

fn insert_menu_item(
    items: &mut Vec<MenuItemModel>,
    branch_path: &[std::sync::Arc<str>],
    item: MenuItemModel,
) {
    let Some((branch_label, remaining_path)) = branch_path.split_first() else {
        items.push(item);
        return;
    };
    if let Some(branch) = items.iter_mut().find(|candidate| {
        candidate.label.eq_ignore_ascii_case(branch_label.as_ref()) && candidate.has_children()
    }) {
        insert_menu_item(&mut branch.children, remaining_path, item);
        branch.enabled = branch.children.iter().any(|child| child.enabled);
        return;
    }

    let mut branch = MenuItemModel::branch(branch_label.as_ref(), Vec::new());
    insert_menu_item(&mut branch.children, remaining_path, item);
    branch.enabled = branch.children.iter().any(|child| child.enabled);
    items.push(branch);
}

fn menu_operation_paths(menu_bar: &MenuBarModel) -> HashSet<EditorOperationPath> {
    let mut operation_paths = HashSet::new();
    for menu in &menu_bar.menus {
        collect_menu_operation_paths(&menu.items, &mut operation_paths);
    }
    operation_paths
}

fn collect_menu_operation_paths(
    items: &[MenuItemModel],
    operation_paths: &mut HashSet<EditorOperationPath>,
) {
    for item in items {
        if let Some(operation_path) = &item.operation_path {
            let _ = operation_paths.insert(operation_path.clone());
        }
        collect_menu_operation_paths(&item.children, operation_paths);
    }
}

#[cfg(test)]
mod operation_index_tests;
