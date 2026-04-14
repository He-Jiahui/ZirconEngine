use crate::{
    activity_descriptors_from_views, build_workbench_reflection_model, default_preview_fixture,
    register_workbench_reflection_routes, WorkbenchViewModel,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use zircon_editor_ui::{EditorUiControlService, EditorUiReflectionAdapter};
use zircon_ui::{UiBindingValue, UiControlRequest, UiControlResponse, UiNodePath};

#[test]
fn workbench_reflection_model_projects_menu_and_activity_descriptors() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let view_model = WorkbenchViewModel::build(&chrome);
    let (views, windows) = activity_descriptors_from_views(&fixture.descriptors);

    assert!(views
        .iter()
        .any(|descriptor| descriptor.view_id == "editor.scene"));
    assert!(windows
        .iter()
        .any(|descriptor| descriptor.window_id == "editor.asset_browser"));

    let reflection = build_workbench_reflection_model(&chrome, &view_model);
    let snapshot = EditorUiReflectionAdapter::build_snapshot(&reflection);
    let mut service = EditorUiControlService::default();
    service.publish_snapshot(snapshot);

    let menu = service.handle_request(UiControlRequest::QueryNode {
        node_path: UiNodePath::new("editor/workbench/menu/file/SaveProject"),
    });
    assert!(matches!(
        menu,
        UiControlResponse::Node(Some(node))
            if node.display_name == "Save Project"
    ));
    let scene = service.handle_request(UiControlRequest::QueryNode {
        node_path: UiNodePath::new("editor/workbench/pages/workbench/editor.scene#1"),
    });
    assert!(matches!(
        scene,
        UiControlResponse::Node(Some(node))
            if node.properties["kind"].reflected_value == serde_json::json!("activity_view")
                && node.actions.contains_key("focus_view")
                && node.actions.contains_key("detach_to_window")
                && node.actions.contains_key("pointer_move")
                && node.actions.contains_key("scroll")
    ));
    let inspector = service.handle_request(UiControlRequest::QueryNode {
        node_path: UiNodePath::new("editor/workbench/drawers/right_top/editor.inspector#1"),
    });
    assert!(matches!(
        inspector,
        UiControlResponse::Node(Some(node))
            if node.actions.contains_key("apply_batch")
                && node.actions["apply_batch"].binding_symbol == "InspectorFieldBatch"
    ));
}

#[test]
fn workbench_reflection_routes_mark_activity_actions_as_remotely_callable() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let view_model = WorkbenchViewModel::build(&chrome);
    let mut service = EditorUiControlService::default();

    let reflection = register_workbench_reflection_routes(
        &mut service,
        build_workbench_reflection_model(&chrome, &view_model),
    );
    let snapshot = EditorUiReflectionAdapter::build_snapshot(&reflection);
    service.publish_snapshot(snapshot);

    let scene = service.handle_request(UiControlRequest::QueryNode {
        node_path: UiNodePath::new("editor/workbench/pages/workbench/editor.scene#1"),
    });
    assert!(matches!(
        scene,
        UiControlResponse::Node(Some(node))
            if node.actions["focus_view"].callable_from_remote
                && node.actions["focus_view"].route_id.is_some()
                && node.actions["detach_to_window"].callable_from_remote
                && node.actions["detach_to_window"].route_id.is_some()
                && node.actions["pointer_move"].callable_from_remote
                && node.actions["pointer_move"].route_id.is_some()
                && node.actions["left_press"].callable_from_remote
                && node.actions["left_press"].route_id.is_some()
                && node.actions["left_release"].callable_from_remote
                && node.actions["left_release"].route_id.is_some()
                && node.actions["scroll"].callable_from_remote
                && node.actions["scroll"].route_id.is_some()
                && node.actions["resize"].callable_from_remote
                && node.actions["resize"].route_id.is_some()
    ));

    let inspector = service.handle_request(UiControlRequest::QueryNode {
        node_path: UiNodePath::new("editor/workbench/drawers/right_top/editor.inspector#1"),
    });
    assert!(matches!(
        inspector,
        UiControlResponse::Node(Some(node))
            if node.actions["apply_batch"].callable_from_remote
                && node.actions["apply_batch"].route_id.is_some()
    ));
}

#[test]
fn workbench_reflection_call_action_dispatches_docking_inspector_and_viewport_actions() {
    let _guard = env_lock().lock().unwrap();
    let runtime = EventRuntimeHarness::new("zircon_workbench_reflection_runtime");

    let inspector = runtime.runtime.handle_control_request(UiControlRequest::CallAction {
        node_path: UiNodePath::new("editor/workbench/drawers/right_top/editor.inspector#1"),
        action_id: "apply_batch".to_string(),
        arguments: vec![
            UiBindingValue::string("entity://selected"),
            UiBindingValue::array(vec![
                UiBindingValue::array(vec![
                    UiBindingValue::string("name"),
                    UiBindingValue::string("Bound Cube"),
                ]),
                UiBindingValue::array(vec![
                    UiBindingValue::string("transform.translation.x"),
                    UiBindingValue::Float(4.0),
                ]),
            ]),
        ],
    });
    assert!(matches!(
        inspector,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));
    let editor_snapshot = runtime.runtime.editor_snapshot();
    assert_eq!(
        editor_snapshot
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.as_str()),
        Some("Bound Cube")
    );
    assert_eq!(
        editor_snapshot
            .inspector
            .as_ref()
            .map(|inspector| inspector.translation[0].as_str()),
        Some("4.00")
    );

    let viewport = runtime.runtime.handle_control_request(UiControlRequest::CallAction {
        node_path: UiNodePath::new("editor/workbench/pages/workbench/editor.scene#1"),
        action_id: "resize".to_string(),
        arguments: vec![
            UiBindingValue::Unsigned(1024),
            UiBindingValue::Unsigned(768),
        ],
    });
    assert!(matches!(
        viewport,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));
    assert_eq!(
        runtime.runtime.editor_snapshot().viewport_size,
        zircon_math::UVec2::new(1024, 768)
    );

    let docking = runtime.runtime.handle_control_request(UiControlRequest::CallAction {
        node_path: UiNodePath::new("editor/workbench/pages/workbench/editor.scene#1"),
        action_id: "detach_to_window".to_string(),
        arguments: Vec::new(),
    });
    assert!(matches!(
        docking,
        UiControlResponse::Invocation(result)
            if result.error.is_none() && result.value.is_some()
    ));
    assert_eq!(runtime.runtime.current_layout().floating_windows.len(), 1);
}
