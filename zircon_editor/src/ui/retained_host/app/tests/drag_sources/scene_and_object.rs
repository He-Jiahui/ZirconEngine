use super::*;

#[test]
fn hierarchy_pointer_down_arms_scene_instance_payload_for_instance_field_drop() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_hierarchy_drag_source_payload");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.hierarchy#1");

    pane_surface_host(&harness.root_ui).invoke_hierarchy_pointer_event(0, 1, 80.0, 40.0, 0.0, 0.0);

    let reference = {
        let host = harness.host.borrow();
        let payload = host
            .active_scene_drag_payload
            .as_ref()
            .expect("hierarchy row pointer down should arm a scene-instance payload");
        assert_eq!(payload.kind, UiDragPayloadKind::SceneInstance);
        assert!(payload.reference.starts_with("scene://"));
        assert!(payload.source_summary().is_some());
        payload.reference.clone()
    };

    {
        let mut host = harness.host.borrow_mut();
        host.dispatch_component_showcase_control_activated(
            "InstanceFieldDemo",
            "UiComponentShowcase/InstanceFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_scene_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("InstanceFieldDemo", "value")
            .as_deref(),
        Some(reference.as_str())
    );
}

#[test]
fn hierarchy_pointer_up_clears_scene_instance_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_hierarchy_drag_clear");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.hierarchy#1");

    pane_surface_host(&harness.root_ui).invoke_hierarchy_pointer_event(0, 1, 80.0, 40.0, 0.0, 0.0);
    assert!(harness.host.borrow().active_scene_drag_payload.is_some());

    pane_surface_host(&harness.root_ui).invoke_hierarchy_pointer_event(2, 1, 80.0, 40.0, 0.0, 0.0);
    assert!(harness.host.borrow().active_scene_drag_payload.is_none());
}

#[test]
fn object_field_drop_accepts_active_scene_instance_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_object_field_scene_payload_drop");
    {
        let mut host = harness.host.borrow_mut();
        host.active_scene_drag_payload = Some(UiDragPayload::new(
            UiDragPayloadKind::SceneInstance,
            "scene://node/42",
        ));
        host.dispatch_component_showcase_control_activated(
            "ObjectFieldDemo",
            "UiComponentShowcase/ObjectFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_scene_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("ObjectFieldDemo", "value")
            .as_deref(),
        Some("scene://node/42")
    );
}

#[test]
fn asset_field_drop_rejects_active_scene_instance_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_asset_field_rejects_scene_payload");
    {
        let mut host = harness.host.borrow_mut();
        host.active_scene_drag_payload = Some(UiDragPayload::new(
            UiDragPayloadKind::SceneInstance,
            "scene://node/42",
        ));
        host.dispatch_component_showcase_control_activated(
            "AssetFieldDemo",
            "UiComponentShowcase/AssetFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_scene_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("AssetFieldDemo", "value")
            .as_deref(),
        Some("res://textures/grid.albedo.png")
    );

    let projection = host
        .component_showcase_runtime
        .project_document("res://ui/editor/component_showcase.zui")
        .unwrap();
    let surface = host
        .component_showcase_runtime
        .build_shared_surface("res://ui/editor/component_showcase.zui")
        .unwrap();
    let host_projection = host
        .component_showcase_runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap();
    let node = host_projection
        .node_by_control_id("AssetFieldDemo")
        .expect("AssetFieldDemo should be projected after rejected drop");
    assert_eq!(node.validation_level.as_deref(), Some("error"));
    assert_eq!(
        node.validation_message.as_deref(),
        Some("rejected drop payload `scene-instance` for AssetField")
    );
}

#[test]
fn instance_field_drop_rejects_active_asset_payload() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_instance_field_rejects_asset_payload");
    {
        let mut host = harness.host.borrow_mut();
        host.active_asset_drag_payload = Some(UiDragPayload::new(
            UiDragPayloadKind::Asset,
            "res://textures/grid.albedo.png",
        ));
        host.dispatch_component_showcase_control_activated(
            "InstanceFieldDemo",
            "UiComponentShowcase/InstanceFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_asset_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("InstanceFieldDemo", "value")
            .as_deref(),
        Some("scene://Root/CameraRig")
    );

    let projection = host
        .component_showcase_runtime
        .project_document("res://ui/editor/component_showcase.zui")
        .unwrap();
    let surface = host
        .component_showcase_runtime
        .build_shared_surface("res://ui/editor/component_showcase.zui")
        .unwrap();
    let host_projection = host
        .component_showcase_runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap();
    let node = host_projection
        .node_by_control_id("InstanceFieldDemo")
        .expect("InstanceFieldDemo should be projected after rejected drop");
    assert_eq!(node.validation_level.as_deref(), Some("error"));
    assert_eq!(
        node.validation_message.as_deref(),
        Some("rejected drop payload `asset` for InstanceField")
    );
}

