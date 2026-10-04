use super::support::*;
use super::*;

#[test]
fn browser_double_click_dispatches_the_selected_asset_open_event() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_browser_double_click_activation");
    let _asset_browser = harness.open_view("editor.asset_browser");
    let mut catalog = asset_drag_source_catalog();
    catalog.assets[0].kind = ResourceKind::AnimationGraph;
    catalog.assets[0].locator = "res://graphs/locomotion.zag".to_string();
    catalog.assets[0].file_name = "locomotion.zag".to_string();
    catalog.assets[0].extension = "zag".to_string();

    {
        let mut host = harness.host.borrow_mut();
        host.runtime.sync_asset_catalog(Arc::new(
            EditorAssetCatalogGeneration::from_snapshot_record(catalog, 1),
        ));
        host.mark_layout_dirty();
        host.refresh_ui();
    }

    let pane = pane_surface_host(&harness.root_ui);
    pane.invoke_asset_content_pointer_clicked("browser".into(), 96.0, 96.0, 0.0, 0.0);
    pane.invoke_asset_content_pointer_clicked("browser".into(), 96.0, 96.0, 0.0, 0.0);

    assert!(
        harness
            .host
            .borrow()
            .runtime
            .journal()
            .records()
            .iter()
            .any(|record| {
                record.event
                    == EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                        asset_locator: "res://graphs/locomotion.zag".to_string(),
                    })
                    && record.result.error.is_none()
                    && record
                        .result
                        .value
                        .as_ref()
                        .and_then(|value| value.get("changed"))
                        .and_then(serde_json::Value::as_bool)
                        == Some(true)
            }),
        "same-row Browser double click should dispatch OpenAsset without a dispatch error"
    );
}

#[test]
fn browser_double_click_rejects_a_search_changed_before_projection_refresh() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_browser_double_click_stale_projection");
    let _asset_browser = harness.open_view("editor.asset_browser");
    let mut catalog = asset_drag_source_catalog();
    catalog.assets[0].kind = ResourceKind::AnimationGraph;
    catalog.assets[0].locator = "res://graphs/locomotion.zag".to_string();
    catalog.assets[0].file_name = "locomotion.zag".to_string();
    catalog.assets[0].extension = "zag".to_string();

    {
        let mut host = harness.host.borrow_mut();
        host.runtime.sync_asset_catalog(Arc::new(
            EditorAssetCatalogGeneration::from_snapshot_record(catalog, 1),
        ));
        host.mark_layout_dirty();
        host.refresh_ui();
    }

    let pane = pane_surface_host(&harness.root_ui);
    pane.invoke_asset_content_pointer_clicked("browser".into(), 96.0, 96.0, 0.0, 0.0);
    let search_record = harness
        .host
        .borrow_mut()
        .runtime
        .dispatch_event(
            crate::core::editor_event::EditorEventSource::RetainedHost,
            EditorEvent::Asset(EditorAssetEvent::SetSearchQuery {
                query: "locomotion".to_string(),
            }),
        )
        .expect("search query dispatch should succeed");
    assert!(search_record.result.error.is_none());
    assert_eq!(
        search_record
            .result
            .value
            .as_ref()
            .and_then(|value| value.get("changed"))
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "search query dispatch should change the current Browser projection"
    );

    // Keep the retained surface unchanged to exercise the committed-projection window.
    pane.invoke_asset_content_pointer_clicked("browser".into(), 96.0, 96.0, 0.0, 0.0);

    let host = harness.host.borrow();
    let stale_status = host
        .runtime
        .context()
        .i18n()
        .translate("asset.activation.target_changed");
    assert_eq!(
        host.runtime.editor_snapshot().status_line,
        stale_status.as_ref(),
        "the stale-projection guard should report the localized changed-target status"
    );
    assert!(
        host.runtime.journal().records().iter().all(|record| {
            record.event
                != EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                    asset_locator: "res://graphs/locomotion.zag".to_string(),
                })
        }),
        "stale retained projection must not open after current Browser query changes"
    );
}

#[test]
fn browser_right_click_opens_the_context_target_asset_not_the_selected_asset() {
    use crate::ui::binding_dispatch::{dispatch_asset_binding, AssetHostEvent};

    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_browser_context_open_target_identity");
    let _asset_browser = harness.open_view("editor.asset_browser");
    let (catalog, target_asset, selected_asset) = asset_drag_source_catalog_with_reference();

    {
        let mut host = harness.host.borrow_mut();
        host.runtime.sync_asset_catalog(Arc::new(
            EditorAssetCatalogGeneration::from_snapshot_record(catalog, 1),
        ));
        host.mark_layout_dirty();
        host.refresh_ui();
        let snapshot = host.runtime.editor_snapshot();
        let target_row = snapshot
            .asset_browser
            .visible_assets
            .get(0)
            .expect("first Browser row should be visible");
        assert_eq!(target_row.uuid, target_asset.uuid);
    }

    let selection_record = harness
        .host
        .borrow_mut()
        .runtime
        .dispatch_event(
            crate::core::editor_event::EditorEventSource::RetainedHost,
            EditorEvent::Asset(EditorAssetEvent::SelectItem {
                asset_uuid: selected_asset.uuid.clone(),
            }),
        )
        .expect("selecting asset B should succeed");
    assert_eq!(
        selection_record
            .result
            .value
            .as_ref()
            .and_then(|value| value.get("changed"))
            .and_then(serde_json::Value::as_bool),
        Some(true),
        "the test must establish asset B as the current selection"
    );

    {
        let mut host = harness.host.borrow_mut();
        host.mark_layout_dirty();
        host.refresh_ui();
        assert_eq!(
            host.runtime
                .editor_snapshot()
                .asset_browser
                .selected_asset_uuid
                .as_deref(),
            Some(selected_asset.uuid.as_str())
        );
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

    let host = harness.host.borrow();
    assert_eq!(
        host.runtime
            .editor_snapshot()
            .asset_browser
            .selected_asset_uuid
            .as_deref(),
        Some(selected_asset.uuid.as_str()),
        "right clicking asset A should leave selected asset B unchanged"
    );
    let expected_target_path = format!(
        "workbench://asset/{}?open_locator=7265733a2f2f677269642e616c6265646f2e706e67",
        target_asset.uuid
    );
    let retained_target_path = host
        .workbench_window_bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .filter(|metadata| metadata.control_id.as_deref() == Some("WorkbenchContextMenu"))
                .and_then(|metadata| metadata.attributes.get("context_target_path"))
                .and_then(|value| value.as_str())
        })
        .expect("right-click should retain the Asset Browser context target path");
    assert_eq!(retained_target_path, expected_target_path);

    let binding = host
        .workbench_window_bridge
        .context_menu_item_binding("WorkbenchContextMenu", "menu.item.asset.open")
        .expect("right-click context menu should expose the Open binding");
    assert_eq!(
        dispatch_asset_binding(&binding).expect("Open binding should dispatch to an asset event"),
        AssetHostEvent::OpenAsset {
            asset_locator: target_asset.locator,
        },
        "Open should dispatch the right-clicked row locator, not selected asset B"
    );
}

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
