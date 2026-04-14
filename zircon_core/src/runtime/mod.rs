//! Service registry and core runtime.

mod core;

pub use core::{
    CoreHandle, CoreRuntime, CoreWeak, DependencySpec, DriverDescriptor, ManagerDescriptor,
    ModuleContext, ModuleDescriptor, PluginContext, PluginDescriptor, RegistryName, ServiceFactory,
};
