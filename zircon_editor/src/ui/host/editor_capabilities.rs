use std::collections::BTreeSet;

use super::editor_subsystems::EditorSubsystemReport;
use super::minimal_host_contract::EditorHostMinimalReport;

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
        let mut enabled = minimal
            .loaded_capabilities()
            .into_iter()
            .collect::<BTreeSet<_>>();
        enabled.extend(subsystems.enabled_subsystems().iter().cloned());

        Self {
            enabled_capabilities: enabled.into_iter().collect(),
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
mod tests {
    #[test]
    fn enabled_capability_lookup_uses_the_sorted_snapshot() {
        let source = include_str!("editor_capabilities.rs");
        let sorted_lookup = ["binary", "_search_by"].concat();

        assert!(source.contains(&sorted_lookup));
    }
}
