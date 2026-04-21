use crate::engine_module::{EngineModule, ModuleDescriptor};

use super::{module_descriptor, INPUT_MODULE_NAME};

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
