use super::*;
use crate::asset::{AssetImporterDescriptor, AssetKind};
use crate::core::framework::bridge::{BridgeInterfaceStatus, PluginInterface};
use std::sync::Arc;

trait StableFinalizeBridge: Send + Sync {}

impl PluginInterface for dyn StableFinalizeBridge {
    const INTERFACE_ID: &'static str = "test.stable-finalize.bridge.v1";
}

struct StableFinalizeBridgeProvider;

impl StableFinalizeBridge for StableFinalizeBridgeProvider {}

#[test]
fn asset_importer_revocation_directly_invalidates_its_finalized_state() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("weather.runtime")
        .expect("plugin module owner");
    registry
        .register_asset_importer_descriptor(
            AssetImporterDescriptor::new("weather.image", "weather", AssetKind::Data, 1)
                .with_source_extensions(["weather"]),
        )
        .expect("asset importer descriptor");
    registry.finalize();
    assert!(registry.asset_importers_finalized);

    let removed = registry.revoke_owner_registrations(owner);

    assert_eq!(removed.asset_importers.len(), 1);
    assert!(!registry.asset_importers_finalized);
}

#[test]
fn repeated_finalize_reuses_the_bridge_table_when_registrations_are_unchanged() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("stable_finalize.runtime")
        .expect("plugin module owner");
    let provider: Arc<dyn StableFinalizeBridge> = Arc::new(StableFinalizeBridgeProvider);
    registry
        .export_interface::<dyn StableFinalizeBridge>(owner, provider)
        .expect("bridge export");
    registry.finalize();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn StableFinalizeBridge as PluginInterface>::INTERFACE_ID)
        .expect("stable bridge slot");
    table
        .set_enabled(slot, false)
        .expect("test should disable the stable bridge");

    registry.finalize();

    assert_eq!(
        registry
            .frozen_bridge_table()
            .interface_status(<dyn StableFinalizeBridge as PluginInterface>::INTERFACE_ID),
        BridgeInterfaceStatus::Disabled
    );
}

#[test]
fn namespace_validation_does_not_collect_split_segments() {
    for source in [
        include_str!("../validation/plugin_event_catalog.rs"),
        include_str!("../validation/plugin_option.rs"),
    ] {
        assert!(
            !source.contains("let segments: Vec<_> = value.split('.').collect();"),
            "namespace validation should stream split segments without allocating a temporary Vec"
        );
    }
}
