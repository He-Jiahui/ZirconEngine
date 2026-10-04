use crate::core::{InitLevel, ModuleDependencySpec, ModuleDescriptor};
use crate::engine_module::EngineModule;

use super::diagnostics::DIAGNOSTICS_CORE_MODULE_NAME;

pub const LOG_MODULE_NAME: &str = "LogModule";
pub const LOG_DIAGNOSTICS_MODULE_NAME: &str = "LogDiagnosticsModule";

#[derive(Clone, Copy, Debug, Default)]
/// 基础 Kernel 声明由内建模块组无条件加入；LogDiagnosticsModule 则只在诊断配置要求时按需加入。
pub struct LogModule;

impl EngineModule for LogModule {
    fn module_name(&self) -> &'static str {
        LOG_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Core process log descriptor for diagnostic log filters and sinks"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(LOG_MODULE_NAME, self.module_description())
            .with_init_level(InitLevel::Kernel)
    }
}

/// 可选的详细日志诊断节点；启动构建器在需要诊断配置时添加它。
/// 其模块边要求基础日志与诊断能力先可用。
#[derive(Clone, Copy, Debug, Default)]
pub struct LogDiagnosticsModule;

impl EngineModule for LogDiagnosticsModule {
    fn module_name(&self) -> &'static str {
        LOG_DIAGNOSTICS_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Development log diagnostics descriptor for verbose runtime diagnostics profiles"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(LOG_DIAGNOSTICS_MODULE_NAME, self.module_description())
            .with_init_level(InitLevel::Kernel)
            .with_module_dependency(ModuleDependencySpec::named(LOG_MODULE_NAME))
            .with_module_dependency(ModuleDependencySpec::named(DIAGNOSTICS_CORE_MODULE_NAME))
    }
}
