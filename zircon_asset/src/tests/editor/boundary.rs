#[test]
fn asset_manager_api_boundary_lives_in_zircon_asset() {
    let lib_source = include_str!("../../lib.rs");
    let pipeline_manager_mod_source = include_str!("../../pipeline/manager.rs");
    let asset_manager_mod_source = include_str!("../../pipeline/manager/asset_manager/mod.rs");
    let asset_manager_trait_source =
        include_str!("../../pipeline/manager/asset_manager/asset_manager.rs");
    let pipeline_resolver_source =
        include_str!("../../pipeline/manager/asset_manager/resolve_asset_manager.rs");
    let module_descriptor_source =
        include_str!("../../pipeline/manager/registration/module_descriptor.rs");
    let service_names_source = include_str!("../../pipeline/manager/registration/service_names.rs");
    let manager_lib_source = include_str!("../../../../zircon_manager/src/lib.rs");
    let manager_resolver_source = include_str!("../../../../zircon_manager/src/resolver.rs");

    for required in [
        "mod asset_manager;",
        "mod records;",
        "mod registration;",
        "pub use asset_manager::{resolve_asset_manager, AssetManager, AssetManagerHandle};",
        "pub use records::{AssetPipelineInfo, AssetStatusRecord, ProjectInfo};",
    ] {
        assert!(
            pipeline_manager_mod_source.contains(required),
            "zircon_asset::pipeline::manager should own `{required}`"
        );
    }

    for required in [
        "mod asset_manager;",
        "mod asset_manager_handle;",
        "mod resolve_asset_manager;",
        "pub use asset_manager::AssetManager;",
        "pub use asset_manager_handle::AssetManagerHandle;",
        "pub use resolve_asset_manager::resolve_asset_manager;",
    ] {
        assert!(
            asset_manager_mod_source.contains(required),
            "zircon_asset::pipeline::manager::asset_manager should own `{required}`"
        );
    }

    for required in [
        "AssetManager",
        "AssetManagerHandle",
        "resolve_asset_manager",
        "AssetPipelineInfo",
        "AssetStatusRecord",
        "ProjectInfo",
        "ASSET_MANAGER_NAME",
    ] {
        assert!(
            lib_source.contains(required),
            "zircon_asset root should publicly export `{required}`"
        );
    }

    assert!(
        asset_manager_trait_source
            .contains("fn subscribe_asset_changes(&self) -> ChannelReceiver<AssetChange>;"),
        "asset manager trait should expose asset-owned AssetChange stream"
    );
    assert!(
        pipeline_resolver_source
            .contains("core.resolve_manager::<AssetManagerHandle>(ASSET_MANAGER_NAME)"),
        "asset manager resolver should resolve the asset-owned handle"
    );
    assert!(
        module_descriptor_source.contains("AssetManagerHandle::new"),
        "asset module descriptor should register the public asset manager with the asset-owned handle"
    );
    assert!(
        !module_descriptor_source
            .contains("use zircon_manager::{AssetManagerHandle, ResourceManagerHandle};"),
        "asset module descriptor should not import AssetManagerHandle from zircon_manager"
    );
    assert!(
        !service_names_source.contains("zircon_manager::ASSET_MANAGER_NAME"),
        "asset service names should not source ASSET_MANAGER_NAME from zircon_manager"
    );

    for source in [manager_lib_source, manager_resolver_source] {
        assert!(
            !source.contains("AssetManager"),
            "zircon_manager should not expose generic asset manager API after asset boundary cleanup"
        );
    }
}

#[test]
fn editor_asset_api_boundary_lives_in_zircon_asset() {
    let lib_source = include_str!("../../lib.rs");
    let editor_mod_source = include_str!("../../editor/mod.rs");
    let module_descriptor_source =
        include_str!("../../pipeline/manager/registration/module_descriptor.rs");
    let service_names_source = include_str!("../../pipeline/manager/registration/service_names.rs");

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
    let editor_manager_source =
        include_str!("../../editor/manager/default_editor_asset_manager.rs");
    let pipeline_asset_status_record_source =
        include_str!("../../pipeline/manager/records/asset_status_record.rs");

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
        !editor_manager_source
            .contains("use zircon_manager::{AssetRecordKind, PreviewStateRecord};"),
        "editor asset manager should not import duplicated taxonomy from zircon_manager"
    );
    assert!(
        !pipeline_asset_status_record_source.contains("AssetRecordKind"),
        "pipeline manager records should use canonical ResourceKind instead of AssetRecordKind"
    );
}

