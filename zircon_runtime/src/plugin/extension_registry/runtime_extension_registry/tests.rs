use super::*;
use crate::asset::{AssetImporterDescriptor, AssetKind};

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
