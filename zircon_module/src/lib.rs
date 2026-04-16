//! Module and plugin descriptor helpers built on top of the core runtime.

mod contexts;
mod descriptors;
mod engine_module;
mod service_factory;

pub use zircon_core::{
    CoreHandle, CoreRuntime, CoreWeak, DependencySpec, DriverDescriptor, LifecycleState,
    ManagerDescriptor, ModuleContext, ModuleDescriptor, PluginContext, PluginDescriptor,
    RegistryName, ServiceKind, StartupMode,
};

pub use contexts::{module_context, plugin_context};
pub use descriptors::{
    dependency_on, qualified_name, stub_driver_descriptor, stub_manager_descriptor,
    stub_module_descriptor, stub_plugin_descriptor, StubDriver, StubManager,
};
pub use engine_module::EngineModule;
pub use service_factory::factory;

#[cfg(test)]
mod tests;
