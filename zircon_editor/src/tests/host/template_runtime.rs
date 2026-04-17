use toml::Value;
use zircon_editor_ui::EditorUiControlService;
use zircon_ui::{UiEventKind, UiFrame, UiInputPolicy, UiSize};

use crate::{
    EditorUiCompatibilityHarness, EditorUiHostRuntime, SlintUiHostAdapter,
    SlintUiHostComponentKind, SlintUiHostValue,
};

#[test]
fn editor_ui_host_runtime_projects_builtin_workbench_template_into_slint_projection() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("WorkbenchShell")
            .unwrap()
            .binding_namespace,
        "WorkbenchShell"
    );

    let projection = runtime.project_document("workbench.shell").unwrap();

    assert_eq!(projection.document_id, "workbench.shell");
    assert_eq!(projection.root.component, "WorkbenchShell");
    assert_eq!(
        projection
            .root
            .children
            .iter()
            .map(|node| node.component.as_str())
            .collect::<Vec<_>>(),
        vec!["VerticalBox", "Container", "Overlay", "Overlay", "Overlay"]
    );

    let open_project = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "WorkbenchMenuBar/OpenProject")
        .unwrap();
    assert_eq!(open_project.binding.path().event_kind, UiEventKind::Click);
    assert_eq!(open_project.binding.path().view_id, "WorkbenchMenuBar");
    assert_eq!(open_project.binding.path().control_id, "OpenProject");
}

#[test]
fn editor_ui_host_runtime_projects_builtin_viewport_toolbar_template_into_slint_projection() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("SceneViewportToolbar")
            .unwrap()
            .binding_namespace,
        "ViewportToolbar"
    );

    let projection = runtime.project_document("scene.viewport_toolbar").unwrap();

    assert_eq!(projection.document_id, "scene.viewport_toolbar");
    assert_eq!(projection.root.component, "SceneViewportToolbar");
    assert_eq!(
        projection
            .root
            .children
            .iter()
            .map(|node| node.control_id.as_deref().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec![
            "SceneViewportToolbarLeftGroup",
            "SceneViewportToolbarRightGroup",
        ]
    );

    let set_tool = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "ViewportToolbar/SetTool")
        .unwrap();
    assert_eq!(set_tool.binding.path().event_kind, UiEventKind::Change);
    assert_eq!(set_tool.binding.path().view_id, "ViewportToolbar");
    assert_eq!(set_tool.binding.path().control_id, "SetTool");

    let frame_selection = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "ViewportToolbar/FrameSelection")
        .unwrap();
    assert_eq!(
        frame_selection.binding.path().event_kind,
        UiEventKind::Click
    );
    assert_eq!(frame_selection.binding.path().view_id, "ViewportToolbar");
    assert_eq!(frame_selection.binding.path().control_id, "FrameSelection");
}

#[test]
fn editor_ui_host_runtime_builds_surface_backed_viewport_toolbar_group_frames() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();
    let mut projection = runtime.project_document("scene.viewport_toolbar").unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let mut surface = runtime
        .build_shared_surface("scene.viewport_toolbar")
        .unwrap();
    surface.compute_layout(UiSize::new(1280.0, 28.0)).unwrap();

    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();

    let root = host_model
        .node_by_control_id("SceneViewportToolbarRoot")
        .expect("viewport toolbar root should exist");
    assert_eq!(root.frame, UiFrame::new(0.0, 0.0, 1280.0, 28.0));

    let set_tool = host_model
        .node_by_control_id("SetTool")
        .expect("set tool group should exist");
    assert_eq!(set_tool.frame, UiFrame::new(8.0, 4.0, 172.0, 20.0));

    let set_transform_space = host_model
        .node_by_control_id("SetTransformSpace")
        .expect("transform space group should exist");
    assert_eq!(
        set_transform_space.frame,
        UiFrame::new(189.0, 4.0, 86.0, 20.0)
    );

    let set_projection_mode = host_model
        .node_by_control_id("SetProjectionMode")
        .expect("projection mode group should exist");
    assert_eq!(
        set_projection_mode.frame,
        UiFrame::new(958.0, 4.0, 92.0, 20.0)
    );

    let align_view = host_model
        .node_by_control_id("AlignView")
        .expect("align view group should exist");
    assert_eq!(align_view.frame, UiFrame::new(1054.0, 4.0, 200.0, 20.0));

    let frame_selection = host_model
        .node_by_control_id("FrameSelection")
        .expect("frame selection control should exist");
    assert_eq!(frame_selection.frame, UiFrame::new(649.0, 4.0, 20.0, 20.0));
}

