use serde_json::json;

use crate::{
    ActivityDrawerSlotPreference, ActivityViewDescriptor, ActivityWindowDescriptor,
    EditorActivityHost, EditorActivityKind, EditorActivityReflection, EditorDrawerReflectionModel,
    EditorHostPageReflectionModel, EditorMenuItemReflectionModel, EditorUiBinding,
    EditorUiBindingPayload, EditorUiControlService, EditorUiEventKind, EditorUiReflectionAdapter,
    EditorWorkbenchReflectionModel,
};
use zircon_runtime::ui::{
    binding::UiBindingCall, binding::UiBindingValue, binding::UiEventBinding, binding::UiEventKind,
    binding::UiEventPath, event_ui::UiControlRequest, event_ui::UiControlResponse,
    event_ui::UiNodePath, event_ui::UiTreeId,
};

#[test]
fn editor_ui_control_service_registers_activity_descriptors() {
    let mut service = EditorUiControlService::default();
    service
        .register_activity_view(
            ActivityViewDescriptor::new("editor.hierarchy", "Hierarchy", "hierarchy")
                .with_multi_instance(false)
                .with_default_drawer(ActivityDrawerSlotPreference::LeftTop)
                .with_reflection_root(UiNodePath::new("editor/views/hierarchy")),
        )
        .unwrap();
    service
        .register_activity_window(
            ActivityWindowDescriptor::new("editor.prefab", "Prefab Editor", "prefab")
                .with_multi_instance(true)
                .with_supports_exclusive_page(true)
                .with_supports_floating_window(true)
                .with_reflection_root(UiNodePath::new("editor/windows/prefab")),
        )
        .unwrap();

    assert!(service.activity_view("editor.hierarchy").is_some());
    assert!(service.activity_window("editor.prefab").is_some());
}

#[test]
fn editor_ui_reflection_adapter_projects_activity_hosts_and_menu_bindings() {
    let mut service = EditorUiControlService::default();
    let menu_binding = EditorUiBinding::new(
        "WorkbenchMenuBar",
        "SaveProject",
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action("SaveProject"),
    );
    let menu_route = service.register_route(menu_binding.as_ui_binding(), |_context| {
        Ok(json!({ "saved": true }))
    });

    let snapshot = EditorUiReflectionAdapter::build_snapshot(&EditorWorkbenchReflectionModel {
        tree_id: UiTreeId::new("editor.workbench"),
        status_line: "Ready".to_string(),
        menu_items: vec![EditorMenuItemReflectionModel {
            menu_id: "file".to_string(),
            control_id: "SaveProject".to_string(),
            label: "Save Project".to_string(),
            enabled: true,
            binding: menu_binding,
            route_id: Some(menu_route),
        }],
        pages: vec![EditorHostPageReflectionModel {
            page_id: "page:prefab".to_string(),
            title: "Prefab Editor".to_string(),
            active: true,
            exclusive: true,
            activities: vec![EditorActivityReflection {
                instance_id: "editor.prefab#1".to_string(),
                descriptor_id: "editor.prefab".to_string(),
                title: "Prefab Editor".to_string(),
                kind: EditorActivityKind::ActivityWindow,
                host: EditorActivityHost::ExclusivePage("page:prefab".to_string()),
                visible: true,
                enabled: true,
                dirty: true,
                properties: vec![("asset_path".to_string(), json!("crate://player.prefab"))]
                    .into_iter()
                    .collect(),
                actions: Vec::new(),
            }],
        }],
        drawers: vec![EditorDrawerReflectionModel {
            drawer_id: "left_top".to_string(),
            title: "Hierarchy".to_string(),
            visible: true,
            activities: vec![EditorActivityReflection {
                instance_id: "editor.hierarchy#1".to_string(),
                descriptor_id: "editor.hierarchy".to_string(),
                title: "Hierarchy".to_string(),
                kind: EditorActivityKind::ActivityView,
                host: EditorActivityHost::Drawer("left_top".to_string()),
                visible: true,
                enabled: true,
                dirty: false,
                properties: vec![("selected_count".to_string(), json!(1))]
                    .into_iter()
                    .collect(),
                actions: Vec::new(),
            }],
        }],
        floating_windows: Vec::new(),
    });

    service.publish_snapshot(snapshot);

    let node = service.handle_request(UiControlRequest::QueryNode {
        node_path: UiNodePath::new("editor/workbench/pages/page:prefab/editor.prefab#1"),
    });
    assert!(matches!(
        node,
        UiControlResponse::Node(Some(node))
            if node.properties["host"].reflected_value == json!("exclusive_page")
    ));
    let menu = service.handle_request(UiControlRequest::QueryNode {
        node_path: UiNodePath::new("editor/workbench/menu/file/SaveProject"),
    });
    assert!(matches!(
        menu,
        UiControlResponse::Node(Some(node))
            if node.actions.contains_key("onClick")
    ));
    let invoked = service.handle_request(UiControlRequest::InvokeBinding {
        binding: UiEventBinding::new(
            UiEventPath::new("WorkbenchMenuBar", "SaveProject", UiEventKind::Click),
            UiBindingCall::new("MenuAction").with_argument(UiBindingValue::string("SaveProject")),
        ),
    });
    assert!(matches!(
        invoked,
        UiControlResponse::Invocation(result)
            if result.value == Some(json!({ "saved": true }))
    ));
}
