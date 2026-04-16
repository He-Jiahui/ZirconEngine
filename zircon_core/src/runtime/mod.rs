//! Service registry and core runtime.

mod contexts;
mod descriptors;
mod handle;
mod runtime;
mod state;
mod weak;

pub use contexts::{ModuleContext, PluginContext};
pub use descriptors::{
    DependencySpec, DriverDescriptor, ManagerDescriptor, ModuleDescriptor, PluginDescriptor,
    RegistryName, ServiceFactory,
};
pub use handle::CoreHandle;
pub use runtime::CoreRuntime;
pub use weak::CoreWeak;

#[cfg(test)]
mod tests;
