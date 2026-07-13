use crate::core::ModuleDescriptor;
use crate::plugin::{
    RuntimeExtensionRegistry, RuntimeExtensionRegistryError, RuntimePlugin, RuntimePluginCatalog,
    RuntimePluginDescriptor, RuntimePluginRegistrationReport,
};
use crate::{builtin::RuntimePluginId, core::framework::platform::RuntimeTargetMode};

#[test]
fn runtime_extension_registry_collects_module_contributions() {
    let mut registry = RuntimeExtensionRegistry::default();
    let module = ModuleDescriptor::new("WeatherPlugin", "Weather simulation plugin");

    registry
        .register_module(module.clone())
        .expect("module contribution");

    assert_eq!(registry.modules().len(), 1);
    assert_eq!(registry.modules()[0].name, module.name);
}

#[test]
fn runtime_extension_registry_rejects_invalid_module_descriptor_fields() {
    let mut registry = RuntimeExtensionRegistry::default();

    let empty_name = registry
        .register_module(ModuleDescriptor::new("", "Weather plugin"))
        .unwrap_err();
    assert_invalid_module_message(empty_name, "name `` must be non-empty and trimmed");

    let untrimmed_description = registry
        .register_module(ModuleDescriptor::new("WeatherPlugin", " Weather plugin "))
        .unwrap_err();
    assert_invalid_module_message(
        untrimmed_description,
        "description ` Weather plugin ` must be non-empty and trimmed",
    );
}

#[test]
fn runtime_extension_registry_rejects_duplicate_module_contributions() {
    let mut registry = RuntimeExtensionRegistry::default();

    registry
        .register_module(ModuleDescriptor::new("WeatherPlugin", "Weather plugin"))
        .expect("first module");
    let duplicate_module = registry
        .register_module(ModuleDescriptor::new(
            "WeatherPlugin",
            "Duplicate weather plugin",
        ))
        .unwrap_err();

    assert!(duplicate_module
        .to_string()
        .contains("module WeatherPlugin already registered"));
}

#[test]
fn runtime_plugin_registration_report_rejects_invalid_module_contributions() {
    let plugin = InvalidModulePlugin::new(ModuleDescriptor::new("", "Weather plugin"));
    let registration = RuntimePluginRegistrationReport::from_plugin(&plugin);

    assert!(!registration.is_success());
    assert!(registration.diagnostics.iter().any(|diagnostic| {
        diagnostic.contains("invalid module contribution")
            && diagnostic.contains("name `` must be non-empty and trimmed")
    }));

    let catalog = RuntimePluginCatalog::from_plugins([&plugin as &dyn RuntimePlugin]);

    assert!(!catalog.is_success());
    assert!(catalog.diagnostics().iter().any(|diagnostic| {
        diagnostic.contains("invalid module contribution")
            && diagnostic.contains("name `` must be non-empty and trimmed")
    }));
}

fn assert_invalid_module_message(error: RuntimeExtensionRegistryError, expected: &str) {
    let message = error.to_string();
    assert!(message.contains("invalid module contribution"));
    assert!(message.contains(expected), "{message}");
}

struct InvalidModulePlugin {
    descriptor: RuntimePluginDescriptor,
    module: ModuleDescriptor,
}

impl InvalidModulePlugin {
    fn new(module: ModuleDescriptor) -> Self {
        Self {
            descriptor: RuntimePluginDescriptor::builder(
                "weather",
                "Weather",
                RuntimePluginId::Particles,
                "zircon_plugin_weather_runtime",
            )
            .with_target_modes([RuntimeTargetMode::ClientRuntime])
            .with_capability("runtime.plugin.weather")
            .build(),
            module,
        }
    }
}

impl RuntimePlugin for InvalidModulePlugin {
    fn descriptor(&self) -> &RuntimePluginDescriptor {
        &self.descriptor
    }

    fn register(
        &self,
        registry: &mut RuntimeExtensionRegistry,
    ) -> Result<(), RuntimeExtensionRegistryError> {
        registry.register_module(self.module.clone())
    }
}
