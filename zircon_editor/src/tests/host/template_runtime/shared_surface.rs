use super::support::*;

#[test]
fn editor_ui_host_runtime_builds_shared_surface_for_builtin_template() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();

    let surface = runtime
        .build_shared_surface(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();

    assert_eq!(
        surface.tree.tree_id.0,
        format!("template.v2.{UI_HOST_WINDOW_DOCUMENT_ID}")
    );
    assert_eq!(surface.tree.roots.len(), 1);
    assert_eq!(surface.tree.nodes.len(), 27);
    assert_eq!(
        surface.render_extract.tree_id.0,
        format!("template.v2.{UI_HOST_WINDOW_DOCUMENT_ID}")
    );
    assert!(surface.render_extract.list.commands.is_empty());
    assert!(surface.arranged_tree.nodes.is_empty());

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
    assert_eq!(template.component, "IconButton");
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
    runtime.load_builtin_host_templates().unwrap();
    let surface = runtime
        .build_shared_surface(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();

    let snapshot = EditorUiCompatibilityHarness::capture_shared_surface_snapshot(&surface);

    assert!(snapshot
        .surface_nodes
        .contains(&"v2/host|UiHostWindow|UiHostWindowRoot".to_string()));
    assert!(snapshot
        .surface_nodes
        .contains(&"v2/OpenProject|IconButton|OpenProject".to_string()));
    assert!(snapshot
        .surface_nodes
        .contains(&"v2/StatusText|Label|StatusText".to_string()));
    assert!(snapshot
        .attribute_entries
        .contains(&"v2/OpenProject.icon=folder-open-outline".to_string()));
    assert!(snapshot
        .binding_ids
        .contains(&"WorkbenchMenuBar/OpenProject".to_string()));
    assert!(snapshot
        .binding_ids
        .contains(&"UiHostWindow/ActivateMainPage".to_string()));
}

#[test]
fn editor_ui_host_runtime_builds_laid_out_host_model_from_shared_surface_authority() {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let mut surface = runtime
        .build_shared_surface(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();

    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();

    let menu_bar = host_model
        .node_by_control_id("WorkbenchMenuBarRoot")
        .unwrap();
    assert_eq!(menu_bar.node_id, "v2/WorkbenchMenuBarRoot");
    assert_eq!(menu_bar.parent_id.as_deref(), Some("v2/WorkbenchScaffold"));
    assert_eq!(menu_bar.frame, UiFrame::new(0.0, 0.0, 1280.0, 24.0));

    let open_project = host_model.node_by_control_id("OpenProject").unwrap();
    assert_eq!(open_project.node_id, "v2/OpenProject");
    assert_eq!(
        open_project.parent_id.as_deref(),
        Some("v2/WorkbenchMenuBarRoot")
    );
    assert_eq!(open_project.frame, UiFrame::new(0.0, 0.0, 76.0, 24.0));

    let activity_rail = host_model.node_by_control_id("ActivityRailRoot").unwrap();
    assert_eq!(activity_rail.frame, UiFrame::new(0.0, 57.0, 44.0, 639.0));

    let document_host = host_model.node_by_control_id("DocumentHostRoot").unwrap();
    assert_eq!(document_host.frame, UiFrame::new(44.0, 57.0, 1236.0, 639.0));

    let tabs = host_model.node_by_control_id("DocumentTabsRoot").unwrap();
    assert_eq!(tabs.frame, UiFrame::new(44.0, 57.0, 1236.0, 32.0));

    let pane_surface = host_model.node_by_control_id("PaneSurfaceRoot").unwrap();
    assert_eq!(pane_surface.frame, UiFrame::new(44.0, 89.0, 1236.0, 607.0));

    let status_bar = host_model.node_by_control_id("StatusBarRoot").unwrap();
    assert_eq!(status_bar.frame, UiFrame::new(0.0, 696.0, 1280.0, 24.0));
}

#[test]
fn editor_ui_compatibility_harness_captures_shared_layout_frames_from_surface_and_retained_projection(
) {
    let mut runtime = EditorUiHostRuntime::default();
    runtime.load_builtin_host_templates().unwrap();
    let mut projection = runtime
        .project_document(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    let mut service = EditorUiControlService::default();
    runtime
        .register_projection_routes(&mut service, &mut projection)
        .unwrap();
    let mut surface = runtime
        .build_shared_surface(UI_HOST_WINDOW_DOCUMENT_ID)
        .unwrap();
    surface.compute_layout(UiSize::new(1280.0, 720.0)).unwrap();
    let host_model = runtime
        .build_host_model_with_surface(&projection, &surface)
        .unwrap();
    let retained_projection = runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap();

    let surface_snapshot = EditorUiCompatibilityHarness::capture_shared_surface_snapshot(&surface);
    let host_snapshot = EditorUiCompatibilityHarness::capture_host_model_snapshot(&host_model);
    let retained_snapshot = EditorUiCompatibilityHarness::capture_retained_host_projection_snapshot(
        &retained_projection,
    );

    assert!(surface_snapshot
        .frame_entries
        .contains(&"v2/WorkbenchMenuBarRoot=0,0,1280,24".to_string()));
    assert!(host_snapshot
        .frame_entries
        .contains(&"v2/DocumentHostRoot=44,57,1236,639".to_string()));
    assert!(retained_snapshot
        .frame_entries
        .contains(&"v2/StatusBarRoot=0,696,1280,24".to_string()));
}
