//! Optional extension module registration surfaces absorbed into the runtime layer.

pub mod animation;
pub mod navigation;
pub mod net;
pub mod particles;
pub mod physics;
pub mod sound;
pub mod texture;

use std::sync::Arc;

use zircon_core::{
    DriverDescriptor, ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_module::{dependency_on, factory, qualified_name};

fn module_descriptor_with_driver_and_manager<D, M>(
    module_name: &'static str,
    description: &'static str,
    driver_name: &'static str,
    manager_name: &'static str,
) -> ModuleDescriptor
where
    D: Default + Send + Sync + 'static,
    M: Default + Send + Sync + 'static,
{
    ModuleDescriptor::new(module_name, description)
        .with_driver(DriverDescriptor::new(
            qualified_name(module_name, ServiceKind::Driver, driver_name),
            StartupMode::Immediate,
            Vec::new(),
            factory(|_| Ok(Arc::new(D::default()) as ServiceObject)),
        ))
        .with_manager(ManagerDescriptor::new(
            qualified_name(module_name, ServiceKind::Manager, manager_name),
            StartupMode::Lazy,
            vec![dependency_on(module_name, ServiceKind::Driver, driver_name)],
            factory(|_| Ok(Arc::new(M::default()) as ServiceObject)),
        ))
}
