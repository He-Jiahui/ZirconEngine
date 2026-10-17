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
    minimum_maturity: Option<crate::plugin::PluginMaturity>,
    host_modules: Vec<Arc<dyn EngineModule>>,
}

impl<'a> RuntimeModuleCompositionCompiler<'a> {
    pub fn new(plugin_plan: &'a CompiledProjectPluginPlan) -> Self {
        Self {
            plugin_plan,
            runtime_profile: plugin_plan.runtime_profile_admission().map(|(id, _)| id),
            minimum_maturity: plugin_plan
                .runtime_profile_admission()
                .map(|(_, minimum)| minimum),
            host_modules: Vec::new(),
        }
    }

    pub fn for_runtime_profile(mut self, runtime_profile: RuntimeProfileId) -> Self {
        self.runtime_profile = Some(runtime_profile);
        self.minimum_maturity =
            Some(crate::plugin::RuntimeProfileDescriptor::for_id(runtime_profile).minimum_maturity);
        self
    }

    /// Requires a plan already admitted with this exact maturity policy.
    pub fn with_minimum_maturity(mut self, minimum: crate::plugin::PluginMaturity) -> Self {
        self.minimum_maturity = Some(minimum);
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
        let expected_admission = self.runtime_profile.zip(self.minimum_maturity);
        let target_matches = self.runtime_profile.is_none_or(|id| {
            crate::plugin::RuntimeProfileDescriptor::for_id(id).target_mode
                == self.plugin_plan.target_mode()
        });
        let policy_matches = expected_admission == self.plugin_plan.runtime_profile_admission()
            && (self.runtime_profile.is_none() == self.minimum_maturity.is_none());
        // Preserve the existing typed target mismatch; reject missing/different
        // admission before any module can enter the activation graph.
        let mut report = if target_matches && !policy_matches {
            super::super::load_report::RuntimeModuleLoadReport::from_core_error(
                crate::core::CoreError::Initialization(
                    "runtime plugin plan profile policy mismatch".to_owned(),
                    format!(
                        "requested {expected_admission:?}, applied {:?}",
                        self.plugin_plan.runtime_profile_admission()
                    ),
                ),
            )
        } else {
            assemble_compiled_project_plugin_plan_candidate(self.plugin_plan, self.runtime_profile)
        };
        report.modules.extend(self.host_modules);
        finish_runtime_module_composition(
            report,
            RuntimeModuleCompositionIdentitySeed::compiled(self.plugin_plan, self.runtime_profile),
        )
    }
}
