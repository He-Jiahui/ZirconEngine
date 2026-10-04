use std::collections::HashSet;

use super::editor_subsystems::EditorSubsystemReport;
use super::minimal_host_contract::EditorHostMinimalReport;

fn sorted_unique_capabilities(capabilities: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut capabilities = capabilities.into_iter().collect::<HashSet<_>>();
    let mut capabilities = capabilities.drain().collect::<Vec<_>>();
    capabilities.sort_unstable();
    capabilities
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorCapabilitySnapshot {
    enabled_capabilities: Vec<String>,
    disabled_capabilities: Vec<String>,
    diagnostics: Vec<String>,
}

impl EditorCapabilitySnapshot {
    pub(crate) fn from_reports(
        minimal: &EditorHostMinimalReport,
        subsystems: &EditorSubsystemReport,
    ) -> Self {
        let enabled_capabilities = sorted_unique_capabilities(
            minimal
                .loaded_capabilities()
                .into_iter()
                .chain(subsystems.enabled_subsystems().iter().cloned()),
        );

        Self {
            enabled_capabilities,
            disabled_capabilities: subsystems.disabled_subsystems().to_vec(),
            diagnostics: subsystems.diagnostics().to_vec(),
        }
    }

    pub fn enabled_capabilities(&self) -> &[String] {
        &self.enabled_capabilities
    }

    pub fn disabled_capabilities(&self) -> &[String] {
        &self.disabled_capabilities
    }

    pub fn diagnostics(&self) -> &[String] {
        &self.diagnostics
    }

    pub fn is_enabled(&self, capability: &str) -> bool {
        self.enabled_capabilities
            .binary_search_by(|enabled| enabled.as_str().cmp(capability))
            .is_ok()
    }

    pub(crate) fn allows_all(&self, capabilities: &[String]) -> bool {
        capabilities
            .iter()
            .all(|capability| self.is_enabled(capability))
    }
}

#[cfg(test)]
#[path = "tests/editor_capabilities.rs"]
mod tests;
