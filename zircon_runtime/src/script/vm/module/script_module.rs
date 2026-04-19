use crate::engine_module::{EngineModule, ModuleDescriptor};

use super::module_descriptor;
use crate::script::SCRIPT_MODULE_NAME;

#[derive(Clone, Copy, Debug, Default)]
pub struct ScriptModule;

impl EngineModule for ScriptModule {
    fn module_name(&self) -> &'static str {
        SCRIPT_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "VM plugin hosting and hot reload"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        module_descriptor()
    }
}
