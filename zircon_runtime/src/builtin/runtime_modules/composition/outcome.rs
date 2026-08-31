use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::core::runtime::FrozenModuleGraph;
use crate::core::{CoreError, ModuleDescriptor};
use crate::engine_module::EngineModule;
use crate::plugin::{RuntimePluginAvailabilityEntry, RuntimePluginAvailabilityReport};

use super::super::load_report::{RuntimeModuleLoadDiagnostic, RuntimeModuleLoadReport};
use super::identity::{RuntimeModuleCompositionIdentity, RuntimeModuleCompositionIdentitySeed};

pub type RuntimeModuleCompositionResult =
    Result<RuntimeModuleCompositionPlan, RuntimeModuleCompositionRejection>;

#[derive(Clone, Debug)]
pub struct RuntimeModuleCompositionPlan {
    modules: Vec<Arc<dyn EngineModule>>,
    descriptors: Vec<ModuleDescriptor>,
    runtime_plugin_availability: RuntimePluginAvailabilityReport,
    diagnostics: Vec<RuntimeModuleLoadDiagnostic>,
    identity: RuntimeModuleCompositionIdentity,
}

impl RuntimeModuleCompositionPlan {
    pub fn modules(&self) -> &[Arc<dyn EngineModule>] {
        &self.modules
    }

    pub fn module_descriptors(&self) -> &[ModuleDescriptor] {
        &self.descriptors
    }

    pub fn runtime_plugin_availability(&self) -> &RuntimePluginAvailabilityReport {
        &self.runtime_plugin_availability
    }

    pub fn diagnostics(&self) -> &[RuntimeModuleLoadDiagnostic] {
        &self.diagnostics
    }

    pub fn warning_messages(&self) -> Vec<String> {
        super::super::load_report::diagnostics::warning_messages(
            &self.runtime_plugin_availability,
            &self.diagnostics,
        )
    }

    pub fn identity(&self) -> &RuntimeModuleCompositionIdentity {
        &self.identity
    }
}

#[derive(Clone, Debug)]
pub struct RuntimeModuleCompositionRejection {
    runtime_plugin_availability: RuntimePluginAvailabilityReport,
    diagnostics: Vec<RuntimeModuleLoadDiagnostic>,
}

impl RuntimeModuleCompositionRejection {
    pub fn runtime_plugin_availability(&self) -> &RuntimePluginAvailabilityReport {
        &self.runtime_plugin_availability
    }

    pub fn diagnostics(&self) -> &[RuntimeModuleLoadDiagnostic] {
        &self.diagnostics
    }

    pub fn required_missing(&self) -> &[RuntimePluginAvailabilityEntry] {
        &self.runtime_plugin_availability.missing_required
    }

    pub fn required_missing_summary(&self) -> String {
        self.required_missing()
            .iter()
            .map(|entry| {
                format!(
                    "required runtime plugin {} is unavailable: {}",
                    entry.runtime_id.label(),
                    entry.reason
                )
            })
            .collect::<Vec<_>>()
            .join("; ")
    }

    pub fn fatal_messages(&self) -> Vec<String> {
        super::super::load_report::diagnostics::fatal_messages(
            &self.runtime_plugin_availability,
            &self.diagnostics,
        )
    }
}

impl fmt::Display for RuntimeModuleCompositionRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.fatal_messages().join("; "))
    }
}

impl Error for RuntimeModuleCompositionRejection {}

pub(in crate::builtin::runtime_modules) fn finish_runtime_module_composition(
    mut report: RuntimeModuleLoadReport,
    identity_seed: RuntimeModuleCompositionIdentitySeed,
) -> RuntimeModuleCompositionResult {
    if report.has_fatal_diagnostics() {
        return Err(reject(report));
    }
    let descriptors = report
        .modules
        .iter()
        .map(|module| module.descriptor())
        .collect::<Vec<_>>();
    let graph = match FrozenModuleGraph::freeze(&descriptors) {
        Ok(graph) => graph,
        Err(error) => {
            report.push_diagnostic(RuntimeModuleLoadDiagnostic::Core(error));
            return Err(reject(report));
        }
    };
    let mut entries_by_name = std::mem::take(&mut report.modules)
        .into_iter()
        .zip(descriptors)
        .map(|(module, descriptor)| (descriptor.name.clone(), (module, descriptor)))
        .collect::<HashMap<_, _>>();
    let entries = match graph
        .module_activation_order()
        .iter()
        .map(|name| {
            entries_by_name
                .remove(name)
                .ok_or_else(|| CoreError::MissingModule(name.clone()))
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(entries) => entries,
        Err(error) => {
            report.push_diagnostic(RuntimeModuleLoadDiagnostic::Core(error));
            return Err(reject(report));
        }
    };
    let (modules, descriptors) = entries.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();
    let identity = identity_seed.finish(&descriptors);
    Ok(RuntimeModuleCompositionPlan {
        modules,
        descriptors,
        runtime_plugin_availability: report.runtime_plugin_availability,
        diagnostics: report.diagnostics,
        identity,
    })
}

fn reject(report: RuntimeModuleLoadReport) -> RuntimeModuleCompositionRejection {
    RuntimeModuleCompositionRejection {
        runtime_plugin_availability: report.runtime_plugin_availability,
        diagnostics: report.diagnostics,
    }
}
