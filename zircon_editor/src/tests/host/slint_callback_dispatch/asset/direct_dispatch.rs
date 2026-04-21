use super::super::support::*;

#[test]
fn asset_search_dispatches_through_runtime_and_requests_asset_sync_effects() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_asset_search");

    let effects = dispatch_asset_search(&harness.runtime, "cube").unwrap();
    let snapshot = harness.runtime.editor_snapshot();

    assert_eq!(snapshot.asset_activity.search_query, "cube");
    assert_eq!(snapshot.asset_browser.search_query, "cube");
    assert!(effects.presentation_dirty);
    assert!(!effects.sync_asset_workspace);
    assert!(!effects.refresh_asset_details);
    assert!(effects.refresh_visible_asset_previews);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
}

#[test]
fn mesh_import_path_edit_dispatch_updates_live_snapshot_without_backend_sync() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_mesh_import_draft");

    let effects = dispatch_mesh_import_path_edit(&harness.runtime, "E:/Models/cube.glb").unwrap();

    assert_eq!(
        harness.runtime.editor_snapshot().mesh_import_path,
        "E:/Models/cube.glb"
    );
    assert!(effects.presentation_dirty);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.sync_asset_workspace);
}

#[test]
fn asset_item_selection_requests_detail_and_preview_refresh_without_backend_sync() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_callback_asset_select");

    let effects =
        dispatch_asset_item_selection(&harness.runtime, "11111111-1111-1111-1111-111111111111")
            .unwrap();

    assert!(effects.presentation_dirty);
    assert!(!effects.sync_asset_workspace);
    assert!(effects.refresh_asset_details);
    assert!(effects.refresh_visible_asset_previews);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
}
