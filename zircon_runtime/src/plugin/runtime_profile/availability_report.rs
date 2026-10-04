//! 序列化友好的拥有型可用性报告，是启动结果、导出计划和诊断的交接格式。
//! 分类行与必需缺失行可重复指向同一插件，后者表示装配失败门槛。

use serde::{Deserialize, Serialize};

use crate::builtin::RuntimePluginId;
use crate::plugin::PluginMaturity;

const RUNTIME_PLUGIN_AVAILABILITY_CATEGORY_COUNT: usize = 8;

fn availability_diagnostic_line_count(report: &RuntimePluginAvailabilityReport) -> usize {
    [
        report.available.len(),
        report.linked.len(),
        report.native_dynamic.len(),
        report.externalized_missing.len(),
        report.stub.len(),
        report.blocked_by_target.len(),
        report.blocked_by_maturity.len(),
        report.missing_required.len(),
    ]
    .into_iter()
    .fold(
        RUNTIME_PLUGIN_AVAILABILITY_CATEGORY_COUNT,
        usize::saturating_add,
    )
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
/// 各类别保持选择顺序；调用方可直接查询，也可生成稳定顺序的诊断行。
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
/// 可用性主类别加必需缺失视图；后者不是独立的加载状态。
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
/// 对外拥有的单项快照，包含包身份、运行时身份、必需性和分类原因。
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

    /// 追加到调用方已有诊断流，保持类别顺序与现有机器可读键格式。
    pub fn push_diagnostic_lines(&self, lines: &mut Vec<String>) {
        lines.reserve(availability_diagnostic_line_count(self));
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

#[cfg(test)]
#[path = "availability_report/tests/capacity_tests.rs"]
mod capacity_tests;
