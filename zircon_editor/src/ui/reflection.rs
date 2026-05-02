use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use zircon_runtime_interface::ui::{
    event_ui::UiActionDescriptor, event_ui::UiNodeDescriptor, event_ui::UiNodeId,
    event_ui::UiNodePath, event_ui::UiPropertyDescriptor, event_ui::UiReflectionSnapshot,
    event_ui::UiRouteId, event_ui::UiStateFlags, event_ui::UiTreeId, event_ui::UiValueType,
};

use crate::ui::binding::EditorUiBinding;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorActivityKind {
    ActivityView,
    ActivityWindow,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorActivityHost {
    Drawer(String),
    DocumentPage(String),
    FloatingWindow(String),
    ExclusivePage(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorActivityReflection {
    pub instance_id: String,
    pub descriptor_id: String,
    pub title: String,
    pub kind: EditorActivityKind,
    pub host: EditorActivityHost,
    pub visible: bool,
    pub enabled: bool,
    pub dirty: bool,
    pub properties: BTreeMap<String, Value>,
    pub actions: Vec<UiActionDescriptor>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorMenuItemReflectionModel {
    pub menu_id: String,
    pub control_id: String,
    pub label: String,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
    pub binding: EditorUiBinding,
    pub route_id: Option<UiRouteId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorHostPageReflectionModel {
    pub page_id: String,
    pub title: String,
    pub active: bool,
    pub exclusive: bool,
    pub activities: Vec<EditorActivityReflection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorFloatingWindowReflectionModel {
    pub window_id: String,
    pub title: String,
    pub activities: Vec<EditorActivityReflection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorDrawerReflectionModel {
    pub drawer_id: String,
    pub title: String,
    pub visible: bool,
    pub activities: Vec<EditorActivityReflection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorWorkbenchReflectionModel {
    pub tree_id: UiTreeId,
    pub status_line: String,
    pub menu_items: Vec<EditorMenuItemReflectionModel>,
    pub pages: Vec<EditorHostPageReflectionModel>,
    pub drawers: Vec<EditorDrawerReflectionModel>,
    pub floating_windows: Vec<EditorFloatingWindowReflectionModel>,
}

impl EditorWorkbenchReflectionModel {
    pub fn new(tree_id: UiTreeId) -> Self {
        Self {
            tree_id,
            status_line: String::new(),
            menu_items: Vec::new(),
            pages: Vec::new(),
            drawers: Vec::new(),
            floating_windows: Vec::new(),
        }
    }
}

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
                    Some(route_id) => {
                        UiActionDescriptor::new("onClick", action.path.event_kind, symbol)
                            .with_callable_from_remote(true)
                            .with_route_id(route_id)
                    }
                    None => UiActionDescriptor::new("onClick", action.path.event_kind, symbol),
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

fn activity_node(
    builder: &mut SnapshotBuilder,
    activity: &EditorActivityReflection,
    node_path: String,
) -> UiNodeId {
    let mut properties = vec![
        UiPropertyDescriptor::new(
            "descriptor_id",
            UiValueType::String,
            json!(activity.descriptor_id),
        ),
        UiPropertyDescriptor::new(
            "host",
            UiValueType::String,
            json!(host_name(&activity.host)),
        ),
        UiPropertyDescriptor::new(
            "kind",
            UiValueType::String,
            json!(kind_name(&activity.kind)),
        ),
    ];
    properties.extend(activity.properties.iter().map(|(name, value)| {
        UiPropertyDescriptor::new(name.clone(), infer_value_type(value), value.clone())
    }));

    builder.push_node(
        node_path,
        match activity.kind {
            EditorActivityKind::ActivityView => "ActivityView",
            EditorActivityKind::ActivityWindow => "ActivityWindow",
        },
        activity.title.clone(),
        UiStateFlags {
            visible: activity.visible,
            enabled: activity.enabled,
            clickable: false,
            hoverable: false,
            focusable: true,
            pressed: false,
            checked: false,
            dirty: activity.dirty,
        },
        properties,
        activity.actions.clone(),
    )
}

fn host_name(host: &EditorActivityHost) -> &'static str {
    match host {
        EditorActivityHost::Drawer(_) => "drawer",
        EditorActivityHost::DocumentPage(_) => "document_page",
        EditorActivityHost::FloatingWindow(_) => "floating_window",
        EditorActivityHost::ExclusivePage(_) => "exclusive_page",
    }
}

fn kind_name(kind: &EditorActivityKind) -> &'static str {
    match kind {
        EditorActivityKind::ActivityView => "activity_view",
        EditorActivityKind::ActivityWindow => "activity_window",
    }
}

fn infer_value_type(value: &Value) -> UiValueType {
    match value {
        Value::Null => UiValueType::Null,
        Value::Bool(_) => UiValueType::Bool,
        Value::Number(number) if number.is_u64() => UiValueType::Unsigned,
        Value::Number(number) if number.is_i64() => UiValueType::Signed,
        Value::Number(_) => UiValueType::Float,
        Value::String(_) => UiValueType::String,
        Value::Array(_) => UiValueType::Array,
        Value::Object(_) => UiValueType::Object,
    }
}

fn visible_enabled_flags(visible: bool, enabled: bool) -> UiStateFlags {
    UiStateFlags {
        visible,
        enabled,
        clickable: false,
        hoverable: false,
        focusable: false,
        pressed: false,
        checked: false,
        dirty: false,
    }
}

struct SnapshotBuilder {
    tree_id: UiTreeId,
    next_id: u64,
    nodes: BTreeMap<UiNodeId, UiNodeDescriptor>,
}

impl SnapshotBuilder {
    fn new(tree_id: UiTreeId) -> Self {
        Self {
            tree_id,
            next_id: 0,
            nodes: BTreeMap::new(),
        }
    }

    fn push_node(
        &mut self,
        path: impl Into<String>,
        class_name: impl Into<String>,
        display_name: impl Into<String>,
        state_flags: UiStateFlags,
        properties: Vec<UiPropertyDescriptor>,
        actions: Vec<UiActionDescriptor>,
    ) -> UiNodeId {
        self.next_id += 1;
        let node_id = UiNodeId::new(self.next_id);
        let mut node =
            UiNodeDescriptor::new(node_id, UiNodePath::new(path), class_name, display_name)
                .with_state_flags(state_flags);
        for property in properties {
            node = node.with_property(property);
        }
        for action in actions {
            node = node.with_action(action);
        }
        self.nodes.insert(node_id, node);
        node_id
    }

    fn add_child(&mut self, parent: UiNodeId, child: UiNodeId) {
        if let Some(node) = self.nodes.get_mut(&parent) {
            node.children.push(child);
        }
    }

    fn finish(self, root: UiNodeId) -> UiReflectionSnapshot {
        UiReflectionSnapshot {
            tree_id: self.tree_id,
            roots: vec![root],
            nodes: self.nodes,
        }
    }
}
