use std::sync::Arc;

use zircon_runtime::core::{
    ManagerDescriptor, ModuleDescriptor, ServiceKind, ServiceObject, StartupMode,
};
use zircon_runtime::engine_module::{factory, qualified_name, EngineModule};

use crate::ParticlesManager;

pub const PARTICLES_MODULE_NAME: &str = "ParticlesModule";
pub const PARTICLES_MANAGER_NAME: &str = "ParticlesModule.Manager.ParticlesManager";

#[derive(Clone, Copy, Debug, Default)]
pub struct ParticlesModule;

pub fn module_descriptor() -> ModuleDescriptor {
    ModuleDescriptor::new(PARTICLES_MODULE_NAME, "CPU/GPU sprite particle simulation").with_manager(
        ManagerDescriptor::new(
            qualified_name(
                PARTICLES_MODULE_NAME,
                ServiceKind::Manager,
                "ParticlesManager",
            ),
            StartupMode::Lazy,
            Vec::new(),
            factory(|_| Ok(Arc::new(ParticlesManager::default()) as ServiceObject)),
        ),
    )
}

impl EngineModule for ParticlesModule {
    fn module_name(&self) -> &'static str {
        PARTICLES_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "CPU/GPU sprite particle simulation"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
