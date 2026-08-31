use std::sync::Arc;

use crate::engine_module::EngineModule;
use crate::plugin::RuntimePluginAvailabilityReport;

use super::diagnostics::RuntimeModuleLoadDiagnostic;
use crate::core::CoreError;

#[derive(Clone, Debug)]
pub(in crate::builtin::runtime_modules) struct RuntimeModuleLoadReport {
    pub(in crate::builtin::runtime_modules) modules: Vec<Arc<dyn EngineModule>>,
    pub(in crate::builtin::runtime_modules) runtime_plugin_availability:
        RuntimePluginAvailabilityReport,
    pub(in crate::builtin::runtime_modules) diagnostics: Vec<RuntimeModuleLoadDiagnostic>,
}

impl RuntimeModuleLoadReport {
    pub(in crate::builtin::runtime_modules) fn new(modules: Vec<Arc<dyn EngineModule>>) -> Self {
        Self {
            modules,
            runtime_plugin_availability: RuntimePluginAvailabilityReport::default(),
            diagnostics: Vec::new(),
        }
    }

    pub(in crate::builtin::runtime_modules) fn from_core_error(error: CoreError) -> Self {
        let mut report = Self::new(Vec::new());
        report.push_diagnostic(RuntimeModuleLoadDiagnostic::Core(error));
        report
    }

    pub(in crate::builtin::runtime_modules) fn with_runtime_plugin_availability(
        mut self,
        runtime_plugin_availability: RuntimePluginAvailabilityReport,
    ) -> Self {
        self.runtime_plugin_availability = runtime_plugin_availability;
        self
    }

    pub(in crate::builtin::runtime_modules) fn push_diagnostic(
        &mut self,
        diagnostic: RuntimeModuleLoadDiagnostic,
    ) {
        self.diagnostics.push(diagnostic);
    }

    pub(in crate::builtin::runtime_modules) fn extend_diagnostics(
        &mut self,
        diagnostics: impl IntoIterator<Item = RuntimeModuleLoadDiagnostic>,
    ) {
        self.diagnostics.extend(diagnostics);
    }

    pub(in crate::builtin::runtime_modules) fn diagnostics(
        &self,
    ) -> &[RuntimeModuleLoadDiagnostic] {
        &self.diagnostics
    }
}
