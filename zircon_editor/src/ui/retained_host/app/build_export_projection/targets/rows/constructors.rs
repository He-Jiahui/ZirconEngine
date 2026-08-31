use crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData;
use zircon_runtime::core::framework::project::{ExportPackagingStrategy, ExportProfile};
use zircon_runtime::plugin::ExportBuildPlan;

use super::super::super::super::build_export_actions;

pub(super) fn target_from_export_plan(plan: ExportBuildPlan) -> BuildExportTargetViewData {
    let has_fatal_diagnostics = plan.has_fatal_diagnostics();
    let profile_name = plan.profile.name.clone();
    let diagnostics = diagnostic_summary(&plan.fatal_diagnostics, &plan.diagnostics);

    BuildExportTargetViewData {
        preset_name: profile_name.clone().into(),
        profile_name: profile_name.into(),
        platform: build_export_actions::export_platform_label(plan.profile.target_platform).into(),
        target_mode: format!("{:?}", plan.profile.target_mode).into(),
        strategies: strategy_summary(&plan.profile.strategies).into(),
        status: if has_fatal_diagnostics {
            "Blocked".into()
        } else {
            "Ready".into()
        },
        enabled_plugins: plan.enabled_runtime_plugins.len().to_string().into(),
        linked_runtime_crates: plan.linked_runtime_crates.len().to_string().into(),
        native_dynamic_packages: plan.native_dynamic_packages.len().to_string().into(),
        generated_files: plan.generated_files.len().to_string().into(),
        diagnostics: diagnostics.into(),
        fatal: has_fatal_diagnostics,
    }
}

pub(super) fn blocked_target_from_profile(
    profile: &ExportProfile,
    error: impl ToString,
) -> BuildExportTargetViewData {
    BuildExportTargetViewData {
        profile_name: profile.name.clone().into(),
        platform: build_export_actions::export_platform_label(profile.target_platform).into(),
        target_mode: format!("{:?}", profile.target_mode).into(),
        strategies: strategy_summary(&profile.strategies).into(),
        status: "Blocked".into(),
        diagnostics: error.to_string().into(),
        fatal: true,
        ..BuildExportTargetViewData::default()
    }
}

fn diagnostic_summary(fatal_diagnostics: &[String], diagnostics: &[String]) -> String {
    let item_count = fatal_diagnostics.len() + diagnostics.len();
    let capacity = fatal_diagnostics
        .iter()
        .chain(diagnostics)
        .map(String::len)
        .sum::<usize>()
        + item_count.saturating_sub(1);
    let mut summary = String::with_capacity(capacity);
    for (index, diagnostic) in fatal_diagnostics.iter().chain(diagnostics).enumerate() {
        if index != 0 {
            summary.push('\n');
        }
        summary.push_str(diagnostic);
    }
    summary
}

fn strategy_summary(strategies: &[ExportPackagingStrategy]) -> String {
    const SEPARATOR: &str = ", ";

    let capacity = strategies
        .iter()
        .map(|strategy| packaging_strategy_label(strategy).len())
        .sum::<usize>()
        + strategies
            .len()
            .saturating_sub(1)
            .saturating_mul(SEPARATOR.len());
    let mut summary = String::with_capacity(capacity);
    for (index, strategy) in strategies.iter().enumerate() {
        if index != 0 {
            summary.push_str(SEPARATOR);
        }
        summary.push_str(packaging_strategy_label(strategy));
    }
    summary
}

fn packaging_strategy_label(strategy: &ExportPackagingStrategy) -> &'static str {
    match strategy {
        ExportPackagingStrategy::SourceTemplate => "SourceTemplate",
        ExportPackagingStrategy::LibraryEmbed => "LibraryEmbed",
        ExportPackagingStrategy::NativeDynamic => "NativeDynamic",
    }
}

#[cfg(test)]
#[path = "constructors/single_buffer_tests.rs"]
mod single_buffer_tests;
