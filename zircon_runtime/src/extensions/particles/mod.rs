mod config;
mod module;
mod service_types;

pub use config::ParticlesConfig;
pub use module::{
    module_descriptor, ParticlesModule, PARTICLES_DRIVER_NAME, PARTICLES_MANAGER_NAME,
    PARTICLES_MODULE_NAME,
};
pub use service_types::{ParticlesDriver, ParticlesManager};
