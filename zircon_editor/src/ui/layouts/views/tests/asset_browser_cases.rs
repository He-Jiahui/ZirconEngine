#[cfg(test)]
#[test]
fn stable_asset_browser_snapshot_reuses_the_composed_model() {
    super::view_projection::clear_view_template_projection_caches_for_tests();
    clear_asset_browser_pane_projection_cache_for_tests();
    let snapshot = AssetWorkspaceSnapshot::default();
    let stable_snapshot = snapshot.clone();
    let size = UiSize::new(900.0, 620.0);

    let first = asset_browser_pane_nodes(&snapshot, size);
    let stable = asset_browser_pane_nodes(&stable_snapshot, size);

    assert!(first.shares_values_with(&stable));
}

#[cfg(test)]
#[test]
fn stable_asset_browser_snapshot_reuses_the_render_source_frame() {
    super::view_projection::clear_view_template_projection_caches_for_tests();
    clear_asset_browser_pane_projection_cache_for_tests();
    let snapshot = AssetWorkspaceSnapshot::default();
    let stable_snapshot = snapshot.clone();
    let size = UiSize::new(900.0, 620.0);

    let first = asset_browser_pane_data(&snapshot, size);
    let stable = asset_browser_pane_data(&stable_snapshot, size);
    let first_frame = first
        .render_source_frame
        .as_ref()
        .expect("initial projection should publish its runtime source frame");
    let stable_frame = stable
        .render_source_frame
        .as_ref()
        .expect("stable cache hit should retain its runtime source frame");

    assert!(Arc::ptr_eq(first_frame, stable_frame));
}

#[cfg(test)]
#[test]
fn catalog_generation_change_invalidates_the_pane_cache() {
    super::view_projection::clear_view_template_projection_caches_for_tests();
    clear_asset_browser_pane_projection_cache_for_tests();
    let snapshot = AssetWorkspaceSnapshot::default();
    let mut changed = snapshot.clone();
    changed.selection.display_name = "Changed selection".to_string();
    changed.catalog_revision = changed.catalog_revision.wrapping_add(1);
    let size = UiSize::new(900.0, 620.0);

    let first = asset_browser_pane_nodes(&snapshot, size);
    let next = asset_browser_pane_nodes(&changed, size);

    assert!(!first.shares_values_with(&next));
    assert!(next.iter().any(|node| node.text == "Changed selection"));
}

#[cfg(test)]
#[test]
fn mesh_import_path_change_invalidates_the_pane_cache_and_reprojects_the_field() {
    super::view_projection::clear_view_template_projection_caches_for_tests();
    clear_asset_browser_pane_projection_cache_for_tests();
    let snapshot = AssetWorkspaceSnapshot::default();
    let mut changed = snapshot.clone();
    changed.mesh_import_path = "E:/Models/cube.glb".to_string();
    let size = UiSize::new(900.0, 620.0);

    let first = asset_browser_pane_nodes(&snapshot, size);
    let next = asset_browser_pane_nodes(&changed, size);
    let import_path = next
        .iter()
        .find(|node| node.control_id == "AssetBrowserImportPathField")
        .expect("import path field");

    assert!(!first.shares_values_with(&next));
    assert_eq!(import_path.value_text.as_str(), "E:/Models/cube.glb");
}

#[cfg(test)]
#[test]
fn selection_change_reuses_the_generation_owned_logical_paint_source() {
    super::view_projection::clear_view_template_projection_caches_for_tests();
    clear_asset_browser_pane_projection_cache_for_tests();
    let mut snapshot = AssetWorkspaceSnapshot::default();
    snapshot.catalog_revision = 7;
    snapshot.view_mode = AssetViewMode::List;
    snapshot.visible_assets = vec![test_asset_item("asset-a"), test_asset_item("asset-b")].into();
    snapshot.selected_asset_uuid = Some("asset-a".to_string());
    let mut changed = snapshot.clone();
    changed.selected_asset_uuid = Some("asset-b".to_string());
    let size = UiSize::new(900.0, 620.0);

    let first = asset_browser_pane_nodes(&snapshot, size);
    let next = asset_browser_pane_nodes(&changed, size);
    let first_metadata = first
        .metadata_rc::<crate::ui::workbench::asset_content_layout::AssetContentPaintMetadata>()
        .expect("first Asset Browser paint metadata");
    let next_metadata = next
        .metadata_rc::<crate::ui::workbench::asset_content_layout::AssetContentPaintMetadata>()
        .expect("next Asset Browser paint metadata");

    assert!(first_metadata.shares_browser_logical_items_with(&next_metadata));
}

#[cfg(test)]
#[test]
fn local_asset_delta_reuses_unchanged_logical_paint_chunks_in_pane_metadata() {
    super::view_projection::clear_view_template_projection_caches_for_tests();
    clear_asset_browser_pane_projection_cache_for_tests();
    let mut snapshot = AssetWorkspaceSnapshot::default();
    snapshot.catalog_revision = 7;
    snapshot.view_mode = AssetViewMode::List;
    snapshot.visible_assets = (0..130)
        .map(|index| test_asset_item(&format!("asset-{index:03}")))
        .collect();
    let mut changed = snapshot.clone();
    let mut replacement = changed.visible_assets[65].clone();
    replacement.display_name = "changed-asset-065.mesh".to_string();
    changed.catalog_revision = changed.catalog_revision.wrapping_add(1);
    changed.visible_assets = changed
        .visible_assets
        .replace_existing_items([replacement])
        .expect("existing asset delta should preserve visible membership");
    let size = UiSize::new(900.0, 620.0);

    let first = asset_browser_pane_nodes(&snapshot, size);
    let next = asset_browser_pane_nodes(&changed, size);
    let first_metadata = first
        .metadata_rc::<crate::ui::workbench::asset_content_layout::AssetContentPaintMetadata>()
        .expect("first Asset Browser paint metadata");
    let next_metadata = next
        .metadata_rc::<crate::ui::workbench::asset_content_layout::AssetContentPaintMetadata>()
        .expect("next Asset Browser paint metadata");

    assert!(first_metadata.shares_browser_logical_item_chunk_with(0, &next_metadata));
    assert!(!first_metadata.shares_browser_logical_item_chunk_with(65, &next_metadata));
    assert!(first_metadata.shares_browser_logical_item_chunk_with(129, &next_metadata));
}