#[test]
fn resource_state_protocol_lives_in_resource_crate() {
    let manager_resource_records_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../zircon_manager/src/records/resource.rs");
    let pipeline_status_record_source =
        include_str!("../../pipeline/manager/records/status_record.rs");
    let pipeline_metadata_import_state_source =
        include_str!("../../pipeline/manager/records/metadata_import_state.rs");
    let manager_lib_source = include_str!("../../../../zircon_manager/src/lib.rs");

    assert!(
        pipeline_metadata_import_state_source.contains("use zircon_resource::ResourceState;"),
        "pipeline manager record helpers should import zircon_resource::ResourceState"
    );
    for source in [
        pipeline_status_record_source,
        pipeline_metadata_import_state_source,
    ] {
        assert!(
            !source.contains("ResourceStateRecord"),
            "pipeline manager records should not depend on zircon_manager::ResourceStateRecord"
        );
    }
    assert!(
        !manager_lib_source.contains("ResourceStateRecord"),
        "zircon_manager lib.rs should not re-export ResourceStateRecord after canonical ResourceState migration"
    );
    assert!(
        !manager_resource_records_path.exists(),
        "zircon_manager should delete src/records/resource.rs after canonical ResourceState migration"
    );
}

#[test]
fn resource_status_protocol_lives_in_resource_crate() {
    let manager_resource_records_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../zircon_manager/src/records/resource.rs");
    let pipeline_asset_status_record_source =
        include_str!("../../pipeline/manager/records/asset_status_record.rs");
    let resource_manager_facade_source =
        include_str!("../../pipeline/manager/facades/resource_manager_facade.rs");
    let manager_lib_source = include_str!("../../../../zircon_manager/src/lib.rs");
    let manager_resolver_source = include_str!("../../../../zircon_manager/src/resolver.rs");

    for source in [
        pipeline_asset_status_record_source,
        resource_manager_facade_source,
        manager_lib_source,
        manager_resolver_source,
    ] {
        assert!(
            !source.contains("ResourceStatusRecord"),
            "resource status boundary should not depend on zircon_manager::ResourceStatusRecord after canonical ResourceRecord migration"
        );
    }

    assert!(
        manager_resolver_source.contains("ResourceManager"),
        "zircon_manager resolver should continue exposing the resource manager handle"
    );
    assert!(
        !manager_resource_records_path.exists(),
        "zircon_manager should delete src/records/resource.rs after canonical ResourceRecord migration"
    );
}

#[test]
fn resource_change_protocol_lives_in_resource_crate() {
    let manager_resource_records_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../zircon_manager/src/records/resource.rs");
    let resource_manager_facade_source =
        include_str!("../../pipeline/manager/facades/resource_manager_facade.rs");
    let manager_lib_source = include_str!("../../../../zircon_manager/src/lib.rs");
    let manager_resolver_source = include_str!("../../../../zircon_manager/src/resolver.rs");

    for source in [resource_manager_facade_source, manager_lib_source, manager_resolver_source] {
        assert!(
            !source.contains("ResourceChangeRecord"),
            "resource change boundary should not depend on zircon_manager::ResourceChangeRecord after canonical ResourceEvent migration"
        );
        assert!(
            !source.contains("ResourceChangeKind"),
            "resource change boundary should not depend on zircon_manager::ResourceChangeKind after canonical ResourceEvent migration"
        );
    }

    assert!(
        manager_resolver_source.contains("ResourceManager"),
        "zircon_manager resolver should continue exposing the resource manager handle"
    );
    assert!(
        !manager_resource_records_path.exists(),
        "zircon_manager should delete src/records/resource.rs after canonical ResourceEvent migration"
    );
}

#[test]
fn asset_root_keeps_only_asset_named_resource_aliases() {
    let lib_source = include_str!("../../lib.rs");

    assert!(
        !lib_source.contains("pub use zircon_resource::{"),
        "zircon_asset root should not directly re-export zircon_resource raw surface"
    );

    for required in [
        "pub type AssetId = zircon_resource::ResourceId;",
        "pub type AssetKind = zircon_resource::ResourceKind;",
        "pub type AssetMetadata = zircon_resource::ResourceRecord;",
        "pub type AssetRegistry = zircon_resource::ResourceRegistry;",
        "pub type AssetReference = zircon_resource::AssetReference;",
        "pub type AssetUuid = zircon_resource::AssetUuid;",
        "pub type AssetUri = zircon_resource::ResourceLocator;",
        "pub type AssetUriError = zircon_resource::ResourceLocatorError;",
        "pub type AssetUriScheme = zircon_resource::ResourceScheme;",
    ] {
        assert!(
            lib_source.contains(required),
            "zircon_asset root should preserve asset semantic alias `{required}`"
        );
    }

    for forbidden in [
        "pub use zircon_resource::ResourceManager;",
        "pub use zircon_resource::ResourceRecord;",
        "pub use zircon_resource::ResourceState;",
        "pub use zircon_resource::ResourceKind;",
        "pub use zircon_resource::ResourceLocator;",
        "pub use zircon_resource::ResourceHandle;",
        "pub use zircon_resource::ResourceEvent;",
        "pub use zircon_resource::ModelMarker;",
    ] {
        assert!(
            !lib_source.contains(forbidden),
            "zircon_asset root should not publicly re-export raw zircon_resource type `{forbidden}`"
        );
    }
}
