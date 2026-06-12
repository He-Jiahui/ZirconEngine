use crate::asset::AssetImporterDescriptor;
use crate::plugin::{
    ComponentTypeDescriptor, PluginEventCatalogManifest, PluginEventManifest, PluginOptionManifest,
    PluginPackageManifest, RuntimeExtensionRegistry, RuntimePlugin, RuntimePluginCatalog,
    RuntimePluginDescriptor, RuntimePluginRegistrationReport, UiComponentDescriptor,
};
use crate::scene::ecs::{Resource, SystemStage};
use crate::{RuntimePluginId, RuntimeTargetMode};

#[test]
fn runtime_plugin_registration_collects_package_manifest_declared_runtime_contributions() {
    let plugin = ManifestDeclaredRuntimePlugin {
        descriptor: RuntimePluginDescriptor::new(
            "weather",
            "Weather",
            RuntimePluginId::Particles,
            "zircon_plugin_weather_runtime",
        )
        .with_target_modes([RuntimeTargetMode::ClientRuntime])
        .with_capability("runtime.plugin.weather"),
    };
    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);

    assert!(registration.is_success(), "{:?}", registration.diagnostics);
    assert_eq!(registration.extensions.plugin_options().len(), 1);
    assert_eq!(
        registration.extensions.plugin_options()[0].key,
        "weather.precipitation"
    );
    assert_eq!(registration.extensions.plugin_event_catalogs().len(), 1);
    assert_eq!(
        registration.extensions.plugin_event_catalogs()[0].namespace,
        "weather.events"
    );
    assert_eq!(registration.extensions.components().len(), 1);
    assert_eq!(
        registration.extensions.components()[0].type_id,
        "weather.Component.CloudLayer"
    );
    assert_eq!(registration.extensions.ui_components().len(), 1);
    assert_eq!(
        registration.extensions.ui_components()[0].component_id,
        "weather.Ui.CloudLayerInspector"
    );
    assert_eq!(
        registration.extensions.asset_importers().descriptors()[0].id,
        "weather.data"
    );

    let catalog = RuntimePluginCatalog::from_registration_reports([registration], []);
    let report = catalog.runtime_extensions();

    assert!(report.is_success(), "{:?}", report.diagnostics);
    assert_eq!(report.registry.plugin_options().len(), 1);
    assert_eq!(report.registry.plugin_event_catalogs().len(), 1);
    assert_eq!(report.registry.components().len(), 1);
    assert_eq!(report.registry.ui_components().len(), 1);
    assert_eq!(report.registry.asset_importers().descriptors().len(), 1);
}

#[test]
fn runtime_extension_registry_tracks_manifest_contribution_owners() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("weather.runtime")
        .expect("plugin module owner");

    registry
        .register_component(ComponentTypeDescriptor::new(
            "weather.Component.CloudLayer",
            "weather",
            "Cloud Layer",
        ))
        .unwrap();
    registry
        .register_ui_component(UiComponentDescriptor::new(
            "weather.Ui.CloudLayerInspector",
            "weather",
            "asset://weather/editor/cloud_layer_inspector.zui",
        ))
        .unwrap();
    registry
        .register_plugin_option(PluginOptionManifest::new(
            "weather.precipitation",
            "Precipitation",
            "bool",
            "true",
        ))
        .unwrap();
    registry
        .register_plugin_event_catalog(PluginEventCatalogManifest {
            namespace: "weather.events".to_string(),
            version: 1,
            events: vec![PluginEventManifest {
                id: "weather.events.storm_front_arrived".to_string(),
                display_name: "Storm Front Arrived".to_string(),
                payload_schema: "weather.schemas.storm_front_payload.v1".to_string(),
            }],
        })
        .unwrap();

    let ownership = registry.ownership_for(owner);
    assert_eq!(ownership.components.len(), 1);
    assert_eq!(ownership.ui_components.len(), 1);
    assert_eq!(ownership.plugin_options.len(), 1);
    assert_eq!(ownership.plugin_event_catalogs.len(), 1);
}

