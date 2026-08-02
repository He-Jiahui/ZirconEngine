use crate::core::commands::{
    CommandEvalCtx, EditorCommandRegistry, MenuBarModel, MenuItemModel, MenuModel,
};
use crate::core::editor_extension::EditorMenuItemDescriptor;
use crate::core::extension::{CapabilitySet, ContributionSnapshot};

pub(super) fn append_extension_menus(
    menu_bar: &mut MenuBarModel,
    command_registry: &EditorCommandRegistry,
    contributions: &ContributionSnapshot,
    capabilities: &CapabilitySet,
    context: &CommandEvalCtx,
) {
    let mut menu_items = contributions.menu_items(capabilities).collect::<Vec<_>>();
    menu_items.sort_by(|left, right| {
        left.priority()
            .cmp(&right.priority())
            .then_with(|| left.path().cmp(right.path()))
    });

    for descriptor in menu_items {
        append_extension_menu_item(menu_bar, command_registry, descriptor, context);
    }

    for view in contributions.views(capabilities) {
        let Ok(operation_path) = view.open_operation_path() else {
            continue;
        };
        if menu_bar.menus.iter().any(|menu| {
            menu.items
                .iter()
                .any(|item| item_contains_operation(item, &operation_path))
        }) {
            continue;
        }
        let enabled = command_registry
            .command(operation_path.as_str())
            .is_some_and(|descriptor| descriptor.is_enabled(context));
        let item = MenuItemModel::leaf(
            view.display_name(),
            None,
            Some(operation_path),
            None,
            enabled,
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

fn append_extension_menu_item(
    menu_bar: &mut MenuBarModel,
    command_registry: &EditorCommandRegistry,
    descriptor: &EditorMenuItemDescriptor,
    context: &CommandEvalCtx,
) {
    let segments = descriptor
        .path()
        .split('/')
        .filter(|segment| !segment.trim().is_empty())
        .map(str::trim)
        .collect::<Vec<_>>();
    let Some(menu_label) = segments.first().copied() else {
        return;
    };
    let Some(item_label) = segments.last().copied() else {
        return;
    };

    let item = MenuItemModel::leaf(
        item_label,
        None,
        Some(descriptor.operation().clone()),
        descriptor.shortcut().map(str::to_string),
        descriptor.enabled()
            && command_registry
                .command(descriptor.operation().as_str())
                .is_some_and(|command| command.is_enabled(context)),
    );
    let branch_path = &segments[1..segments.len().saturating_sub(1)];

    let menu = if let Some(index) = menu_bar
        .menus
        .iter()
        .position(|menu| menu.label.eq_ignore_ascii_case(menu_label))
    {
        &mut menu_bar.menus[index]
    } else {
        menu_bar.menus.push(MenuModel {
            label: menu_label.to_string(),
            items: Vec::new(),
        });
        let index = menu_bar.menus.len() - 1;
        &mut menu_bar.menus[index]
    };
    insert_menu_item(&mut menu.items, branch_path, item);
}

fn insert_menu_item(items: &mut Vec<MenuItemModel>, branch_path: &[&str], item: MenuItemModel) {
    let Some((branch_label, remaining_path)) = branch_path.split_first() else {
        items.push(item);
        return;
    };
    if let Some(branch) = items.iter_mut().find(|candidate| {
        candidate.label.eq_ignore_ascii_case(branch_label) && candidate.has_children()
    }) {
        insert_menu_item(&mut branch.children, remaining_path, item);
        branch.enabled = branch.children.iter().any(|child| child.enabled);
        return;
    }

    let mut branch = MenuItemModel::branch(*branch_label, Vec::new());
    insert_menu_item(&mut branch.children, remaining_path, item);
    branch.enabled = branch.children.iter().any(|child| child.enabled);
    items.push(branch);
}

fn item_contains_operation(
    item: &MenuItemModel,
    operation_path: &crate::core::editor_operation::EditorOperationPath,
) -> bool {
    item.operation_path.as_ref() == Some(operation_path)
        || item
            .children
            .iter()
            .any(|child| item_contains_operation(child, operation_path))
}
