//! Particle module skeleton wired into the core runtime.

use zircon_module::{stub_module_descriptor, ModuleDescriptor};

pub const PARTICLES_MODULE_NAME: &str = "ParticlesModule";

#[derive(Clone, Debug, Default)]
pub struct ParticlesConfig {
    pub enabled: bool,
}

pub fn module_descriptor() -> ModuleDescriptor {
    stub_module_descriptor(
        PARTICLES_MODULE_NAME,
        "Particle simulation and VFX authoring data",
        "ParticlesDriver",
        "ParticlesManager",
    )
}
