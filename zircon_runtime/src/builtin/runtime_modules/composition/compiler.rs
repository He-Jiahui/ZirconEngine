use std::sync::Arc;

use crate::core::framework::project::RuntimeProfileId;
use crate::engine_module::EngineModule;
use crate::plugin::CompiledProjectPluginPlan;

use super::super::assembly::compiled_plan::assemble_compiled_project_plugin_plan_candidate;
use super::super::composition::{
    finish_runtime_module_composition, RuntimeModuleCompositionIdentitySeed,
    RuntimeModuleCompositionResult,
};

/// Compiles the one final module graph consumed by a runtime product entry.
pub struct RuntimeModuleCompositionCompiler<'a> {
    plugin_plan: &'a CompiledProjectPluginPlan,
    runtime_profile: Option<RuntimeProfileId>,
    host_modules: Vec<Arc<dyn EngineModule>>,
}

impl<'a> RuntimeModuleCompositionCompiler<'a> {
    pub fn new(plugin_plan: &'a CompiledProjectPluginPlan) -> Self {
        Self {
            plugin_plan,
            runtime_profile: None,
            host_modules: Vec::new(),
        }
    }

    pub fn for_runtime_profile(mut self, runtime_profile: RuntimeProfileId) -> Self {
        self.runtime_profile = Some(runtime_profile);
        self
    }

    pub fn with_host_module(mut self, module: Arc<dyn EngineModule>) -> Self {
        self.host_modules.push(module);
        self
    }

    pub fn with_host_modules(
        mut self,
        modules: impl IntoIterator<Item = Arc<dyn EngineModule>>,
    ) -> Self {
        self.host_modules.extend(modules);
        self
    }

    pub fn compile(self) -> RuntimeModuleCompositionResult {
        let mut report =
            assemble_compiled_project_plugin_plan_candidate(self.plugin_plan, self.runtime_profile);
        report.modules.extend(self.host_modules);
        finish_runtime_module_composition(
            report,
            RuntimeModuleCompositionIdentitySeed::compiled(self.plugin_plan, self.runtime_profile),
        )
    }
}
