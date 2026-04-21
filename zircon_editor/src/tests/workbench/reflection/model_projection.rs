use crate::ui::control::EditorUiControlService;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::reflection::{
    activity_descriptors_from_views, build_workbench_reflection_model,
};
use crate::ui::EditorUiReflectionAdapter;
use zircon_runtime::ui::event_ui::{UiControlRequest, UiControlResponse, UiNodePath};

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
    assert!(windows
        .iter()
        .any(|descriptor| descriptor.window_id == "editor.ui_asset"));

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
                && node.actions.contains_key("edit_field")
                && node.actions.contains_key("create_animation_track")
                && node.actions["apply_batch"].binding_symbol == "InspectorFieldBatch"
                && node.actions["edit_field"].binding_symbol
                    == "DraftCommand.SetInspectorField"
                && node.actions["create_animation_track"].binding_symbol
                    == "AnimationCommand.CreateTrack"
    ));

    let assets = service.handle_request(UiControlRequest::QueryNode {
        node_path: UiNodePath::new("editor/workbench/drawers/left_top/editor.assets#1"),
    });
    assert!(matches!(
        assets,
        UiControlResponse::Node(Some(node))
            if node.actions.contains_key("set_mesh_import_path")
                && node.actions.contains_key("import_model")
                && node.actions["set_mesh_import_path"].binding_symbol
                    == "DraftCommand.SetMeshImportPath"
                && node.actions["import_model"].binding_symbol
                    == "AssetCommand.ImportModel"
    ));
}
