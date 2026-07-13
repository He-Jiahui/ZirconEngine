use crate::asset::AssetImporterRegistryError;
use crate::core::CoreError;
use crate::plugin::{RuntimePluginAvailabilityEntry, RuntimePluginFeatureBlock};

use super::report::RuntimeModuleLoadReport;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeModuleLoadDiagnostic {
    Core(CoreError),
    UnknownPlugin { id: String, required: bool },
    FeatureBlocked(RuntimePluginFeatureBlock),
    FeatureDefinition(String),
    AssetImporter(AssetImporterRegistryError),
}

impl RuntimeModuleLoadDiagnostic {
    pub fn is_fatal(&self) -> bool {
        match self {
            Self::Core(_) | Self::FeatureDefinition(_) | Self::AssetImporter(_) => true,
            Self::UnknownPlugin { required, .. } => *required,
            Self::FeatureBlocked(blocked) => blocked.required,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Core(error) => error.to_string(),
            Self::UnknownPlugin { id, .. } => {
                format!("plugin {id} has no known runtime id")
            }
            Self::FeatureBlocked(blocked) => blocked.to_diagnostic(),
            Self::FeatureDefinition(diagnostic) => diagnostic.clone(),
            Self::AssetImporter(error) => {
                format!("asset importer registration failed: {error}")
            }
        }
    }
}

impl RuntimeModuleLoadReport {
    pub fn required_missing(&self) -> &[RuntimePluginAvailabilityEntry] {
        &self.runtime_plugin_availability.missing_required
    }

    pub fn required_missing_summary(&self) -> String {
        self.required_missing()
            .iter()
            .map(required_plugin_missing_message)
            .collect::<Vec<_>>()
            .join("; ")
    }

    pub fn warning_messages(&self) -> Vec<String> {
        optional_unavailable_entries(&self.runtime_plugin_availability)
            .map(optional_plugin_unavailable_message)
            .chain(
                self.diagnostics()
                    .iter()
                    .filter(|diagnostic| !diagnostic.is_fatal())
                    .map(RuntimeModuleLoadDiagnostic::message),
            )
            .collect()
    }

    pub fn fatal_messages(&self) -> Vec<String> {
        self.required_missing()
            .iter()
            .map(required_plugin_missing_message)
            .chain(
                self.diagnostics()
                    .iter()
                    .filter(|diagnostic| diagnostic.is_fatal())
                    .map(RuntimeModuleLoadDiagnostic::message),
            )
            .collect()
    }

    pub fn has_fatal_diagnostics(&self) -> bool {
        !self.required_missing().is_empty()
            || self
                .diagnostics()
                .iter()
                .any(RuntimeModuleLoadDiagnostic::is_fatal)
    }
}

fn required_plugin_missing_message(entry: &RuntimePluginAvailabilityEntry) -> String {
    format!(
        "required runtime plugin {} is unavailable: {}",
        entry.runtime_id.label(),
        entry.reason
    )
}

fn optional_unavailable_entries<'a>(
    availability: &'a crate::plugin::RuntimePluginAvailabilityReport,
) -> impl Iterator<Item = &'a RuntimePluginAvailabilityEntry> + 'a {
    availability
        .externalized_missing
        .iter()
        .chain(&availability.stub)
        .chain(&availability.blocked_by_target)
        .chain(&availability.blocked_by_maturity)
        .filter(|entry| !entry.required)
}

fn optional_plugin_unavailable_message(entry: &RuntimePluginAvailabilityEntry) -> String {
    format!(
        "optional runtime plugin {} is unavailable: {}",
        entry.runtime_id.label(),
        entry.reason
    )
}
