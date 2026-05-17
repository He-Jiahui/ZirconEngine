use crate::core::ModuleDescriptor;
use crate::engine_module::EngineModule;

pub const LOG_MODULE_NAME: &str = "LogModule";
pub const LOG_DIAGNOSTICS_MODULE_NAME: &str = "LogDiagnosticsModule";

#[derive(Clone, Copy, Debug, Default)]
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
    }
}

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
    }
}
