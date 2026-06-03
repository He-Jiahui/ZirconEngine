use crate::asset::AssetImporterDescriptor;
use crate::plugin::{
    ComponentTypeDescriptor, PluginEventCatalogManifest, PluginEventManifest, PluginOptionManifest,
    PluginPackageManifest, RuntimePlugin, RuntimePluginCatalog, RuntimePluginDescriptor,
    RuntimePluginRegistrationReport, UiComponentDescriptor,
};
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
