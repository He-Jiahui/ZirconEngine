use crate::plugin::{
    PluginEventCatalogManifest, PluginEventManifest, PluginPackageManifest,
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginCatalog,
    RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};
use crate::{RuntimePluginId, RuntimeTargetMode};

#[test]
fn runtime_extension_registry_accepts_valid_plugin_event_catalog() {
    let mut registry = RuntimeExtensionRegistry::default();
    let catalog = valid_event_catalog();

    registry
        .register_plugin_event_catalog(catalog.clone())
        .expect("valid event catalog should register");

    assert_eq!(registry.plugin_event_catalogs(), &[catalog]);
}

#[test]
fn typed_event_registration_derives_schema_valid_event_catalog() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("weather.runtime")
        .expect("plugin module id");

    registry
        .register_event::<WeatherRegistryEvent>(
            owner,
            PluginEventManifest {
                id: "weather.events.changed".to_string(),
                display_name: "Weather Changed".to_string(),
                payload_schema: "weather.schemas.changed.v1".to_string(),
            },
        )
        .expect("typed event should register");

    assert_eq!(registry.plugin_event_catalogs(), &[typed_event_catalog()]);
}

#[test]
fn typed_event_registration_rejects_event_id_outside_derived_catalog_namespace() {
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("weather.runtime")
        .expect("plugin module id");

    let error = registry
        .register_event::<WeatherRegistryEvent>(
            owner,
            PluginEventManifest {
                id: "weather.changed".to_string(),
                display_name: "Weather Changed".to_string(),
                payload_schema: "weather.schemas.changed.v1".to_string(),
            },
        )
        .unwrap_err()
        .to_string();

    assert!(error.contains("catalog namespace `weather.events`"));
}

#[test]
fn runtime_plugin_registration_report_validates_shadowed_manifest_event_catalogs() {
    let plugin = ShadowedInvalidEventCatalogRuntimePlugin {
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

    assert!(
        !registration.is_success(),
        "shadowed invalid manifest event catalog should remain diagnostic: {:?}",
        registration.diagnostics
    );
    assert!(registration
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("at least one event")));
    assert_eq!(
        registration.extensions.plugin_event_catalogs(),
        &[valid_event_catalog()]
    );

    let report =
        RuntimePluginCatalog::from_registration_reports([registration], []).runtime_extensions();
    assert!(report.fatal_diagnostics.iter().any(|diagnostic| diagnostic
        .contains("runtime plugin weather diagnostic")
        && diagnostic.contains("at least one event")));
}

#[test]
fn native_runtime_plugin_registration_report_diagnoses_duplicate_manifest_event_catalogs() {
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_event_catalog(valid_event_catalog())
            .with_event_catalog(valid_event_catalog()),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("event catalog namespace `weather.events`")
        && diagnostic.contains("unique")));
    assert_eq!(registration.extensions.plugin_event_catalogs().len(), 1);
}

#[test]
fn native_runtime_plugin_registration_report_diagnoses_unowned_manifest_event_catalogs() {
    let mut catalog = valid_event_catalog();
    catalog.namespace = "storm.events".to_string();
    catalog.events[0].id = "storm.events.storm_front_arrived".to_string();
    catalog.events[0].payload_schema = "storm.schemas.storm_front_payload.v1".to_string();
    catalog.events[1].id = "storm.events.tick".to_string();

    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new("weather", "Weather")
            .with_capability("runtime.plugin.weather")
            .with_event_catalog(catalog),
    );

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| diagnostic
        .contains("event catalog namespace `storm.events`")
        && diagnostic.contains("package id `weather`")));
    assert_eq!(registration.extensions.plugin_event_catalogs().len(), 1);
}

#[test]
fn runtime_extension_registry_rejects_invalid_event_catalog_namespace() {
    let mut catalog = valid_event_catalog();
    catalog.namespace = "weather".to_string();
    assert!(register_error(catalog).contains("dot-separated namespace"));

    let mut catalog = valid_event_catalog();
    catalog.namespace = "Weather.events".to_string();
    assert!(register_error(catalog).contains("lowercase ASCII"));
}

#[test]
fn runtime_extension_registry_rejects_empty_event_catalogs() {
    let mut catalog = valid_event_catalog();
    catalog.version = 0;
    assert!(register_error(catalog).contains("positive u32"));

    let mut catalog = valid_event_catalog();
    catalog.events.clear();
    assert!(register_error(catalog).contains("at least one event"));
}

#[test]
fn runtime_extension_registry_rejects_events_outside_catalog_namespace() {
    let mut catalog = valid_event_catalog();
    catalog.events[0].id = "weather.other.storm_front_arrived".to_string();
    assert!(register_error(catalog).contains("must stay under catalog namespace"));
}

#[test]
fn runtime_extension_registry_rejects_duplicate_event_ids() {
    let mut catalog = valid_event_catalog();
    catalog.events.push(catalog.events[0].clone());
    assert!(register_error(catalog).contains("must be unique"));
}

#[test]
fn runtime_extension_registry_rejects_invalid_event_payload_schema() {
    let mut catalog = valid_event_catalog();
    catalog.events[0].payload_schema = "weather.schemas.storm_front_payload".to_string();
    assert!(register_error(catalog).contains("version segment"));

    let mut catalog = valid_event_catalog();
    catalog.events[0].payload_schema = "cloud.schemas.storm_front_payload.v1".to_string();
    assert!(register_error(catalog).contains("package namespace"));

    let mut catalog = valid_event_catalog();
    catalog.events[0].payload_schema = "weather.schemas.storm_front_payload.v01".to_string();
    assert!(register_error(catalog).contains("positive integer"));
}

#[test]
fn runtime_extension_registry_rejects_untrimmed_event_display_names() {
    let mut catalog = valid_event_catalog();
    catalog.events[0].display_name = " Storm Front Arrived".to_string();
    assert!(register_error(catalog).contains("display_name"));
}

fn valid_event_catalog() -> PluginEventCatalogManifest {
    PluginEventCatalogManifest {
        namespace: "weather.events".to_string(),
        version: 1,
        events: vec![
            PluginEventManifest {
                id: "weather.events.storm_front_arrived".to_string(),
                display_name: "Storm Front Arrived".to_string(),
                payload_schema: "weather.schemas.storm_front_payload.v1".to_string(),
            },
            PluginEventManifest {
                id: "weather.events.tick".to_string(),
                display_name: "Tick".to_string(),
                payload_schema: String::new(),
            },
        ],
    }
}

fn typed_event_catalog() -> PluginEventCatalogManifest {
    PluginEventCatalogManifest {
        namespace: "weather.events".to_string(),
        version: 1,
        events: vec![PluginEventManifest {
            id: "weather.events.changed".to_string(),
            display_name: "Weather Changed".to_string(),
            payload_schema: "weather.schemas.changed.v1".to_string(),
        }],
    }
}

fn register_error(catalog: PluginEventCatalogManifest) -> String {
    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_plugin_event_catalog(catalog)
        .unwrap_err()
        .to_string()
}

#[derive(Debug)]
struct WeatherRegistryEvent;

#[derive(Debug)]
struct ShadowedInvalidEventCatalogRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl RuntimePlugin for ShadowedInvalidEventCatalogRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn package_manifest(&self) -> PluginPackageManifest {
        let mut catalog = valid_event_catalog();
        catalog.events.clear();
        self.descriptor()
            .package_manifest()
            .with_event_catalog(catalog)
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        registry.register_plugin_event_catalog(valid_event_catalog())
    }
}
