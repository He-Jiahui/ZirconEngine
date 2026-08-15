use super::super::*;
use crate::core::asset::{
    AssetToolkitDescriptor, AssetToolkitOpenRoute, AssetTypeContribution, AssetTypeId,
};
use crate::core::editor_extension::{EditorExtensionRegistry, ViewDescriptor};
use crate::core::editor_operation::EditorOperationPath;
use crate::ui::host::editor_asset_manager::{
    EditorAssetCatalogGeneration, EditorAssetCatalogRecord, EditorAssetCatalogSnapshotRecord,
    EditorAssetFolderRecord,
};
use std::fs;
use std::sync::Arc;
use zircon_runtime::asset::project::PreviewState;
use zircon_runtime_interface::resource::ResourceKind;

#[test]
fn asset_open_event_opens_the_indexed_registry_toolkit() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_indexed_asset_open");
    let asset_locator = "res://ui/runtime_ui_asset.zui";
    let asset_type = AssetTypeId::from_resource_kind(ResourceKind::UiLayout);
    let open_operation =
        EditorOperationPath::parse("view.editor.ui_asset.integration.open").unwrap();
    let mut extension = EditorExtensionRegistry::default();
    extension
        .register_view(ViewDescriptor::new(
            "editor.ui_asset.integration",
            "UI Asset Integration",
            "Assets",
        ))
        .unwrap();
    extension
        .register_asset_type_contribution(AssetTypeContribution::augment(asset_type).with_toolkit(
            AssetToolkitDescriptor::new("editor.ui_asset.integration", open_operation),
        ))
        .unwrap();
    runtime
        .runtime
        .register_editor_extension(extension.into_contribution_batch().unwrap())
        .unwrap();
    runtime.runtime.sync_asset_catalog(Arc::new(
        EditorAssetCatalogGeneration::from_snapshot_record(
            EditorAssetCatalogSnapshotRecord {
                project_name: "Indexed Asset Open".to_string(),
                project_root: "E:/IndexedAssetOpen".to_string(),
                assets_root: "E:/IndexedAssetOpen/assets".to_string(),
                cache_root: "E:/IndexedAssetOpen/.zircon/cache".to_string(),
                default_scene_uri: String::new(),
                catalog_revision: 1,
                folders: vec![EditorAssetFolderRecord {
                    folder_id: "res://".to_string(),
                    parent_folder_id: None,
                    locator_prefix: "res://".to_string(),
                    display_name: "Assets".to_string(),
                    child_folder_ids: Vec::new(),
                    direct_asset_uuids: vec!["11111111-1111-1111-1111-111111111111".to_string()],
                    recursive_asset_count: 1,
                }],
                assets: vec![EditorAssetCatalogRecord {
                    uuid: "11111111-1111-1111-1111-111111111111".to_string(),
                    id: "22222222-2222-2222-2222-222222222222".to_string(),
                    locator: asset_locator.to_string(),
                    kind: ResourceKind::UiLayout,
                    display_name: "runtime_ui_asset.zui".to_string(),
                    file_name: "runtime_ui_asset.zui".to_string(),
                    extension: "zui".to_string(),
                    preview_state: PreviewState::Dirty,
                    meta_path: "E:/IndexedAssetOpen/assets/ui/runtime_ui_asset.zui.zmeta"
                        .to_string(),
                    preview_artifact_path: String::new(),
                    source_mtime_unix_ms: 0,
                    source_hash: String::new(),
                    dirty: false,
                    diagnostics: Vec::new(),
                    direct_reference_uuids: Vec::new(),
                }],
            },
            1,
        ),
    ));

    let record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: asset_locator.to_string(),
            }),
        )
        .unwrap();

    assert_eq!(
        record.event,
        EditorEvent::Asset(EditorAssetEvent::OpenAsset {
            asset_locator: asset_locator.to_string(),
        })
    );
    assert!(record.effects.contains(&EditorEventEffect::LayoutChanged));
    let toolkit_view = runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .find(|instance| {
            instance.descriptor_id == ViewDescriptorId::new("editor.ui_asset.integration")
        })
        .expect("indexed asset toolkit view should open");
    let route: AssetToolkitOpenRoute =
        serde_json::from_value(toolkit_view.serializable_payload).unwrap();
    assert_eq!(route.asset_locator().to_string(), asset_locator);
    assert_eq!(
        route.open_operation(),
        &EditorOperationPath::parse("view.editor.ui_asset.integration.open").unwrap()
    );
}

#[test]
fn asset_open_event_does_not_infer_a_toolkit_from_the_file_suffix() {
    let _guard = env_lock().lock().unwrap();

    let runtime = EventRuntimeHarness::new("zircon_editor_event_suffix_only_asset_open");
    let ui_asset_path = std::env::temp_dir().join("zircon_editor_event_suffix_only_asset_open.zui");
    fs::write(
        &ui_asset_path,
        r#"
[asset]
kind = "view"
id = "editor.tests.non_zui_runtime_ui_asset"
version = 1
display_name = "Non-ZUI Runtime UI Asset"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "Label"
props = { text = "Non-ZUI" }
"#,
    )
    .unwrap();

    let record = runtime
        .runtime
        .dispatch_event(
            EditorEventSource::Headless,
            EditorEvent::Asset(EditorAssetEvent::OpenAsset {
                asset_locator: ui_asset_path.to_string_lossy().into_owned(),
            }),
        )
        .expect("suffix-only asset event should be rejected by the indexed registry boundary");

    assert_eq!(
        record.event,
        EditorEvent::Asset(EditorAssetEvent::OpenAsset {
            asset_locator: ui_asset_path.to_string_lossy().into_owned(),
        })
    );
    assert!(!record.effects.contains(&EditorEventEffect::LayoutChanged));
    assert!(!runtime
        .runtime
        .current_view_instances()
        .into_iter()
        .any(|instance| instance.descriptor_id == ViewDescriptorId::new("editor.ui_asset")));
    assert_eq!(
        runtime.runtime.editor_snapshot().status_line,
        format!(
            "Invalid asset locator {}: resource locator is missing scheme: {}",
            ui_asset_path.to_string_lossy(),
            ui_asset_path.to_string_lossy(),
        )
    );

    let _ = fs::remove_file(ui_asset_path);
}