#[test]
fn object_field_drop_consumes_active_object_drag_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_object_field_object_payload_drop");
    {
        let mut host = harness.host.borrow_mut();
        host.active_object_drag_payload = Some(
            UiDragPayload::new(UiDragPayloadKind::Object, "object://scene/node/7").with_source(
                UiDragSourceMetadata {
                    source_surface: "inspector".to_string(),
                    source_control_id: "InspectorHeaderPanel".to_string(),
                    locator: Some("object://scene/node/7".to_string()),
                    display_name: Some("Runtime Demo Camera".to_string()),
                    asset_kind: Some("Scene Object".to_string()),
                    ..UiDragSourceMetadata::default()
                },
            ),
        );
        host.dispatch_component_showcase_control_activated(
            "ObjectFieldDemo",
            "UiComponentShowcase/ObjectFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_object_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("ObjectFieldDemo", "value")
            .as_deref(),
        Some("object://scene/node/7")
    );
    let projection = host
        .component_showcase_runtime
        .project_document("res://ui/editor/component_showcase.zui")
        .unwrap();
    let surface = host
        .component_showcase_runtime
        .build_shared_surface("res://ui/editor/component_showcase.zui")
        .unwrap();
    let host_projection = host
        .component_showcase_runtime
        .build_retained_host_projection_with_surface(&projection, &surface)
        .unwrap();
    assert_eq!(
        host_projection
            .node_by_control_id("ObjectFieldDemo")
            .and_then(|node| node.drop_source_summary.as_deref()),
        Some("Scene Object: Runtime Demo Camera")
    );
}

#[test]
fn inspector_pointer_down_arms_active_object_payload_for_object_field_drop() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_inspector_object_drag_source_payload");
    harness.activate_workbench_page();

    let (expected_reference, expected_summary) = {
        let host = harness.host.borrow();
        let inspector = host
            .runtime
            .editor_snapshot()
            .inspector
            .expect("default test scene should have a selected inspector object");
        (
            format!("object://scene/node/{}", inspector.id),
            format!("Scene Object: {}", inspector.name),
        )
    };

    pane_surface_host(&harness.root_ui)
        .invoke_inspector_reference_pointer_event(0, 1, 12.0, 10.0, 260.0, 180.0);

    {
        let host = harness.host.borrow();
        let payload = host
            .active_object_drag_payload
            .as_ref()
            .expect("inspector header pointer down should arm an object payload");
        assert_eq!(payload.kind, UiDragPayloadKind::Object);
        assert_eq!(payload.reference, expected_reference);
        assert_eq!(
            payload.source_summary().as_deref(),
            Some(expected_summary.as_str())
        );
        let source = payload.source.as_ref().expect("object source metadata");
        assert_eq!(source.source_surface, "inspector");
        assert_eq!(source.source_control_id, "InspectorHeaderPanel");
    }

    {
        let mut host = harness.host.borrow_mut();
        host.dispatch_component_showcase_control_activated(
            "ObjectFieldDemo",
            "UiComponentShowcase/ObjectFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_object_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("ObjectFieldDemo", "value")
            .as_deref(),
        Some(expected_reference.as_str())
    );
}

#[test]
fn inspector_pointer_up_clears_active_object_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_inspector_object_drag_clear");
    harness.activate_workbench_page();

    pane_surface_host(&harness.root_ui)
        .invoke_inspector_reference_pointer_event(0, 1, 12.0, 10.0, 260.0, 180.0);
    assert!(harness.host.borrow().active_object_drag_payload.is_some());

    pane_surface_host(&harness.root_ui)
        .invoke_inspector_reference_pointer_event(2, 1, 12.0, 10.0, 260.0, 180.0);
    assert!(harness.host.borrow().active_object_drag_payload.is_none());
}
