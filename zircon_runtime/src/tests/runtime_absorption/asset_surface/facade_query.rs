use super::support::{read_runtime_file, read_workspace_file};

#[test]
fn runtime_04_asset_facade_query_surface_stays_manager_owned_and_server_free() {
    let facade_doc = read_workspace_file("docs/zircon_runtime/asset/facade.md");
    let runtime_04_plan =
        read_workspace_file("docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md");
    let runtime_index = read_workspace_file("docs/plans/zircon_runtime/runtime/index.md");
    let facade_manager_source = read_runtime_file("src/asset/facade/manager.rs");
    let readiness_source = read_runtime_file("src/asset/facade/readiness.rs");
    let assets_source = read_runtime_file("src/asset/facade/assets.rs");
    let event_source = read_runtime_file("src/asset/facade/event.rs");
    let service_contract_source =
        read_runtime_file("src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs");

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

    for forbidden_event_bridge_anchor in [
        "spawn_named_thread",
        "asset-event-filter-",
        "let (sender, receiver) = unbounded()",
        "let (shutdown_sender, shutdown_receiver) = bounded::<()>(0)",
    ] {
        assert!(
            !event_source.contains(forbidden_event_bridge_anchor),
            "typed asset event subscription must not create a dedicated thread or second unbounded queue: `{forbidden_event_bridge_anchor}`"
        );
    }
    assert!(
        event_source.contains("receiver: ResourceEventReceiver"),
        "typed asset event receiver must filter the shared bounded resource log lazily"
    );
    assert!(!event_source.contains("ChannelReceiver<ResourceEvent>"));
}
