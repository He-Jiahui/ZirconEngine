//! Module and plugin descriptor helpers built on top of the core runtime.

mod contexts;
mod descriptors;
mod engine_module;
mod engine_service;
mod service_factory;

pub use crate::core::{
    CoreHandle, CoreRuntime, CoreWeak, DependencySpec, DriverDescriptor, InitLevel, LifecycleState,
    ManagerDescriptor, ModuleContext, ModuleDependencySpec, ModuleDescriptor, ModuleLifecycle,
    NoopModuleLifecycle, PluginContext, PluginDescriptor, PluginFactory, RegistryName,
    ServiceFactory, ServiceKind, StartupMode,
};

pub use contexts::{module_context, plugin_context};
pub use descriptors::{dependency_on, qualified_name};
pub use engine_module::EngineModule;
pub use engine_service::{
    driver_contract, manager_contract, plugin_contract, DriverContract, EngineDriver,
    EngineManager, EnginePlugin, EngineService, ManagerContract, PluginContract,
};
pub use service_factory::{factory, plugin_factory};

#[cfg(test)]
mod tests;
