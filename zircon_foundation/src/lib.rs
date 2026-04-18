//! Built-in runtime foundation services layered above the core runtime.

mod module;
mod runtime;

use zircon_module::{EngineModule, ModuleDescriptor};

pub use module::{
    module_descriptor, CONFIG_DRIVER_NAME, EVENT_DRIVER_NAME, FOUNDATION_MODULE_NAME,
};
pub use runtime::{ConfigDriver, DefaultConfigManager, DefaultEventManager, EventDriver};

#[derive(Clone, Copy, Debug, Default)]
pub struct FoundationModule;

impl EngineModule for FoundationModule {
    fn module_name(&self) -> &'static str {
        FOUNDATION_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Built-in runtime foundation services"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}

#[cfg(test)]
mod tests;
