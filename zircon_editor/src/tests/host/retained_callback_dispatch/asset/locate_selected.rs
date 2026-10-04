use super::super::support::*;
use crate::core::i18n::EditorLocale;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
    EditorAssetFolderRecord,
};
use std::sync::Arc;
use zircon_runtime::asset::project::PreviewState;
use zircon_runtime_interface::resource::ResourceKind;
use zircon_runtime_interface::ui::binding::UiBindingValue;

#[test]
fn locate_selected_asset_control_navigates_to_the_selected_asset_folder() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_asset_locate_selected");
    let bridge = BuiltinAssetSurfaceTemplateBridge::new()
        .expect("builtin asset surface bridge should build");
    harness.runtime.sync_asset_catalog(shared_catalog());

    dispatch_builtin_asset_surface_control(
        &harness.runtime,
        &bridge,
        "SelectItem",
        UiEventKind::Change,
        vec![UiBindingValue::string(
            "22222222-2222-2222-2222-222222222222",
        )],
    )
    .expect("asset item control should resolve")
    .expect("asset item selection should dispatch");

    let before = harness.runtime.editor_snapshot();
    assert_eq!(
        before.asset_browser.selected_folder_id.as_deref(),
        Some("res://")
    );
    assert_eq!(
        before.asset_browser.selected_asset_uuid.as_deref(),
        Some("22222222-2222-2222-2222-222222222222")
    );
    let views_before = harness.runtime.current_view_instances();
    let layout_before = harness.runtime.current_layout();

    let effects = dispatch_builtin_asset_surface_control(
        &harness.runtime,
        &bridge,
        "LocateSelectedAsset",
        UiEventKind::Click,
        Vec::new(),
    )
    .expect("locate control should resolve")
    .expect("locate action should dispatch");

    let after = harness.runtime.editor_snapshot();
    let views_after = harness.runtime.current_view_instances();
    let layout_after = harness.runtime.current_layout();
    assert_eq!(
        after.asset_browser.selected_folder_id.as_deref(),
        Some("res://scenes")
    );
    assert_eq!(
        after.asset_browser.selected_asset_uuid.as_deref(),
        Some("22222222-2222-2222-2222-222222222222")
    );
    assert_eq!(after.asset_browser.visible_assets.len(), 1);
    assert_eq!(
        after.asset_browser.visible_assets[0].uuid,
        "22222222-2222-2222-2222-222222222222"
    );
    assert_eq!(after.status_line, "Located selected asset");
    assert!(effects.layout_dirty);
    assert!(effects.presentation_dirty);
    assert!(effects.refresh_visible_asset_previews);
    assert!(views_after
        .iter()
        .any(|view| view.descriptor_id.0 == "editor.assets"));
    assert!(views_after != views_before || layout_after != layout_before);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Asset(EditorAssetEvent::LocateSelectedAsset)
    );
}

#[test]
fn locate_selected_asset_without_selection_reports_failure_without_opening_assets() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_retained_asset_locate_empty");
    let bridge = BuiltinAssetSurfaceTemplateBridge::new()
        .expect("builtin asset surface bridge should build");

    let views_before = harness.runtime.current_view_instances();
    let layout_before = harness.runtime.current_layout();
    let effects = dispatch_builtin_asset_surface_control(
        &harness.runtime,
        &bridge,
        "LocateSelectedAsset",
        UiEventKind::Click,
        Vec::new(),
    )
    .expect("locate control should resolve")
    .expect("empty locate action should be handled");

    let snapshot = harness.runtime.editor_snapshot();
    let views_after = harness.runtime.current_view_instances();
    let layout_after = harness.runtime.current_layout();
    let i18n = harness.runtime.context().i18n();
    assert_eq!(i18n.active_locale().as_str(), "en");
    assert_eq!(snapshot.status_line, "Select an asset before locating it");
    assert!(effects.presentation_dirty);
    assert!(!effects.layout_dirty);
    assert_eq!(views_after, views_before);
    assert_eq!(layout_after, layout_before);
    assert_eq!(
        harness.runtime.journal().records().last().unwrap().event,
        EditorEvent::Asset(EditorAssetEvent::LocateSelectedAsset)
    );
    let chinese = EditorLocale::parse("zh-CN").expect("Chinese locale should be supported");
    assert_eq!(
        i18n.translate_for_locale(&chinese, "asset.locate.selection_required")
            .as_ref(),
        "请先选择资产，再进行定位。"
    );
    assert_eq!(
        i18n.translate_for_locale(&chinese, "asset.locate.success")
            .as_ref(),
        "已定位所选资产。"
    );
}

fn shared_catalog() -> Arc<EditorAssetCatalogGeneration> {
    Arc::new(EditorAssetCatalogGeneration::from_snapshot_record(
        EditorAssetCatalogSnapshotRecord {
            project_name: "Sandbox".to_string(),
            project_root: "E:/Sandbox".to_string(),
            assets_root: "E:/Sandbox/assets".to_string(),
            cache_root: "E:/Sandbox/.zircon/cache".to_string(),
            default_scene_uri: "res://scenes/main.scene.toml".to_string(),
            catalog_revision: 1,
            folders: vec![
                EditorAssetFolderRecord {
                    folder_id: "res://".to_string(),
                    parent_folder_id: None,
                    locator_prefix: "res://".to_string(),
                    display_name: "Assets".to_string(),
                    child_folder_ids: vec!["res://scenes".to_string()],
                    direct_asset_uuids: Vec::new(),
                    recursive_asset_count: 1,
                },
                EditorAssetFolderRecord {
                    folder_id: "res://scenes".to_string(),
                    parent_folder_id: Some("res://".to_string()),
                    locator_prefix: "res://scenes".to_string(),
                    display_name: "scenes".to_string(),
                    child_folder_ids: Vec::new(),
                    direct_asset_uuids: vec!["22222222-2222-2222-2222-222222222222".to_string()],
                    recursive_asset_count: 1,
                },
            ],
            assets: vec![EditorAssetCatalogRecord {
                uuid: "22222222-2222-2222-2222-222222222222".to_string(),
                id: "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb".to_string(),
                locator: "res://scenes/main.scene.toml".to_string(),
                kind: ResourceKind::Scene,
                display_name: "main.scene".to_string(),
                file_name: "main.scene.toml".to_string(),
                extension: "toml".to_string(),
                preview_state: PreviewState::Ready,
                meta_path: "E:/Sandbox/assets/scenes/main.scene.toml.zmeta".to_string(),
                preview_artifact_path: "E:/Sandbox/.zircon/cache/editor-previews/main.png"
                    .to_string(),
                source_mtime_unix_ms: 0,
                source_hash: "scene".to_string(),
                dirty: false,
                diagnostics: Vec::new(),
                direct_reference_uuids: Vec::new(),
            }],
        },
        1,
    ))
}