#[test]
fn editor_ui_host_runtime_projects_builtin_asset_surface_template_into_slint_projection() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("AssetSurfaceControls")
            .unwrap()
            .binding_namespace,
        "AssetSurface"
    );

    let projection = runtime.project_document("asset.surface_controls").unwrap();

    assert_eq!(projection.document_id, "asset.surface_controls");
    assert_eq!(projection.root.component, "AssetSurfaceControls");
    assert_eq!(
        projection
            .root
            .children
            .iter()
            .map(|node| node.control_id.as_deref().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec![
            "SelectFolder",
            "SelectItem",
            "SearchEdited",
            "SetKindFilter",
            "SetViewMode",
            "SetUtilityTab",
            "ActivateReference",
            "OpenAssetBrowser",
            "LocateSelectedAsset",
            "ImportModel",
        ]
    );

    let search = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "AssetSurface/SearchEdited")
        .unwrap();
    assert_eq!(search.binding.path().event_kind, UiEventKind::Change);
    assert_eq!(search.binding.path().view_id, "AssetSurface");
    assert_eq!(search.binding.path().control_id, "SearchEdited");

    let open_browser = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "AssetSurface/OpenAssetBrowser")
        .unwrap();
    assert_eq!(open_browser.binding.path().event_kind, UiEventKind::Click);
    assert_eq!(open_browser.binding.path().view_id, "AssetSurface");
    assert_eq!(open_browser.binding.path().control_id, "OpenAssetBrowser");
}

#[test]
fn editor_ui_host_runtime_projects_builtin_welcome_surface_template_into_slint_projection() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("WelcomeSurfaceControls")
            .unwrap()
            .binding_namespace,
        "WelcomeSurface"
    );

    let projection = runtime
        .project_document("startup.welcome_controls")
        .unwrap();

    assert_eq!(projection.document_id, "startup.welcome_controls");
    assert_eq!(projection.root.component, "WelcomeSurfaceControls");
    assert_eq!(
        projection
            .root
            .children
            .iter()
            .map(|node| node.control_id.as_deref().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec![
            "ProjectNameEdited",
            "LocationEdited",
            "CreateProject",
            "OpenExistingProject",
            "OpenRecentProject",
            "RemoveRecentProject",
        ]
    );

    let project_name = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "WelcomeSurface/ProjectNameEdited")
        .unwrap();
    assert_eq!(project_name.binding.path().event_kind, UiEventKind::Change);
    assert_eq!(project_name.binding.path().view_id, "WelcomeSurface");
    assert_eq!(project_name.binding.path().control_id, "ProjectNameEdited");

    let remove_recent = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "WelcomeSurface/RemoveRecentProject")
        .unwrap();
    assert_eq!(remove_recent.binding.path().event_kind, UiEventKind::Click);
    assert_eq!(remove_recent.binding.path().view_id, "WelcomeSurface");
    assert_eq!(
        remove_recent.binding.path().control_id,
        "RemoveRecentProject"
    );
}

