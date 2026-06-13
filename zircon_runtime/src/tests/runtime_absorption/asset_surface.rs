#[test]
fn asset_module_registration_is_absorbed_into_runtime_asset_surface() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let asset_entry = runtime_root.join("src/asset.rs");
    let asset_mod = runtime_root.join("src/asset/mod.rs");
    let legacy_asset_lib = runtime_root.join("../zircon_asset/src/lib.rs");

    let asset_entry_source = std::fs::read_to_string(&asset_entry).unwrap_or_default();
    let asset_mod_source = std::fs::read_to_string(&asset_mod).unwrap_or_default();

    assert!(
        asset_mod.exists(),
        "expected zircon_runtime/src/asset/mod.rs to own the absorbed asset module registration surface"
    );
    assert!(
        asset_mod_source.contains("AssetModule"),
        "zircon_runtime::asset should define AssetModule after asset module absorption"
    );
    assert!(
        !asset_entry_source.contains("pub use zircon_asset::*"),
        "zircon_runtime/src/asset.rs should stop re-exporting the entire zircon_asset crate after absorption"
    );
    assert!(
        !asset_mod_source.contains("pub use zircon_asset::*"),
        "zircon_runtime/src/asset/mod.rs should stop wildcard-re-exporting zircon_asset"
    );
    assert!(
        !legacy_asset_lib.exists(),
        "standalone zircon_asset crate should be removed after merging into zircon_runtime::asset"
    );
}

#[test]
fn runtime_asset_surface_keeps_project_and_watch_under_namespaces() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let asset_mod_source =
        std::fs::read_to_string(runtime_root.join("src/asset/mod.rs")).unwrap_or_default();
    let legacy_asset_editor_surface = runtime_root.join("src/asset/editor");

    for required in [
        "pub mod artifact;",
        "pub mod assets;",
        "pub mod importer;",
        "pub mod pipeline;",
        "pub mod project;",
        "pub mod watch;",
    ] {
        assert!(
            asset_mod_source.contains(required),
            "zircon_runtime::asset should expose namespace surface `{required}`"
        );
    }

    for forbidden in [
        "pub use zircon_asset::ArtifactStore;",
        "pub use zircon_asset::MaterialAsset;",
        "pub use zircon_asset::ProjectAssetManager;",
        "pub use zircon_asset::EditorAssetManager;",
        "pub use zircon_asset::AssetWorkerPool;",
        "pub use zircon_asset::AssetId;",
        "pub use zircon_asset::AssetKind;",
        "pub use zircon_asset::AssetReference;",
        "pub use zircon_asset::AssetUri;",
        "pub use zircon_asset::AssetUuid;",
        "pub use zircon_asset::project::{",
        "pub use zircon_asset::watch::{AssetChange, AssetChangeKind, AssetWatchEvent, AssetWatcher};",
        "pub use zircon_asset::{",
    ] {
        assert!(
            !asset_mod_source.contains(forbidden),
            "zircon_runtime::asset should stop flattening namespace-owned surface `{forbidden}`"
        );
    }

    assert!(
        !asset_mod_source.contains("pub mod editor;"),
        "zircon_runtime::asset should not keep the absorbed editor asset surface at the runtime root"
    );
    assert!(
        !legacy_asset_editor_surface.exists(),
        "runtime asset namespace should delete the legacy editor surface after zircon_editor absorbs it"
    );
}

