use serde::{Deserialize, Serialize};

use crate::builtin::RuntimePluginId;
use crate::plugin::PluginMaturity;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePluginAvailabilityReport {
    pub available: Vec<RuntimePluginAvailabilityEntry>,
    pub linked: Vec<RuntimePluginAvailabilityEntry>,
    pub native_dynamic: Vec<RuntimePluginAvailabilityEntry>,
    pub externalized_missing: Vec<RuntimePluginAvailabilityEntry>,
    pub stub: Vec<RuntimePluginAvailabilityEntry>,
    pub blocked_by_target: Vec<RuntimePluginAvailabilityEntry>,
    pub blocked_by_maturity: Vec<RuntimePluginAvailabilityEntry>,
    pub missing_required: Vec<RuntimePluginAvailabilityEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimePluginAvailabilityCategory {
    Available,
    Linked,
    NativeDynamic,
    ExternalizedMissing,
    Stub,
    BlockedByTarget,
    BlockedByMaturity,
    MissingRequired,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimePluginAvailabilityEntry {
    pub id: String,
    pub runtime_id: RuntimePluginId,
    pub required: bool,
    pub maturity: PluginMaturity,
    pub reason: String,
}

impl RuntimePluginAvailabilityReport {
    pub fn has_missing_required(&self) -> bool {
        !self.missing_required.is_empty()
    }

    pub fn entries(
        &self,
        category: RuntimePluginAvailabilityCategory,
    ) -> &[RuntimePluginAvailabilityEntry] {
        match category {
            RuntimePluginAvailabilityCategory::Available => &self.available,
            RuntimePluginAvailabilityCategory::Linked => &self.linked,
            RuntimePluginAvailabilityCategory::NativeDynamic => &self.native_dynamic,
            RuntimePluginAvailabilityCategory::ExternalizedMissing => &self.externalized_missing,
            RuntimePluginAvailabilityCategory::Stub => &self.stub,
            RuntimePluginAvailabilityCategory::BlockedByTarget => &self.blocked_by_target,
            RuntimePluginAvailabilityCategory::BlockedByMaturity => &self.blocked_by_maturity,
            RuntimePluginAvailabilityCategory::MissingRequired => &self.missing_required,
        }
    }

    pub fn category_count(&self, category: RuntimePluginAvailabilityCategory) -> usize {
        self.entries(category).len()
    }

    pub fn contains(
        &self,
        category: RuntimePluginAvailabilityCategory,
        runtime_id: RuntimePluginId,
    ) -> bool {
        self.entries(category)
            .iter()
            .any(|entry| entry.runtime_id == runtime_id)
    }

    pub fn entry_for(
        &self,
        category: RuntimePluginAvailabilityCategory,
        runtime_id: RuntimePluginId,
    ) -> Option<&RuntimePluginAvailabilityEntry> {
        self.entries(category)
            .iter()
            .find(|entry| entry.runtime_id == runtime_id)
    }

    pub fn diagnostic_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        self.push_diagnostic_lines(&mut lines);
        lines
    }

    pub fn push_diagnostic_lines(&self, lines: &mut Vec<String>) {
        push_availability_diagnostic_lines(lines, "available", &self.available);
        push_availability_diagnostic_lines(lines, "linked", &self.linked);
        push_availability_diagnostic_lines(lines, "native_dynamic", &self.native_dynamic);
        push_availability_diagnostic_lines(
            lines,
            "externalized_missing",
            &self.externalized_missing,
        );
        push_availability_diagnostic_lines(lines, "stub", &self.stub);
        push_availability_diagnostic_lines(lines, "blocked_by_target", &self.blocked_by_target);
        push_availability_diagnostic_lines(lines, "blocked_by_maturity", &self.blocked_by_maturity);
        push_availability_diagnostic_lines(lines, "missing_required", &self.missing_required);
    }
}

impl RuntimePluginAvailabilityCategory {
    pub fn key(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Linked => "linked",
            Self::NativeDynamic => "native_dynamic",
            Self::ExternalizedMissing => "externalized_missing",
            Self::Stub => "stub",
            Self::BlockedByTarget => "blocked_by_target",
            Self::BlockedByMaturity => "blocked_by_maturity",
            Self::MissingRequired => "missing_required",
        }
    }
}

fn push_availability_diagnostic_lines(
    lines: &mut Vec<String>,
    category: &str,
    entries: &[RuntimePluginAvailabilityEntry],
) {
    lines.push(format!(
        "runtime_plugin_availability.{category}.count={}",
        entries.len()
    ));
    lines.extend(entries.iter().map(|entry| {
        format!(
            "runtime_plugin_availability.{category}={} required={} maturity={:?} reason={}",
            entry.id, entry.required, entry.maturity, entry.reason
        )
    }));
}
