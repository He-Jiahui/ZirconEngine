use super::super::support::*;

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
            zircon_runtime::ui::binding::UiBindingValue::string("browser"),
            zircon_runtime::ui::binding::UiBindingValue::string("thumbnail"),
        ],
    )
    .expect("asset surface control should resolve through template bridge")
    .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Asset(EditorAssetEvent::SetViewMode {
            surface: crate::core::editor_event::EditorAssetSurface::Browser,
            view_mode: crate::core::editor_event::EditorAssetViewMode::Thumbnail,
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
        vec![zircon_runtime::ui::binding::UiBindingValue::string("cube")],
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
