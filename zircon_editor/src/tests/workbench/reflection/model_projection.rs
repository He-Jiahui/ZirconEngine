use crate::core::editor_operation::EditorOperationPath;
use crate::ui::binding::{EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::control::EditorUiControlService;
use crate::ui::workbench::event::editor_operation_binding;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::model::{MenuBarModel, MenuItemModel, MenuModel, WorkbenchViewModel};
use crate::ui::workbench::reflection::{
    activity_descriptors_from_views, build_workbench_reflection_model,
};
use crate::ui::EditorUiReflectionAdapter;
use zircon_runtime_interface::ui::event_ui::{UiControlRequest, UiControlResponse, UiNodePath};

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
                && node.properties["operation_path"].reflected_value
                    == serde_json::json!("File.Project.Save")
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

#[test]
fn workbench_reflection_model_projects_nested_menu_leaves() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let operation_path = EditorOperationPath::parse("Weather.CloudLayer.Refresh").unwrap();
    let mut view_model = WorkbenchViewModel::build(&chrome);
    view_model.menu_bar = MenuBarModel {
        menus: vec![MenuModel {
            label: "Tools".to_string(),
            items: vec![MenuItemModel::branch(
                "Weather",
                vec![MenuItemModel::leaf(
                    "Refresh Cloud Layers",
                    None,
                    editor_operation_binding(&operation_path),
                    Some(operation_path.clone()),
                    Some("Ctrl+Alt+R".to_string()),
                    true,
                )],
            )],
        }],
    };

    let reflection = build_workbench_reflection_model(&chrome, &view_model);

    assert_eq!(reflection.menu_items.len(), 1);
    let item = &reflection.menu_items[0];
    assert_eq!(item.menu_id, "tools");
    assert_eq!(item.control_id, "Weather.CloudLayer.Refresh");
    assert_eq!(item.label, "Refresh Cloud Layers");
    assert_eq!(
        item.operation_path.as_deref(),
        Some(operation_path.as_str())
    );
    assert_eq!(item.shortcut.as_deref(), Some("Ctrl+Alt+R"));
    assert!(matches!(
        item.binding.payload(),
        EditorUiBindingPayload::EditorOperation { operation_id, .. }
            if operation_id == operation_path.as_str()
    ));
    assert_eq!(item.binding.path().event_kind, EditorUiEventKind::Click);
}
