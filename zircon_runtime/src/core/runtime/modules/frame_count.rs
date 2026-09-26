use crate::core::{InitLevel, ModuleDependencySpec, ModuleDescriptor};
use crate::engine_module::EngineModule;

use super::time::TIME_MODULE_NAME;

pub const FRAME_COUNT_MODULE_NAME: &str = "FrameCountModule";

/// 帧计数诊断的声明节点；依赖 TimeModule 保证时间入口先进入 Kernel 图。
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
            .with_init_level(InitLevel::Kernel)
            .with_module_dependency(ModuleDependencySpec::named(TIME_MODULE_NAME))
    }
}
