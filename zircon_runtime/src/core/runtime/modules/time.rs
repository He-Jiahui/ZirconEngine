use crate::core::ModuleDescriptor;
use crate::engine_module::EngineModule;

pub const TIME_MODULE_NAME: &str = "TimeModule";

#[derive(Clone, Copy, Debug, Default)]
pub struct TimeModule;

impl EngineModule for TimeModule {
    fn module_name(&self) -> &'static str {
        TIME_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Core frame timing descriptor for runtime-owned real, virtual, and fixed clocks"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(TIME_MODULE_NAME, self.module_description())
    }
}
