use crate::asset::{AssetImporterDescriptor, AssetKind};
use crate::builtin::RuntimePluginId;
use crate::graphics::RenderFeatureDescriptor;
use crate::plugin::{
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginCatalog,
    RuntimePluginDescriptor,
};
use crate::scene::World;

#[test]
fn runtime_extension_catalog_finalizes_dense_tables_before_apply() {
    let plugin = FrozenCatalogPlugin {
        descriptor: RuntimePluginDescriptor::builder(
            "weather",
            "Weather",
            RuntimePluginId::Particles,
            "zircon_plugin_weather_runtime",
        )
        .build(),
    };
    let catalog = RuntimePluginCatalog::from_plugins([&plugin as &dyn RuntimePlugin]);
    let mut report = catalog.runtime_extensions();

    assert!(report.registry.is_finalized());
    assert_eq!(report.registry.render_features().len(), 1);
    assert_eq!(report.registry.render_features()[0].name, "weather.clouds");

    report
        .registry
        .register_render_feature(render_feature("weather.rain"))
        .expect("post-freeze staging registration");
    assert!(!report.registry.is_finalized());

    report.registry.finalize();
    assert!(report.registry.is_finalized());
    assert_eq!(report.registry.render_features().len(), 2);
    assert_eq!(report.registry.render_features()[1].name, "weather.rain");
}

#[test]
fn runtime_extension_apply_finalizes_dense_tables() {
    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_render_feature(render_feature("weather.clouds"))
        .expect("staging extension");
    assert!(!registry.is_finalized());

    registry
        .apply_to_world(&mut World::empty())
        .expect("apply frozen extension registry");

    assert!(registry.is_finalized());
}

#[test]
fn asset_importer_registration_invalidates_finalized_registry_epoch() {
    let mut registry = RuntimeExtensionRegistry::default();
    registry.finalize();
    assert!(registry.is_finalized());

    registry
        .register_asset_importer_descriptor(
            AssetImporterDescriptor::new("weather.data", "weather", AssetKind::Data, 1)
                .with_source_extensions(["weather"]),
        )
        .expect("asset importer descriptor");

    assert!(!registry.is_finalized());
    registry.finalize();
    assert!(registry.is_finalized());
}

#[test]
fn asset_importer_revocation_invalidates_finalized_registry_epoch() {
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

    let removed = registry.revoke_owner_registrations(owner);

    assert_eq!(removed.asset_importers.len(), 1);
    assert!(!registry.is_finalized());
}

#[test]
fn owner_unload_revokes_all_slots() {
    let mut registry = RuntimeExtensionRegistry::default();
    let weather = registry
        .intern_plugin_module("weather.runtime")
        .expect("weather owner");
    let storm = registry
        .intern_plugin_module("storm.runtime")
        .expect("storm owner");
    registry
        .register_render_feature(render_feature("weather.clouds"))
        .expect("weather extension");
    registry
        .register_render_feature(render_feature("weather.rain"))
        .expect("second weather extension");
    registry
        .register_render_feature(render_feature("storm.lightning"))
        .expect("storm extension");
    registry.finalize();

    let removed = registry.revoke_owner_registrations(weather);

    assert_eq!(removed.render_features.len(), 2);
    assert!(!registry.is_finalized());
    assert_eq!(registry.ownership_for(storm).render_features.len(), 1);
    assert_eq!(registry.render_features().len(), 1);
    assert_eq!(registry.render_features()[0].name, "storm.lightning");

    registry.finalize();
    assert!(registry.is_finalized());
}

fn render_feature(name: &str) -> RenderFeatureDescriptor {
    RenderFeatureDescriptor {
        name: name.to_string(),
        required_extract_sections: Vec::new(),
        capability_requirements: Vec::new(),
        history_bindings: Vec::new(),
        stage_passes: Vec::new(),
    }
}

#[derive(Debug)]
struct FrozenCatalogPlugin {
    descriptor: RuntimePluginDescriptor,
}

impl RuntimePlugin for FrozenCatalogPlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        registry.register_render_feature(render_feature("weather.clouds"))
    }
}
