use std::sync::Arc;

use crate::engine_module::EngineModule;
use crate::plugin::RuntimePluginAvailabilityReport;

use super::RuntimePluginId;

#[derive(Clone, Debug)]
pub struct RuntimeModuleLoadReport {
    pub modules: Vec<Arc<dyn EngineModule>>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub runtime_plugin_availability: RuntimePluginAvailabilityReport,
    pub(super) required_missing: Vec<RuntimeRequiredPluginMissing>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeRequiredPluginMissing {
    pub id: RuntimePluginId,
    pub reason: String,
}

impl RuntimeModuleLoadReport {
    pub(super) fn new(modules: Vec<Arc<dyn EngineModule>>) -> Self {
        Self {
            modules,
            warnings: Vec::new(),
            errors: Vec::new(),
            runtime_plugin_availability: RuntimePluginAvailabilityReport::default(),
            required_missing: Vec::new(),
        }
    }

    pub(super) fn with_runtime_plugin_availability(
        mut self,
        runtime_plugin_availability: RuntimePluginAvailabilityReport,
    ) -> Self {
        self.runtime_plugin_availability = runtime_plugin_availability;
        self
    }

    pub fn required_missing(&self) -> &[RuntimeRequiredPluginMissing] {
        &self.required_missing
    }

    pub fn effective_required_missing(&self) -> Vec<RuntimeRequiredPluginMissing> {
        let mut missing = self.required_missing.clone();
        for entry in &self.runtime_plugin_availability.missing_required {
            let structured_missing = RuntimeRequiredPluginMissing {
                id: entry.runtime_id,
                reason: entry.reason.clone(),
            };
            if !missing
                .iter()
                .any(|existing| existing.id == structured_missing.id)
            {
                missing.push(structured_missing);
            }
        }
        missing
    }

    pub fn required_missing_summary(&self) -> String {
        self.effective_required_missing()
            .into_iter()
            .map(|missing| {
                format!(
                    "required runtime plugin {} is unavailable: {}",
                    missing.id.label(),
                    missing.reason
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    }

    pub fn effective_errors(&self) -> Vec<String> {
        let mut errors = self.errors.clone();
        for missing in self.effective_required_missing() {
            let diagnostic = format!(
                "required runtime plugin {} is unavailable: {}",
                missing.id.label(),
                missing.reason
            );
            if !errors.iter().any(|existing| existing == &diagnostic) {
                errors.push(diagnostic);
            }
        }
        errors
    }

    pub fn has_fatal_diagnostics(&self) -> bool {
        !self.effective_errors().is_empty()
    }
}
