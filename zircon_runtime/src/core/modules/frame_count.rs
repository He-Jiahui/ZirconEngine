use crate::core::ModuleDescriptor;
use crate::engine_module::EngineModule;

pub const FRAME_COUNT_MODULE_NAME: &str = "FrameCountModule";

#[derive(Clone, Copy, Debug, Default)]
pub struct FrameCountModule;

impl EngineModule for FrameCountModule {
    fn module_name(&self) -> &'static str {
        FRAME_COUNT_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Core frame count descriptor for runtime diagnostics"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(FRAME_COUNT_MODULE_NAME, self.module_description())
    }
}
