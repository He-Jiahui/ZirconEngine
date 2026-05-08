use crate::core::ModuleDescriptor;
use crate::engine_module::EngineModule;

pub const DIAGNOSTICS_CORE_MODULE_NAME: &str = "DiagnosticsCoreModule";

#[derive(Clone, Copy, Debug, Default)]
pub struct DiagnosticsCoreModule;

impl EngineModule for DiagnosticsCoreModule {
    fn module_name(&self) -> &'static str {
        DIAGNOSTICS_CORE_MODULE_NAME
    }

    fn module_description(&self) -> &'static str {
        "Core diagnostics descriptor for runtime tooling snapshots"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(DIAGNOSTICS_CORE_MODULE_NAME, self.module_description())
    }
}