#[test]
fn editor_ui_host_runtime_projects_builtin_inspector_surface_template_into_slint_projection() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();

    assert_eq!(
        runtime
            .component_descriptor("InspectorSurfaceControls")
            .unwrap()
            .binding_namespace,
        "InspectorView"
    );

    let projection = runtime
        .project_document("inspector.surface_controls")
        .unwrap();

    assert_eq!(projection.document_id, "inspector.surface_controls");
    assert_eq!(projection.root.component, "InspectorSurfaceControls");
    assert_eq!(
        projection
            .root
            .children
            .iter()
            .map(|node| node.control_id.as_deref().unwrap_or_default())
            .collect::<Vec<_>>(),
        vec![
            "NameField",
            "ParentField",
            "PositionXField",
            "PositionYField",
            "PositionZField",
            "ApplyBatchButton",
            "DeleteSelected",
        ]
    );

    let name = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "InspectorView/NameField")
        .unwrap();
    assert_eq!(name.binding.path().event_kind, UiEventKind::Change);
    assert_eq!(name.binding.path().view_id, "InspectorView");
    assert_eq!(name.binding.path().control_id, "NameField");

    let apply = projection
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "InspectorView/ApplyBatchButton")
        .unwrap();
    assert_eq!(apply.binding.path().event_kind, UiEventKind::Click);
    assert_eq!(apply.binding.path().view_id, "InspectorView");
    assert_eq!(apply.binding.path().control_id, "ApplyBatchButton");
}

#[test]
fn editor_ui_host_runtime_registers_projection_bindings_as_route_stubs() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();
    let mut projection = runtime.project_document("workbench.shell").unwrap();
    let mut service = EditorUiControlService::default();

    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();

    assert!(!projection.bindings.is_empty());
    for binding in &projection.bindings {
        let route_id = binding.route_id.expect("route id");
        assert_eq!(
            service.route_binding(route_id).unwrap(),
            binding.binding.as_ui_binding()
        );
    }
}

#[test]
fn editor_ui_compatibility_harness_captures_projection_shape_for_parity_checks() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();
    let projection = runtime.project_document("workbench.shell").unwrap();

    let snapshot = EditorUiCompatibilityHarness::capture_projection_snapshot(&projection);

    assert_eq!(
        snapshot.components,
        vec![
            "WorkbenchShell",
            "VerticalBox",
            "UiHostToolbar",
            "UiHostIconButton",
            "UiHostIconButton",
            "UiHostIconButton",
            "HorizontalBox",
            "ActivityRail",
            "UiHostIconButton",
            "UiHostIconButton",
            "UiHostIconButton",
            "DocumentHost",
            "DocumentTabs",
            "PaneSurface",
            "StatusBar",
            "UiHostLabel",
            "Container",
            "Overlay",
            "Container",
            "Container",
            "Overlay",
            "Container",
            "Container",
            "Overlay",
            "Container",
            "Container",
        ]
    );
    assert!(snapshot
        .control_ids
        .contains(&"WorkbenchMenuBarRoot".to_string()));
    assert!(snapshot
        .binding_ids
        .contains(&"WorkbenchMenuBar/ResetLayout".to_string()));
}

#[test]
fn editor_ui_host_runtime_builds_host_node_model_with_routes_and_attributes() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();
    let mut projection = runtime.project_document("workbench.shell").unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();

    let host_model = runtime.build_host_model(&projection).unwrap();

    assert_eq!(host_model.document_id, "workbench.shell");
    assert_eq!(
        host_model
            .nodes
            .iter()
            .map(|node| node.component.as_str())
            .collect::<Vec<_>>(),
        vec![
            "WorkbenchShell",
            "VerticalBox",
            "UiHostToolbar",
            "UiHostIconButton",
            "UiHostIconButton",
            "UiHostIconButton",
            "HorizontalBox",
            "ActivityRail",
            "UiHostIconButton",
            "UiHostIconButton",
            "UiHostIconButton",
            "DocumentHost",
            "DocumentTabs",
            "PaneSurface",
            "StatusBar",
            "UiHostLabel",
            "Container",
            "Overlay",
            "Container",
            "Container",
            "Overlay",
            "Container",
            "Container",
            "Overlay",
            "Container",
            "Container",
        ]
    );

    let open_project = host_model
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some("OpenProject"))
        .unwrap();
    assert_eq!(open_project.node_id, "root.0.0.0");
    assert_eq!(open_project.parent_id.as_deref(), Some("root.0.0"));
    assert_eq!(
        open_project.attributes.get("icon"),
        Some(&Value::String("folder-open-outline".to_string()))
    );
    assert_eq!(
        open_project.attributes.get("label"),
        Some(&Value::String("Open".to_string()))
    );
    let open_project_binding = open_project
        .bindings
        .iter()
        .find(|binding| binding.binding_id == "WorkbenchMenuBar/OpenProject")
        .unwrap();
    assert_eq!(open_project_binding.event_kind, UiEventKind::Click);
    let route_id = open_project_binding.route_id.expect("registered route");
    assert_eq!(
        service.route_binding(route_id).unwrap(),
        projection
            .bindings
            .iter()
            .find(|binding| binding.binding_id == "WorkbenchMenuBar/OpenProject")
            .unwrap()
            .binding
            .as_ui_binding()
    );

    let status_text = host_model
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some("StatusText"))
        .unwrap();
    assert_eq!(status_text.node_id, "root.0.2.0");
    assert_eq!(
        status_text.attributes.get("text"),
        Some(&Value::String("Ready".to_string()))
    );
}

