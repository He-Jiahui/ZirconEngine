use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
use crate::ui::workbench::event::editor_operation_binding;

use super::super::menu_item_model::MenuItemModel;
use super::super::menu_model::MenuModel;
use super::super::MenuBarModel;

pub(super) fn append_extension_menus(
    menu_bar: &mut MenuBarModel,
    extensions: &[EditorExtensionRegistry],
    enabled_capabilities: &[String],
) {
    let mut contributions = extensions
        .iter()
        .flat_map(EditorExtensionRegistry::menu_items)
        .filter(|descriptor| descriptor_enabled(descriptor, enabled_capabilities))
        .collect::<Vec<_>>();
    contributions.sort_by(|left, right| {
        left.priority()
            .cmp(&right.priority())
            .then_with(|| left.path().cmp(right.path()))
    });

    for descriptor in contributions {
        append_extension_menu_item(menu_bar, descriptor);
    }

    for extension in extensions {
        for view in extension.views() {
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
            let item = MenuItemModel::leaf(
                view.display_name(),
                None,
                editor_operation_binding(&operation_path),
                Some(operation_path),
                None,
                true,
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
}

fn descriptor_enabled(
    descriptor: &EditorMenuItemDescriptor,
    enabled_capabilities: &[String],
) -> bool {
    descriptor.required_capabilities().iter().all(|required| {
        enabled_capabilities
            .iter()
            .any(|enabled| enabled == required)
    })
}

fn append_extension_menu_item(menu_bar: &mut MenuBarModel, descriptor: &EditorMenuItemDescriptor) {
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
        editor_operation_binding(descriptor.operation()),
        Some(descriptor.operation().clone()),
        descriptor.shortcut().map(str::to_string),
        descriptor.enabled(),
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
        menu_bar
            .menus
            .last_mut()
            .expect("just pushed extension menu")
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
