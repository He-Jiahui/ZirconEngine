use crate::core::editor_extension::{EditorExtensionRegistry, EditorMenuItemDescriptor};
use crate::ui::workbench::event::editor_operation_binding;

use super::super::menu_item_model::MenuItemModel;
use super::super::menu_model::MenuModel;
use super::super::MenuBarModel;

pub(super) fn append_extension_menus(
    menu_bar: &mut MenuBarModel,
    extensions: &[EditorExtensionRegistry],
) {
    let mut contributions = extensions
        .iter()
        .flat_map(EditorExtensionRegistry::menu_items)
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
            if menu_bar
                .menus
                .iter()
                .flat_map(|menu| menu.items.iter())
                .any(|item| item.operation_path.as_ref() == Some(&operation_path))
            {
                continue;
            }
            let item = MenuItemModel {
                label: view.display_name().to_string(),
                action: None,
                binding: editor_operation_binding(&operation_path),
                operation_path: Some(operation_path),
                shortcut: None,
                enabled: true,
            };
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

fn append_extension_menu_item(menu_bar: &mut MenuBarModel, descriptor: &EditorMenuItemDescriptor) {
    let mut segments = descriptor
        .path()
        .split('/')
        .filter(|segment| !segment.trim().is_empty())
        .collect::<Vec<_>>();
    let Some(menu_label) = segments.first().copied() else {
        return;
    };
    let Some(item_label) = segments.pop() else {
        return;
    };

    let item = MenuItemModel {
        label: item_label.to_string(),
        action: None,
        binding: editor_operation_binding(descriptor.operation()),
        operation_path: Some(descriptor.operation().clone()),
        shortcut: descriptor.shortcut().map(str::to_string),
        enabled: true,
    };

    if let Some(menu) = menu_bar
        .menus
        .iter_mut()
        .find(|menu| menu.label.eq_ignore_ascii_case(menu_label))
    {
        menu.items.push(item);
        return;
    }

    menu_bar.menus.push(MenuModel {
        label: menu_label.to_string(),
        items: vec![item],
    });
}
