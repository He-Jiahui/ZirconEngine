#[test]
fn editor_asset_api_boundary_lives_in_zircon_asset() {
    let lib_source = include_str!("../../lib.rs");
    let editor_mod_source = include_str!("../../editor/mod.rs");
    let module_descriptor_source = include_str!("../../pipeline/manager/module_descriptor.rs");
    let service_names_source = include_str!("../../pipeline/manager/service_names.rs");

    for required in [
        "mod api;",
        "mod records;",
        "mod resolver;",
        "pub use api::EditorAssetManager;",
        "pub use records::{",
        "pub use resolver::{",
        "EditorAssetManagerHandle",
        "resolve_editor_asset_manager",
    ] {
        assert!(
            editor_mod_source.contains(required),
            "zircon_asset::editor should own `{required}`"
        );
    }

    for required in [
        "EditorAssetCatalogRecord",
        "EditorAssetCatalogSnapshotRecord",
        "EditorAssetChangeKind",
        "EditorAssetChangeRecord",
        "EditorAssetDetailsRecord",
        "EditorAssetFolderRecord",
        "EditorAssetManager",
        "EditorAssetManagerHandle",
        "resolve_editor_asset_manager",
        "EDITOR_ASSET_MANAGER_NAME",
    ] {
        assert!(
            lib_source.contains(required),
            "zircon_asset root should publicly export `{required}`"
        );
    }

    assert!(
        module_descriptor_source.contains("EditorAssetManagerHandle::new"),
        "asset module descriptor should register the public editor asset manager with the asset-owned handle"
    );
    assert!(
        !module_descriptor_source.contains(
            "use zircon_manager::{AssetManagerHandle, EditorAssetManagerHandle, ResourceManagerHandle};"
        ),
        "asset module descriptor should not import EditorAssetManagerHandle from zircon_manager"
    );
    assert!(
        !service_names_source.contains("zircon_manager::EDITOR_ASSET_MANAGER_NAME"),
        "asset service names should not source EDITOR_ASSET_MANAGER_NAME from zircon_manager"
    );
}

#[test]
fn asset_kind_and_preview_taxonomy_live_in_resource_and_asset_crates() {
    let editor_records_source = include_str!("../../editor/records.rs");
    let editor_manager_source = include_str!("../../editor/manager/default_editor_asset_manager.rs");
    let pipeline_records_source = include_str!("../../pipeline/manager/records.rs");

    assert!(
        editor_records_source.contains("pub kind: ResourceKind"),
        "editor asset catalog records should use zircon_resource::ResourceKind"
    );
    assert!(
        editor_records_source.contains("pub preview_state: PreviewState"),
        "editor asset catalog records should use zircon_asset::PreviewState"
    );
    assert!(
        editor_records_source.contains("pub kind: Option<ResourceKind>"),
        "editor asset reference records should use zircon_resource::ResourceKind"
    );
    assert!(
        !editor_records_source.contains("AssetRecordKind"),
        "editor asset records should not depend on zircon_manager::AssetRecordKind"
    );
    assert!(
        !editor_records_source.contains("PreviewStateRecord"),
        "editor asset records should not depend on zircon_manager::PreviewStateRecord"
    );
    assert!(
        !editor_manager_source.contains("use zircon_manager::{AssetRecordKind, PreviewStateRecord};"),
        "editor asset manager should not import duplicated taxonomy from zircon_manager"
    );
    assert!(
        !pipeline_records_source.contains("AssetRecordKind"),
        "pipeline manager records should use canonical ResourceKind instead of AssetRecordKind"
    );
}

#[test]
fn resource_state_protocol_lives_in_resource_crate() {
    let pipeline_records_source = include_str!("../../pipeline/manager/records.rs");
    let manager_lib_source = include_str!("../../../../zircon_manager/src/lib.rs");
    let manager_resource_records_source =
        include_str!("../../../../zircon_manager/src/records/resource.rs");

    assert!(
        pipeline_records_source.contains("use zircon_resource::ResourceState;"),
        "pipeline manager records should import zircon_resource::ResourceState"
    );
    assert!(
        !pipeline_records_source.contains("ResourceStateRecord"),
        "pipeline manager records should not depend on zircon_manager::ResourceStateRecord"
    );
    assert!(
        !manager_lib_source.contains("ResourceStateRecord"),
        "zircon_manager lib.rs should not re-export ResourceStateRecord after canonical ResourceState migration"
    );
    assert!(
        !manager_resource_records_source.contains("pub enum ResourceStateRecord"),
        "zircon_manager resource records should not define ResourceStateRecord after canonical ResourceState migration"
    );
}