#[test]
fn editor_ui_compatibility_harness_captures_host_model_routes_and_attributes() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();
    let mut projection = runtime.project_document("workbench.shell").unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let host_model = runtime.build_host_model(&projection).unwrap();

    let snapshot = EditorUiCompatibilityHarness::capture_host_model_snapshot(&host_model);

    assert!(snapshot
        .host_nodes
        .contains(&"root.0.0.0|UiHostIconButton|OpenProject".to_string()));
    assert!(snapshot
        .host_nodes
        .contains(&"root.0.2.0|UiHostLabel|StatusText".to_string()));
    assert!(snapshot
        .route_bindings
        .iter()
        .any(|entry| entry.starts_with("WorkbenchMenuBar/OpenProject@")));
    assert!(snapshot
        .attribute_entries
        .contains(&"root.0.0.0.icon=folder-open-outline".to_string()));
    assert!(snapshot
        .attribute_entries
        .contains(&"root.0.2.0.text=Ready".to_string()));
}

#[test]
fn slint_ui_host_adapter_builds_generic_projection_from_host_model() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();
    let mut projection = runtime.project_document("workbench.shell").unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let host_model = runtime.build_host_model(&projection).unwrap();

    let slint_projection = SlintUiHostAdapter::build_projection(&host_model);

    assert_eq!(slint_projection.document_id, "workbench.shell");
    assert_eq!(
        slint_projection
            .nodes
            .iter()
            .map(|node| node.kind)
            .collect::<Vec<_>>(),
        vec![
            SlintUiHostComponentKind::Root,
            SlintUiHostComponentKind::VerticalBox,
            SlintUiHostComponentKind::Toolbar,
            SlintUiHostComponentKind::IconButton,
            SlintUiHostComponentKind::IconButton,
            SlintUiHostComponentKind::IconButton,
            SlintUiHostComponentKind::HorizontalBox,
            SlintUiHostComponentKind::ActivityRail,
            SlintUiHostComponentKind::IconButton,
            SlintUiHostComponentKind::IconButton,
            SlintUiHostComponentKind::IconButton,
            SlintUiHostComponentKind::DocumentHost,
            SlintUiHostComponentKind::TabStrip,
            SlintUiHostComponentKind::PaneSurface,
            SlintUiHostComponentKind::StatusBar,
            SlintUiHostComponentKind::Label,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
            SlintUiHostComponentKind::Unknown,
        ]
    );

    let open_project = slint_projection.node_by_control_id("OpenProject").unwrap();
    assert_eq!(open_project.node_id, "root.0.0.0");
    assert_eq!(open_project.parent_id.as_deref(), Some("root.0.0"));
    assert_eq!(open_project.kind, SlintUiHostComponentKind::IconButton);
    assert_eq!(open_project.text.as_deref(), Some("Open"));
    assert_eq!(open_project.icon.as_deref(), Some("folder-open-outline"));
    assert_eq!(
        open_project.properties.get("label").cloned().unwrap(),
        SlintUiHostValue::String("Open".to_string())
    );
    let click_route = open_project
        .routes
        .iter()
        .find(|route| route.binding_id == "WorkbenchMenuBar/OpenProject")
        .unwrap()
        .route_id
        .expect("click route");
    assert_eq!(
        service.route_binding(click_route).unwrap(),
        projection
            .bindings
            .iter()
            .find(|binding| binding.binding_id == "WorkbenchMenuBar/OpenProject")
            .unwrap()
            .binding
            .as_ui_binding()
    );

    let status_text = slint_projection.node_by_control_id("StatusText").unwrap();
    assert_eq!(status_text.kind, SlintUiHostComponentKind::Label);
    assert_eq!(status_text.text.as_deref(), Some("Ready"));
    assert_eq!(status_text.icon, None);
}

