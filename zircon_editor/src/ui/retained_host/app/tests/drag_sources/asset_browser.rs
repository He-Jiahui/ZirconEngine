use super::support::*;
use super::*;

#[test]
fn asset_content_pointer_down_arms_active_asset_drag_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_asset_drag_source_payload");
    let _asset_browser = harness.open_view("editor.asset_browser");

    {
        let mut host = harness.host.borrow_mut();
        host.runtime
            .sync_asset_catalog(shared_asset_drag_source_catalog());
        host.mark_layout_dirty();
        host.refresh_ui();
    }

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_event(
        "browser".into(),
        0,
        2,
        96.0,
        96.0,
        0.0,
        0.0,
        0.0,
        0.0,
    );
    assert!(
        harness.host.borrow().active_asset_drag_payload.is_none(),
        "right-button pointer down should not arm an active payload"
    );

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_event(
        "browser".into(),
        0,
        1,
        96.0,
        96.0,
        0.0,
        0.0,
        0.0,
        0.0,
    );

    let host = harness.host.borrow();
    let payload = host
        .active_asset_drag_payload
        .as_ref()
        .expect("asset row pointer down should arm an active payload");
    assert_eq!(payload.kind, UiDragPayloadKind::Asset);
    assert!(payload.reference.starts_with("res://"));
    assert!(payload.source_summary().is_some());
    drop(host);

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_event(
        "browser".into(),
        2,
        1,
        96.0,
        96.0,
        0.0,
        0.0,
        0.0,
        0.0,
    );
    assert!(
        harness.host.borrow().active_asset_drag_payload.is_none(),
        "left-button pointer up should clear the active payload"
    );
}

#[test]
fn asset_reference_pointer_down_arms_active_asset_drag_payload() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_retained_asset_reference_drag_source_payload");
    let _asset_browser = harness.open_view("editor.asset_browser");
    let (catalog, source_asset, reference_asset) = asset_drag_source_catalog_with_reference();

    {
        let mut host = harness.host.borrow_mut();
        host.runtime.sync_asset_catalog(Arc::new(
            EditorAssetCatalogGeneration::from_snapshot_record(catalog, 1),
        ));
        host.mark_layout_dirty();
        host.refresh_ui();
    }

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_clicked(
        "browser".into(),
        96.0,
        96.0,
        0.0,
        0.0,
        0.0,
        0.0,
    );

    {
        let mut host = harness.host.borrow_mut();
        host.runtime
            .sync_asset_details(Some(Arc::new(EditorAssetDetailsGeneration::from(
                EditorAssetDetailsRecord {
                    asset: source_asset,
                    direct_references: vec![EditorAssetReferenceRecord {
                        uuid: reference_asset.uuid.clone(),
                        locator: reference_asset.locator.clone(),
                        display_name: reference_asset.display_name.clone(),
                        kind: Some(reference_asset.kind),
                        known_project_asset: true,
                    }],
                    referenced_by: Vec::new(),
                    package_id: None,
                    unit: AssetSourceUnit::Single,
                    included_files: Vec::new(),
                    subassets: Vec::new(),
                },
            ))));
        host.mark_layout_dirty();
        host.refresh_ui();
    }

    pane_surface_host(&harness.root_ui).invoke_asset_reference_pointer_event(
        "browser".into(),
        "references".into(),
        0,
        2,
        16.0,
        44.0,
        260.0,
        160.0,
    );
    assert!(
        harness.host.borrow().active_asset_drag_payload.is_none(),
        "right-button reference pointer down should not arm an active payload"
    );

    pane_surface_host(&harness.root_ui).invoke_asset_reference_pointer_event(
        "browser".into(),
        "references".into(),
        0,
        1,
        16.0,
        44.0,
        260.0,
        160.0,
    );

    let host = harness.host.borrow();
    let payload = host
        .active_asset_drag_payload
        .as_ref()
        .expect("known reference row pointer down should arm an active payload");
    assert_eq!(payload.kind, UiDragPayloadKind::Asset);
    assert_eq!(payload.reference, "res://materials/runtime_demo.mat");
    assert_eq!(
        payload.source_summary().as_deref(),
        Some("Material: Runtime Demo")
    );
    let source = payload.source.as_ref().expect("source metadata");
    assert_eq!(source.source_surface, "browser.references");
    assert_eq!(source.source_control_id, "AssetBrowserReferenceLeftPanel");
    drop(host);

    pane_surface_host(&harness.root_ui).invoke_asset_reference_pointer_event(
        "browser".into(),
        "references".into(),
        2,
        1,
        16.0,
        44.0,
        260.0,
        160.0,
    );
    assert!(
        harness.host.borrow().active_asset_drag_payload.is_none(),
        "left-button reference pointer up should clear the active payload"
    );
}

#[test]
fn asset_browser_pointer_drop_applies_real_payload_to_showcase_asset_field() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_asset_browser_real_payload_drop");
    let _asset_browser = harness.open_view("editor.asset_browser");

    {
        let mut host = harness.host.borrow_mut();
        host.runtime
            .sync_asset_catalog(shared_asset_drag_source_catalog());
        host.mark_layout_dirty();
        host.refresh_ui();
    }

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_event(
        "browser".into(),
        0,
        1,
        96.0,
        96.0,
        0.0,
        0.0,
        0.0,
        0.0,
    );

    {
        let mut host = harness.host.borrow_mut();
        let payload = host
            .active_asset_drag_payload
            .as_ref()
            .expect("visible asset row pointer down should arm an active payload");
        assert_eq!(payload.reference, "res://grid.albedo.png");
        assert_eq!(
            payload.source_summary().as_deref(),
            Some("Texture: Grid Albedo")
        );

        host.dispatch_component_showcase_control_activated(
            "AssetFieldDemo",
            "UiComponentShowcase/AssetFieldDropped",
        );
    }

    let host = harness.host.borrow();
    assert!(host.active_asset_drag_payload.is_none());
    assert_eq!(
        host.component_showcase_runtime
            .showcase_demo_state()
            .value_text("AssetFieldDemo", "value")
            .as_deref(),
        Some("res://grid.albedo.png")
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
            .node_by_control_id("AssetFieldDemo")
            .and_then(|node| node.drop_source_summary.as_deref()),
        Some("Texture: Grid Albedo")
    );
}

#[test]
fn asset_content_pointer_unknown_surface_clears_active_asset_drag_payload() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_retained_asset_drag_unknown_surface_clear");
    let _asset_browser = harness.open_view("editor.asset_browser");

    {
        let mut host = harness.host.borrow_mut();
        host.runtime
            .sync_asset_catalog(shared_asset_drag_source_catalog());
        host.mark_layout_dirty();
        host.refresh_ui();
    }

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_event(
        "browser".into(),
        0,
        1,
        96.0,
        96.0,
        0.0,
        0.0,
        0.0,
        0.0,
    );
    assert!(harness.host.borrow().active_asset_drag_payload.is_some());

    pane_surface_host(&harness.root_ui).invoke_asset_content_pointer_event(
        "unknown".into(),
        0,
        1,
        96.0,
        96.0,
        0.0,
        0.0,
        0.0,
        0.0,
    );
    assert!(
        harness.host.borrow().active_asset_drag_payload.is_none(),
        "unknown asset surface should clear stale active payload"
    );
}
