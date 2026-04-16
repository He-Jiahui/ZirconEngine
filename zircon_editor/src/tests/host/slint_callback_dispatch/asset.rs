use super::support::*;

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

#[test]
fn builtin_asset_surface_view_mode_dispatches_dynamic_binding_from_template() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_asset_view_mode");
    let bridge = BuiltinAssetSurfaceTemplateBridge::new().unwrap();

    let effects = dispatch_builtin_asset_surface_control(
        &harness.runtime,
        &bridge,
        "SetViewMode",
        UiEventKind::Change,
        vec![
            zircon_ui::UiBindingValue::string("browser"),
            zircon_ui::UiBindingValue::string("thumbnail"),
        ],
    )
    .expect("asset surface control should resolve through template bridge")
    .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Asset(EditorAssetEvent::SetViewMode {
            surface: crate::EditorAssetSurface::Browser,
            view_mode: crate::EditorAssetViewMode::Thumbnail,
        })
    );
    assert!(effects.presentation_dirty);
    assert!(effects.refresh_visible_asset_previews);
    assert!(!effects.sync_asset_workspace);
}

#[test]
fn builtin_asset_surface_open_browser_dispatches_static_binding_from_template() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_asset_open_browser");
    let bridge = BuiltinAssetSurfaceTemplateBridge::new().unwrap();

    let effects = dispatch_builtin_asset_surface_control(
        &harness.runtime,
        &bridge,
        "OpenAssetBrowser",
        UiEventKind::Click,
        Vec::new(),
    )
    .expect("asset browser control should resolve through template bridge")
    .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Asset(EditorAssetEvent::OpenAssetBrowser)
    );
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn builtin_asset_surface_import_model_dispatches_host_request_from_template() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_asset_import_model");
    let bridge = BuiltinAssetSurfaceTemplateBridge::new().unwrap();

    let effects = dispatch_builtin_asset_surface_control(
        &harness.runtime,
        &bridge,
        "ImportModel",
        UiEventKind::Click,
        Vec::new(),
    )
    .expect("asset import control should resolve through template bridge")
    .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Asset(EditorAssetEvent::ImportModel)
    );
    assert!(effects.import_model_requested);
    assert!(!effects.layout_dirty);
    assert!(!effects.render_dirty);
    assert!(!effects.sync_asset_workspace);
}

#[test]
fn builtin_asset_surface_search_matches_legacy_asset_search_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let legacy_harness = EventRuntimeHarness::new("zircon_slint_parity_asset_search_legacy");
    let legacy_effects = dispatch_asset_search(&legacy_harness.runtime, "cube").unwrap();
    let legacy_record = legacy_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    let builtin_harness = EventRuntimeHarness::new("zircon_slint_parity_asset_search_builtin");
    let bridge = BuiltinAssetSurfaceTemplateBridge::new().unwrap();
    let builtin_effects = dispatch_builtin_asset_surface_control(
        &builtin_harness.runtime,
        &bridge,
        "SearchEdited",
        UiEventKind::Change,
        vec![zircon_ui::UiBindingValue::string("cube")],
    )
    .expect("templated asset search control should resolve")
    .unwrap();
    let builtin_record = builtin_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    assert_eq!(builtin_effects, legacy_effects);
    assert_eq!(builtin_record, legacy_record);
}
