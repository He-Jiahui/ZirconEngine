use std::sync::Arc;

use crate::engine_module::EngineModule;
use crate::plugin::RuntimePluginAvailabilityReport;

use super::super::ids::RuntimePluginId;
use super::missing::RuntimeRequiredPluginMissing;

#[derive(Clone, Debug)]
pub struct RuntimeModuleLoadReport {
    pub modules: Vec<Arc<dyn EngineModule>>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub runtime_plugin_availability: RuntimePluginAvailabilityReport,
    required_missing: Vec<RuntimeRequiredPluginMissing>,
}

impl RuntimeModuleLoadReport {
    pub(in crate::builtin::runtime_modules) fn new(modules: Vec<Arc<dyn EngineModule>>) -> Self {
        Self {
            modules,
            warnings: Vec::new(),
            errors: Vec::new(),
            runtime_plugin_availability: RuntimePluginAvailabilityReport::default(),
            required_missing: Vec::new(),
        }
    }

    pub(in crate::builtin::runtime_modules) fn with_runtime_plugin_availability(
        mut self,
        runtime_plugin_availability: RuntimePluginAvailabilityReport,
    ) -> Self {
        self.runtime_plugin_availability = runtime_plugin_availability;
        self
    }

    pub(in crate::builtin::runtime_modules) fn push_required_missing(
        &mut self,
        id: RuntimePluginId,
        reason: String,
    ) {
        self.required_missing
            .push(RuntimeRequiredPluginMissing { id, reason });
    }

    pub fn required_missing(&self) -> &[RuntimeRequiredPluginMissing] {
        &self.required_missing
    }

    pub(crate) fn owned_required_missing(&self) -> Vec<RuntimeRequiredPluginMissing> {
        self.required_missing.clone()
    }
}
