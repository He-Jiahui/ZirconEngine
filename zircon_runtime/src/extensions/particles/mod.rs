mod service_types;

use crate::engine_module::{EngineModule, ModuleDescriptor};

pub use service_types::{ParticlesDriver, ParticlesManager};

pub const PARTICLES_MODULE_NAME: &str = "ParticlesModule";
pub const PARTICLES_DRIVER_NAME: &str = "ParticlesModule.Driver.ParticlesDriver";
pub const PARTICLES_MANAGER_NAME: &str = "ParticlesModule.Manager.ParticlesManager";

#[derive(Clone, Debug, Default)]
pub struct ParticlesConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ParticlesModule;

pub fn module_descriptor() -> ModuleDescriptor {
    super::module_descriptor_with_driver_and_manager::<ParticlesDriver, ParticlesManager>(
        PARTICLES_MODULE_NAME,
        "Particle simulation and VFX authoring data",
        "ParticlesDriver",
        "ParticlesManager",
    )
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
