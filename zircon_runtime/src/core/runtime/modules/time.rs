use crate::core::{InitLevel, ModuleDescriptor};
use crate::engine_module::EngineModule;

pub const TIME_MODULE_NAME: &str = "TimeModule";

#[derive(Clone, Copy, Debug, Default)]
pub struct TimeModule;

impl EngineModule for TimeModule {
    fn module_name(&self) -> &'static str {
        TIME_MODULE_NAME
    }

    // TODO: [CR-RUNTIME-LIFECYCLE-0002] 核对描述中的 virtual/fixed 所有权；当前 World 派生时钟由 LevelSystem 持有。
    // 当前实现中 RuntimeTimeAuthority 只持有 MonotonicReal 与新 World 的默认策略；LevelSystem 的 WorldTimeController 才持有虚拟/固定时钟及其债务和提交边界。下方描述字符串仍不准确。
    fn module_description(&self) -> &'static str {
        "Core frame timing descriptor for runtime-owned real, virtual, and fixed clocks"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(TIME_MODULE_NAME, self.module_description())
            .with_init_level(InitLevel::Kernel)
    }
}
