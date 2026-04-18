//! Particles module scaffold with explicit core service descriptors.

use std::sync::Arc;

use zircon_module::{
    dependency_on, factory, qualified_name, DriverDescriptor, EngineModule, ManagerDescriptor,
    ModuleDescriptor, ServiceKind, StartupMode,
};

pub const PARTICLES_MODULE_NAME: &str = "ParticlesModule";
pub const PARTICLES_DRIVER_NAME: &str = "ParticlesModule.Driver.ParticlesDriver";
pub const PARTICLES_MANAGER_NAME: &str = "ParticlesModule.Manager.ParticlesManager";

#[derive(Clone, Debug, Default)]
pub struct ParticlesConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ParticlesModule;

#[derive(Clone, Debug, Default)]
pub struct ParticlesDriver;

#[derive(Clone, Debug, Default)]
pub struct ParticlesManager;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(
        PARTICLES_MODULE_NAME,
        "Particle simulation and VFX authoring data",
    )
    .with_driver(DriverDescriptor::new(
        qualified_name(
            PARTICLES_MODULE_NAME,
            ServiceKind::Driver,
            "ParticlesDriver",
        ),
        StartupMode::Immediate,
        Vec::new(),
        factory(|_| Ok(Arc::new(ParticlesDriver::default()) as _)),
    ))
    .with_manager(ManagerDescriptor::new(
        qualified_name(
            PARTICLES_MODULE_NAME,
            ServiceKind::Manager,
            "ParticlesManager",
        ),
        StartupMode::Lazy,
        vec![dependency_on(
            PARTICLES_MODULE_NAME,
            ServiceKind::Driver,
            "ParticlesDriver",
        )],
        factory(|_| Ok(Arc::new(ParticlesManager::default()) as _)),
    ))
}

impl EngineModule for ParticlesModule {
    fn module_name(&self) -> &'static str {
        PARTICLES_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Particle simulation and VFX authoring data"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
