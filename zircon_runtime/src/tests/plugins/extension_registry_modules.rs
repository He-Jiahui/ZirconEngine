//! 核对模块贡献的收集、字段约束与重名拒绝，并验证非法贡献会使插件注册报告和目录聚合报告携带失败诊断。
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

#[test]
fn runtime_named_module_revokes_with_the_exact_declared_owner() {
    let mut registry = RuntimeExtensionRegistry::default();
    registry
        .register_module(ModuleDescriptor::new("client.runtime", "Client module"))
        .unwrap();
    registry
        .register_module(ModuleDescriptor::new("other.runtime", "Other module"))
        .unwrap();
    let owner = registry.intern_plugin_module("client.runtime").unwrap();
    let other = registry.intern_plugin_module("other.runtime").unwrap();
    assert_eq!(registry.ownership_for(owner).modules.len(), 1);
    assert_eq!(registry.ownership_for(other).modules.len(), 1);
    let removed = registry.revoke_owner_registrations(owner);
    assert_eq!(removed.modules.len(), 1);
    assert!(registry.ownership_for(owner).is_empty());
    assert_eq!(registry.modules().len(), 1);
    assert_eq!(registry.modules()[0].name, "other.runtime");
    assert_eq!(registry.ownership_for(other).modules.len(), 1);
    assert!(registry.revoke_owner_registrations(owner).is_empty());
}
