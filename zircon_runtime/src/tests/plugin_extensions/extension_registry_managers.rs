use crate::builtin::RuntimePluginId;
use crate::core::runtime::ServiceObject;
use crate::core::{ManagerDescriptor, ModuleDescriptor, ServiceKind, StartupMode};
use crate::engine_module::{factory, qualified_name};
use crate::plugin::{
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginCatalog,
    RuntimePluginDescriptor,
};

#[test]
fn runtime_extension_registry_collects_manager_contributions_and_projects_into_modules() {
    let mut registry = RuntimeExtensionRegistry::default();
    let manager = weather_manager();

    registry
        .register_manager("weather", manager.clone())
        .expect("manager contribution");

    assert_eq!(registry.managers().len(), 1);

    let module = ModuleDescriptor::new("WeatherPlugin", "Weather plugin").with_manager(manager);
    let merged = registry.apply_to_module(module);
    assert_eq!(merged.managers.len(), 2);
}

#[test]
fn runtime_extension_registry_rejects_invalid_manager_plugin_ids() {
    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_manager("Weather", weather_manager())
        .unwrap_err();

    assert!(error.to_string().contains("plugin_id"));
    assert!(error.to_string().contains("lowercase ASCII"));

    let mut registry = RuntimeExtensionRegistry::default();
    let error = registry
        .register_manager("weather.layer", weather_manager())
        .unwrap_err();

    assert!(error.to_string().contains("plugin_id"));
    assert!(error.to_string().contains("lowercase ASCII"));
}

#[test]
fn runtime_extension_registry_rejects_duplicate_manager_contributions() {
    let mut registry = RuntimeExtensionRegistry::default();
    let manager = weather_manager();

    registry
        .register_manager("weather", manager.clone())
        .expect("first manager");
    let duplicate_manager = registry.register_manager("weather", manager).unwrap_err();

    assert!(duplicate_manager
        .to_string()
        .contains("manager WeatherPlugin.Manager.WeatherManager already registered"));
}

#[test]
fn runtime_plugin_catalog_reports_duplicate_manager_contributions() {
    let left = ManagerRuntimePlugin {
        descriptor: RuntimePluginDescriptor::builder(
            "weather_left",
            "Weather Left",
            RuntimePluginId::Particles,
            "zircon_plugin_weather_left_runtime",
        )
        .with_capability("runtime.plugin.weather_left")
        .build(),
    };
    let right = ManagerRuntimePlugin {
        descriptor: RuntimePluginDescriptor::builder(
            "weather_right",
            "Weather Right",
            RuntimePluginId::HybridGi,
            "zircon_plugin_weather_right_runtime",
        )
        .with_capability("runtime.plugin.weather_right")
        .build(),
    };
    let catalog = RuntimePluginCatalog::from_plugins([
        &left as &dyn RuntimePlugin,
        &right as &dyn RuntimePlugin,
    ]);
    let report = catalog.runtime_extensions();

    assert!(!report.is_success());
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("manager WeatherPlugin.Manager.WeatherManager already registered")
    }));
    assert_eq!(report.registry.managers().len(), 1);
}

fn weather_manager() -> ManagerDescriptor {
    ManagerDescriptor::new(
        qualified_name("WeatherPlugin", ServiceKind::Manager, "WeatherManager"),
        StartupMode::Lazy,
        Vec::new(),
        factory(|_| Ok(std::sync::Arc::new(()) as ServiceObject)),
    )
}

#[derive(Debug)]
struct ManagerRuntimePlugin {
    descriptor: RuntimePluginDescriptor,
}

impl RuntimePlugin for ManagerRuntimePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        registry.register_manager(
            self.descriptor().package_id().to_string(),
            weather_manager(),
        )
    }
}