#[test]
fn runtime_04_asset_facade_query_surface_stays_manager_owned_and_server_free() {
    let runtime_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let facade_doc =
        std::fs::read_to_string(runtime_root.join("../docs/zircon_runtime/asset/facade.md"))
            .unwrap_or_default();
    let runtime_04_plan = std::fs::read_to_string(
        runtime_root.join("../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"),
    )
    .unwrap_or_default();
    let runtime_index =
        std::fs::read_to_string(runtime_root.join("../docs/plans/zircon_runtime/runtime/index.md"))
            .unwrap_or_default();
    let facade_manager_source =
        std::fs::read_to_string(runtime_root.join("src/asset/facade/manager.rs"))
            .unwrap_or_default();
    let readiness_source =
        std::fs::read_to_string(runtime_root.join("src/asset/facade/readiness.rs"))
            .unwrap_or_default();
    let assets_source = std::fs::read_to_string(runtime_root.join("src/asset/facade/assets.rs"))
        .unwrap_or_default();
    let service_contract_source = std::fs::read_to_string(
        runtime_root.join("src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs"),
    )
    .unwrap_or_default();

    for required_manager_query in [
        "pub fn load_state<TAsset: Asset>(&self, handle: Handle<TAsset>) -> AssetLoadState",
        "pub fn failure_reason<TAsset: Asset>(&self, handle: Handle<TAsset>) -> Option<String>",
        "pub fn dependency_load_state<TAsset: Asset>(",
        "pub fn load_states<TAsset: Asset>(&self, handle: Handle<TAsset>) -> AssetLoadStates",
        "pub fn readiness_report<TAsset: Asset>(&self, handle: Handle<TAsset>) -> AssetReadinessReport",
        "pub fn is_loaded_with_dependencies<TAsset: Asset>(&self, handle: Handle<TAsset>) -> bool",
        "pub fn asset_load_state_by_id<TAsset: Asset>(&self, id: AssetId) -> AssetLoadState",
        "pub fn subscribe_asset_events<TAsset: Asset>(&self) -> AssetEventReceiver<TAsset>",
    ] {
        let source = if required_manager_query.contains("readiness_report") {
            &readiness_source
        } else {
            &facade_manager_source
        };
        assert!(
            source.contains(required_manager_query),
            "Runtime 04 facade query surface is missing `{required_manager_query}`"
        );
    }

    for required_assets_query in [
        "pub fn get(&self, handle: Handle<TAsset>) -> Option<Arc<TAsset>>",
        "pub fn acquire(&self, handle: Handle<TAsset>) -> Option<ResourceLease<TAsset>>",
        "pub fn contains(&self, handle: Handle<TAsset>) -> bool",
        "pub fn load_state(&self, handle: Handle<TAsset>) -> AssetLoadState",
        "pub fn failure_reason(&self, handle: Handle<TAsset>) -> Option<String>",
        "pub fn subscribe_events(&self) -> AssetEventReceiver<TAsset>",
    ] {
        assert!(
            assets_source.contains(required_assets_query),
            "Runtime 04 typed Assets<TAsset> query surface is missing `{required_assets_query}`"
        );
    }

    for required_service_query in [
        "fn pipeline_info(&self) -> AssetPipelineInfo",
        "fn asset_importer_capability_reports(&self) -> Vec<AssetImporterCapabilityReport>",
        "fn asset_importer_capability_report_for_source(",
        "fn current_project(&self) -> Option<ProjectInfo>",
        "fn asset_status(&self, uri: &str) -> Option<AssetStatusRecord>",
        "fn list_assets(&self) -> Vec<AssetStatusRecord>",
        "fn subscribe_asset_changes(&self) -> ChannelReceiver<AssetChange>",
        "fn subscribe_asset_watch_errors(&self) -> ChannelReceiver<AssetWatchError>",
    ] {
        assert!(
            service_contract_source.contains(required_service_query),
            "Runtime 04 AssetManager service query surface is missing `{required_service_query}`"
        );
    }

    for required_doc_anchor in [
        "Loading and status queries are split across manager/facade/service surfaces.",
        "Keep `manager` / `facade` naming.",
        "ProjectAssetManager::readiness_report<TAsset>(handle)` is read-only.",
        "All facade state queries are read-only.",
        "The public `AssetManager` service trait forwards the importer capability report helpers",
        "runtime_04_asset_facade_query_surface_stays_manager_owned_and_server_free",
        "Runtime 04 decision",
        "Keep the split. Concrete format parsing stays under importer/load owners",
        "Do not reopen a second processor design here.",
    ] {
        assert!(
            facade_doc.contains(required_doc_anchor)
                || runtime_04_plan.contains(required_doc_anchor)
                || runtime_index.contains(required_doc_anchor),
            "Runtime 04 docs must record asset facade query anchor `{required_doc_anchor}`"
        );
    }

    const STALE_PENDING_QUERY_SURFACE: &str =
        "\u{5f85}\u{8865}\u{67e5}\u{8be2}\u{9762}\u{76d8}\u{70b9}";
    const STALE_PENDING_COMPARISON: &str = "\u{5f85}\u{5bf9}\u{7167}";
    const STALE_PENDING_DECISION: &str = "\u{5f85}\u{88c1}\u{51b3}";

    for stale_plan_anchor in [
        STALE_PENDING_QUERY_SURFACE,
        STALE_PENDING_COMPARISON,
        STALE_PENDING_DECISION,
        "asset server vocabulary",
    ] {
        assert!(
            !runtime_04_plan.contains(stale_plan_anchor),
            "Runtime 04 plan still contains stale asset query/server anchor `{stale_plan_anchor}`"
        );
    }

    for forbidden_source_anchor in [
        "pub struct AssetServer",
        "pub trait AssetServer",
        "pub mod server",
        "asset_server",
        "AssetServer::",
    ] {
        assert!(
            !facade_manager_source.contains(forbidden_source_anchor)
                && !readiness_source.contains(forbidden_source_anchor)
                && !assets_source.contains(forbidden_source_anchor)
                && !service_contract_source.contains(forbidden_source_anchor),
            "Runtime 04 asset query surface must not introduce `{forbidden_source_anchor}`"
        );
    }
}