#[test]
fn editor_ui_host_runtime_builds_slint_host_projection_and_snapshot() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();
    let mut projection = runtime.project_document("workbench.shell").unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();

    let slint_projection = runtime.build_slint_host_projection(&projection).unwrap();
    let snapshot =
        EditorUiCompatibilityHarness::capture_slint_host_projection_snapshot(&slint_projection);

    assert_eq!(slint_projection.nodes.len(), 26);
    assert!(snapshot
        .slint_nodes
        .contains(&"root.0.0.0|IconButton|OpenProject".to_string()));
    assert!(snapshot
        .slint_nodes
        .contains(&"root.0.2.0|Label|StatusText".to_string()));
    assert!(snapshot
        .text_entries
        .contains(&"root.0.0.0=Open".to_string()));
    assert!(snapshot
        .icon_entries
        .contains(&"root.0.0.0=folder-open-outline".to_string()));
    assert!(snapshot
        .route_bindings
        .iter()
        .any(|entry| entry.starts_with("WorkbenchMenuBar/OpenProject@")));
}

#[test]
fn editor_ui_host_runtime_builds_shared_surface_for_builtin_template() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();

    let surface = runtime.build_shared_surface("workbench.shell").unwrap();

    assert_eq!(surface.tree.tree_id.0, "template.workbench.shell");
    assert_eq!(surface.tree.roots.len(), 1);
    assert_eq!(surface.tree.nodes.len(), 26);
    assert_eq!(surface.render_extract.tree_id.0, "template.workbench.shell");
    assert_eq!(surface.render_extract.list.commands.len(), 26);

    let open_project = surface
        .tree
        .nodes
        .values()
        .find(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                == Some("OpenProject")
        })
        .unwrap();
    let template = open_project.template_metadata.as_ref().unwrap();
    assert_eq!(template.component, "UiHostIconButton");
    assert_eq!(
        template.attributes.get("icon").unwrap().as_str(),
        Some("folder-open-outline")
    );
    assert_eq!(template.bindings[0].id, "WorkbenchMenuBar/OpenProject");
    assert_eq!(open_project.input_policy, UiInputPolicy::Receive);
    assert!(open_project.state_flags.clickable);
    assert!(open_project.state_flags.hoverable);
    assert!(open_project.state_flags.focusable);
}

#[test]
fn editor_ui_compatibility_harness_captures_shared_surface_snapshot() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();
    let surface = runtime.build_shared_surface("workbench.shell").unwrap();

    let snapshot = EditorUiCompatibilityHarness::capture_shared_surface_snapshot(&surface);

    assert!(snapshot
        .surface_nodes
        .contains(&"root|WorkbenchShell|WorkbenchShellRoot".to_string()));
    assert!(snapshot.surface_nodes.contains(
        &"root/WorkbenchScaffold_0/WorkbenchMenuBarRoot_0/OpenProject_0|UiHostIconButton|OpenProject"
            .to_string()
    ));
    assert!(snapshot.surface_nodes.contains(
        &"root/WorkbenchScaffold_0/StatusBarRoot_2/StatusText_0|UiHostLabel|StatusText".to_string()
    ));
    assert!(snapshot.attribute_entries.contains(
        &"root/WorkbenchScaffold_0/WorkbenchMenuBarRoot_0/OpenProject_0.icon=folder-open-outline"
            .to_string()
    ));
    assert!(snapshot
        .binding_ids
        .contains(&"WorkbenchMenuBar/OpenProject".to_string()));
    assert!(snapshot
        .binding_ids
        .contains(&"WorkbenchShell/ActivateMainPage".to_string()));
}

