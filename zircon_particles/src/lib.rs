//! Particle module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, EngineModule, ModuleDescriptor};

pub const PARTICLES_MODULE_NAME: &str = "ParticlesModule";

#[derive(Clone, Debug, Default)]
pub struct ParticlesConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ParticlesModule;

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
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