#[test]
fn runtime_extension_registry_revokes_owner_tracked_contributions() {
    let mut registry = RuntimeExtensionRegistry::default();
    let weather = registry
        .intern_plugin_module("weather.runtime")
        .expect("weather owner");
    let storm = registry
        .intern_plugin_module("storm.runtime")
        .expect("storm owner");

    registry
        .register_component(ComponentTypeDescriptor::new(
            "weather.Component.CloudLayer",
            "weather",
            "Cloud Layer",
        ))
        .unwrap();
    registry
        .register_component(ComponentTypeDescriptor::new(
            "storm.Component.LightningCell",
            "storm",
            "Lightning Cell",
        ))
        .unwrap();
    registry
        .register_plugin_option(PluginOptionManifest::new(
            "weather.precipitation",
            "Precipitation",
            "bool",
            "true",
        ))
        .unwrap();
    registry
        .register_event::<WeatherRegistryEvent>(
            weather,
            PluginEventManifest {
                id: "weather.changed".to_string(),
                display_name: "Weather Changed".to_string(),
                payload_schema: "weather.schemas.changed.v1".to_string(),
            },
        )
        .unwrap();
    registry
        .register_resource::<WeatherRegistryResource>(weather, || WeatherRegistryResource)
        .unwrap();
    registry
        .register_asset_importer_descriptor(
            AssetImporterDescriptor::new(
                "weather.data",
                "weather",
                crate::asset::AssetKind::Data,
                1,
            )
            .with_source_extensions(["weather"]),
        )
        .unwrap();
    registry
        .register_asset_importer_descriptor(
            AssetImporterDescriptor::new("storm.data", "storm", crate::asset::AssetKind::Data, 1)
                .with_source_extensions(["storm"]),
        )
        .unwrap();
    registry
        .register_native_system::<(), _>(weather, "weather.tick", SystemStage::Update, |()| {})
        .register()
        .unwrap();
    registry
        .register_native_system::<(), _>(storm, "storm.tick", SystemStage::Update, |()| {})
        .register()
        .unwrap();

    let removed = registry.revoke_owner_registrations(weather);

    assert_eq!(removed.components.len(), 1);
    assert_eq!(removed.plugin_options.len(), 1);
    assert_eq!(removed.plugin_events.len(), 1);
    assert_eq!(removed.plugin_event_catalogs.len(), 1);
    assert_eq!(removed.plugin_resources.len(), 1);
    assert_eq!(removed.plugin_systems.len(), 1);
    assert_eq!(removed.asset_importers.len(), 1);
    assert_eq!(removed.asset_importers[0].id, "weather.data");
    assert!(registry.ownership_for(weather).is_empty());
    assert_eq!(registry.ownership_for(storm).components.len(), 1);
    assert_eq!(registry.ownership_for(storm).plugin_systems.len(), 1);
    assert_eq!(registry.ownership_for(storm).asset_importers.len(), 1);
    assert!(registry
        .components()
        .iter()
        .all(|component| component.plugin_id != "weather"));
    assert!(registry
        .plugin_systems()
        .all(|(_, system)| system.id != "weather.tick"));
    assert_eq!(registry.asset_importers().descriptors()[0].id, "storm.data");
    registry
        .register_component(ComponentTypeDescriptor::new(
            "weather.Component.CloudLayer",
            "weather",
            "Cloud Layer",
        ))
        .unwrap();
    registry
        .register_asset_importer_descriptor(
            AssetImporterDescriptor::new(
                "weather.data",
                "weather",
                crate::asset::AssetKind::Data,
                1,
            )
            .with_source_extensions(["weather"]),
        )
        .unwrap();
}

#[derive(Debug, PartialEq, Eq)]
struct WeatherRegistryResource;

impl Resource for WeatherRegistryResource {}

#[derive(Debug, PartialEq, Eq)]
struct WeatherRegistryEvent;

#[derive(Debug)]
struct ManifestDeclaredRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl RuntimePlugin for ManifestDeclaredRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        self.descriptor
            .package_manifest()
            .with_option(
                PluginOptionManifest::new("weather.precipitation", "Precipitation", "bool", "true")
                    .with_required_capability("runtime.plugin.weather"),
            )
            .with_event_catalog(PluginEventCatalogManifest {
                namespace: "weather.events".to_string(),
                version: 1,
                events: vec![PluginEventManifest {
                    id: "weather.events.storm_front_arrived".to_string(),
                    display_name: "Storm Front Arrived".to_string(),
                    payload_schema: "weather.schemas.storm_front_payload.v1".to_string(),
                }],
            })
            .with_component(ComponentTypeDescriptor::new(
                "weather.Component.CloudLayer",
                "weather",
                "Cloud Layer",
            ))
            .with_ui_component(UiComponentDescriptor::new(
                "weather.Ui.CloudLayerInspector",
                "weather",
                "asset://weather/editor/cloud_layer_inspector.zui",
            ))
            .with_asset_importer(
                AssetImporterDescriptor::new(
                    "weather.data",
                    "weather",
                    crate::asset::AssetKind::Data,
                    7,
                )
                .with_source_extensions(["weather"]),
            )
    }
}
