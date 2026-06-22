use serde_json::json;
use zircon_runtime_interface::ui::event_ui::{
    UiActionDescriptor, UiPropertyDescriptor, UiReflectionSnapshot, UiStateFlags, UiValueType,
};

use super::activity::activity_node;
use super::builder::SnapshotBuilder;
use super::model::EditorWorkbenchReflectionModel;
use super::state_flags::visible_enabled_flags;

#[derive(Default)]
pub struct EditorUiReflectionAdapter;

impl EditorUiReflectionAdapter {
    pub fn build_snapshot(model: &EditorWorkbenchReflectionModel) -> UiReflectionSnapshot {
        let mut builder = SnapshotBuilder::new(model.tree_id.clone());
        let root = builder.push_node(
            "editor/workbench",
            "EditorWorkbench",
            "Workbench",
            UiStateFlags {
                visible: true,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                pressed: false,
                checked: false,
                dirty: false,
            },
            vec![UiPropertyDescriptor::new(
                "status_line",
                UiValueType::String,
                json!(model.status_line),
            )],
            Vec::new(),
        );
        let menu_root = builder.push_node(
            "editor/workbench/menu",
            "MenuBar",
            "Menu",
            visible_enabled_flags(true, true),
            Vec::new(),
            Vec::new(),
        );
        builder.add_child(root, menu_root);
        for item in &model.menu_items {
            let action = item.binding.as_ui_binding();
            let symbol = action
                .action
                .as_ref()
                .map(|call| call.symbol.clone())
                .unwrap_or_else(|| "Action".to_string());
            let mut properties = vec![
                UiPropertyDescriptor::new("menu_id", UiValueType::String, json!(item.menu_id)),
                UiPropertyDescriptor::new(
                    "native_binding",
                    UiValueType::String,
                    json!(item.binding.native_binding()),
                ),
            ];
            if let Some(operation_path) = &item.operation_path {
                properties.push(UiPropertyDescriptor::new(
                    "operation_path",
                    UiValueType::String,
                    json!(operation_path),
                ));
            }
            if let Some(shortcut) = &item.shortcut {
                properties.push(UiPropertyDescriptor::new(
                    "shortcut",
                    UiValueType::String,
                    json!(shortcut),
                ));
            }
            let node = builder.push_node(
                format!("editor/workbench/menu/{}/{}", item.menu_id, item.control_id),
                "MenuItem",
                item.label.clone(),
                visible_enabled_flags(true, item.enabled),
                properties,
                vec![match item.route_id {
                    Some(route_id) => UiActionDescriptor::new(
                        "workbench.menu.item.click",
                        action.path.event_kind,
                        symbol,
                    )
                    .with_callable_from_remote(true)
                    .with_route_id(route_id),
                    None => UiActionDescriptor::new(
                        "workbench.menu.item.click",
                        action.path.event_kind,
                        symbol,
                    ),
                }],
            );
            builder.add_child(menu_root, node);
        }

        let pages_root = builder.push_node(
            "editor/workbench/pages",
            "PageCollection",
            "Pages",
            visible_enabled_flags(true, true),
            Vec::new(),
            Vec::new(),
        );
        builder.add_child(root, pages_root);
        for page in &model.pages {
            let page_node = builder.push_node(
                format!("editor/workbench/pages/{}", page.page_id),
                if page.exclusive {
                    "ExclusivePage"
                } else {
                    "WorkbenchPage"
                },
                page.title.clone(),
                UiStateFlags {
                    visible: true,
                    enabled: true,
                    clickable: false,
                    hoverable: false,
                    focusable: true,
                    pressed: false,
                    checked: page.active,
                    dirty: false,
                },
                vec![UiPropertyDescriptor::new(
                    "exclusive",
                    UiValueType::Bool,
                    json!(page.exclusive),
                )],
                Vec::new(),
            );
            builder.add_child(pages_root, page_node);
            for activity in &page.activities {
                let node = activity_node(
                    &mut builder,
                    activity,
                    format!(
                        "editor/workbench/pages/{}/{}",
                        page.page_id, activity.instance_id
                    ),
                );
                builder.add_child(page_node, node);
            }
        }

        let drawers_root = builder.push_node(
            "editor/workbench/drawers",
            "DrawerCollection",
            "Drawers",
            visible_enabled_flags(true, true),
            Vec::new(),
            Vec::new(),
        );
        builder.add_child(root, drawers_root);
        for drawer in &model.drawers {
            let drawer_node = builder.push_node(
                format!("editor/workbench/drawers/{}", drawer.drawer_id),
                "ActivityDrawer",
                drawer.title.clone(),
                visible_enabled_flags(drawer.visible, true),
                Vec::new(),
                Vec::new(),
            );
            builder.add_child(drawers_root, drawer_node);
            for activity in &drawer.activities {
                let node = activity_node(
                    &mut builder,
                    activity,
                    format!(
                        "editor/workbench/drawers/{}/{}",
                        drawer.drawer_id, activity.instance_id
                    ),
                );
                builder.add_child(drawer_node, node);
            }
        }

        let floating_root = builder.push_node(
            "editor/workbench/floating",
            "FloatingWindows",
            "Floating Windows",
            visible_enabled_flags(true, true),
            Vec::new(),
            Vec::new(),
        );
        builder.add_child(root, floating_root);
        for window in &model.floating_windows {
            let window_node = builder.push_node(
                format!("editor/workbench/floating/{}", window.window_id),
                "FloatingWindow",
                window.title.clone(),
                visible_enabled_flags(true, true),
                Vec::new(),
                Vec::new(),
            );
            builder.add_child(floating_root, window_node);
            for activity in &window.activities {
                let node = activity_node(
                    &mut builder,
                    activity,
                    format!(
                        "editor/workbench/floating/{}/{}",
                        window.window_id, activity.instance_id
                    ),
                );
                builder.add_child(window_node, node);
            }
        }

        builder.finish(root)
    }
}
