use crate::ui::control::EditorUiControlService;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::reflection::{
    build_workbench_reflection_model, register_workbench_reflection_routes,
};
use crate::ui::EditorUiReflectionAdapter;
use zircon_runtime_interface::ui::event_ui::{UiControlRequest, UiControlResponse, UiNodePath};

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
                && node.actions["edit_field"].callable_from_remote
                && node.actions["edit_field"].route_id.is_some()
                && node.actions["create_animation_track"].callable_from_remote
                && node.actions["create_animation_track"].route_id.is_some()
    ));

    let assets = service.handle_request(UiControlRequest::QueryNode {
        node_path: UiNodePath::new("editor/workbench/drawers/left_top/editor.assets#1"),
    });
    assert!(matches!(
        assets,
        UiControlResponse::Node(Some(node))
            if node.actions["set_mesh_import_path"].callable_from_remote
                && node.actions["set_mesh_import_path"].route_id.is_some()
                && node.actions["import_model"].callable_from_remote
                && node.actions["import_model"].route_id.is_some()
    ));
}