#[test]
fn editor_ui_host_runtime_builds_laid_out_host_model_from_shared_surface_authority() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();
    let mut projection = runtime.project_document("workbench.shell").unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let mut surface = runtime.build_shared_surface("workbench.shell").unwrap();
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();

    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();

    let menu_bar = host_model
        .node_by_control_id("WorkbenchMenuBarRoot")
        .unwrap();
    assert_eq!(
        menu_bar.node_id,
        "root/WorkbenchScaffold_0/WorkbenchMenuBarRoot_0"
    );
    assert_eq!(
        menu_bar.parent_id.as_deref(),
        Some("root/WorkbenchScaffold_0")
    );
    assert_eq!(menu_bar.frame, UiFrame::new(0.0, 0.0, 1280.0, 40.0));

    let open_project = host_model.node_by_control_id("OpenProject").unwrap();
    assert_eq!(
        open_project.node_id,
        "root/WorkbenchScaffold_0/WorkbenchMenuBarRoot_0/OpenProject_0"
    );
    assert_eq!(
        open_project.parent_id.as_deref(),
        Some("root/WorkbenchScaffold_0/WorkbenchMenuBarRoot_0")
    );
    assert_eq!(open_project.frame, UiFrame::new(0.0, 0.0, 120.0, 32.0));

    let activity_rail = host_model.node_by_control_id("ActivityRailRoot").unwrap();
    assert_eq!(activity_rail.frame, UiFrame::new(0.0, 40.0, 56.0, 656.0));

    let document_host = host_model.node_by_control_id("DocumentHostRoot").unwrap();
    assert_eq!(document_host.frame, UiFrame::new(56.0, 40.0, 1224.0, 656.0));

    let tabs = host_model.node_by_control_id("DocumentTabsRoot").unwrap();
    assert_eq!(tabs.frame, UiFrame::new(56.0, 40.0, 1224.0, 32.0));

    let pane_surface = host_model.node_by_control_id("PaneSurfaceRoot").unwrap();
    assert_eq!(pane_surface.frame, UiFrame::new(56.0, 72.0, 1224.0, 624.0));

    let status_bar = host_model.node_by_control_id("StatusBarRoot").unwrap();
    assert_eq!(status_bar.frame, UiFrame::new(0.0, 696.0, 1280.0, 24.0));
}

#[test]
fn editor_ui_compatibility_harness_captures_shared_layout_frames_from_surface_and_slint_projection()
{
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_workbench_shell().unwrap();
    let mut projection = runtime.project_document("workbench.shell").unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let mut surface = runtime.build_shared_surface("workbench.shell").unwrap();
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();
    let slint_projection = runtime
        .build_slint_host_projection_with_surface(&projection, &surface)
        .unwrap();

    let surface_snapshot = EditorUiCompatibilityHarness::capture_shared_surface_snapshot(&surface);
    let host_snapshot = EditorUiCompatibilityHarness::capture_host_model_snapshot(&host_model);
    let slint_snapshot =
        EditorUiCompatibilityHarness::capture_slint_host_projection_snapshot(&slint_projection);

    assert!(surface_snapshot
        .frame_entries
        .contains(&"root/WorkbenchScaffold_0/WorkbenchMenuBarRoot_0=0,0,1280,40".to_string()));
    assert!(host_snapshot.frame_entries.contains(
        &"root/WorkbenchScaffold_0/WorkbenchBody_1/DocumentHostRoot_1=56,40,1224,656".to_string()
    ));
    assert!(slint_snapshot
        .frame_entries
        .contains(&"root/WorkbenchScaffold_0/StatusBarRoot_2=0,696,1280,24".to_string()));
}

#[test]
fn editor_template_runtime_splits_builtin_data_from_runtime_pipeline() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("host")
        .join("template_runtime");

    for relative in [
        "runtime/mod.rs",
        "runtime/runtime_host.rs",
        "runtime/build_session.rs",
        "runtime/projection.rs",
        "builtin/mod.rs",
        "builtin/template_documents.rs",
        "builtin/template_bindings.rs",
        "builtin/component_descriptors.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected template runtime module {relative} under {:?}",
            root
        );
    }
}
