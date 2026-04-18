//! Input module wired into the core runtime with a stable façade.

mod module;
mod runtime;

use zircon_module::{EngineModule, ModuleDescriptor};

pub use module::{
    module_descriptor, InputConfig, INPUT_DRIVER_NAME, INPUT_MANAGER_NAME, INPUT_MODULE_NAME,
};
pub use runtime::{DefaultInputManager, InputDriver};
pub use zircon_input_protocol::{InputButton, InputEvent, InputEventRecord, InputSnapshot};

#[derive(Clone, Copy, Debug, Default)]
pub struct InputModule;

impl EngineModule for InputModule {
    fn module_name(&self) -> &'static str {
        INPUT_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "High-level input routing and action maps"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}

#[cfg(test)]
mod tests;
