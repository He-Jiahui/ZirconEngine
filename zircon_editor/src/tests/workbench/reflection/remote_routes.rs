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
            if node.actions["workbench.view.focus"].callable_from_remote
                && node.actions["workbench.view.focus"].route_id.is_some()
                && node.actions["workbench.view.detach_to_window"].callable_from_remote
                && node.actions["workbench.view.detach_to_window"].route_id.is_some()
                && node.actions["workbench.viewport.pointer.move"].callable_from_remote
                && node.actions["workbench.viewport.pointer.move"].route_id.is_some()
                && node.actions["workbench.viewport.pointer.left.press"].callable_from_remote
                && node.actions["workbench.viewport.pointer.left.press"].route_id.is_some()
                && node.actions["workbench.viewport.pointer.left.release"].callable_from_remote
                && node.actions["workbench.viewport.pointer.left.release"].route_id.is_some()
                && node.actions["workbench.viewport.scroll"].callable_from_remote
                && node.actions["workbench.viewport.scroll"].route_id.is_some()
                && node.actions["workbench.viewport.resize"].callable_from_remote
                && node.actions["workbench.viewport.resize"].route_id.is_some()
    ));

    let inspector = service.handle_request(UiControlRequest::QueryNode {
        node_path: UiNodePath::new("editor/workbench/drawers/right_top/editor.inspector#1"),
    });
    assert!(matches!(
        inspector,
        UiControlResponse::Node(Some(node))
            if node.actions["inspector.apply_batch.invoke"].callable_from_remote
                && node.actions["inspector.apply_batch.invoke"].route_id.is_some()
                && node.actions["inspector.field.edit"].callable_from_remote
                && node.actions["inspector.field.edit"].route_id.is_some()
                && node.actions["animation.track.create"].callable_from_remote
                && node.actions["animation.track.create"].route_id.is_some()
    ));

    let assets = service.handle_request(UiControlRequest::QueryNode {
        node_path: UiNodePath::new("editor/workbench/drawers/left_top/editor.assets#1"),
    });
    assert!(matches!(
        assets,
        UiControlResponse::Node(Some(node))
            if node.actions["workbench.asset.mesh_import.path.set"].callable_from_remote
                && node.actions["workbench.asset.mesh_import.path.set"].route_id.is_some()
                && node.actions["workbench.asset.model.import"].callable_from_remote
                && node.actions["workbench.asset.model.import"].route_id.is_some()
    ));
}
