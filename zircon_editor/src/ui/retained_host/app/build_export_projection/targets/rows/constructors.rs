use crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData;
use zircon_runtime::core::framework::project::ExportProfile;
use zircon_runtime::plugin::ExportBuildPlan;

use super::super::super::super::build_export_actions;

pub(super) fn target_from_export_plan(plan: ExportBuildPlan) -> BuildExportTargetViewData {
    let has_fatal_diagnostics = plan.has_fatal_diagnostics();
    let profile_name = plan.profile.name.clone();
    let diagnostics = plan
        .fatal_diagnostics
        .iter()
        .chain(plan.diagnostics.iter())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");

    BuildExportTargetViewData {
        preset_name: profile_name.clone().into(),
        profile_name: profile_name.into(),
        platform: build_export_actions::export_platform_label(plan.profile.target_platform).into(),
        target_mode: format!("{:?}", plan.profile.target_mode).into(),
        strategies: plan
            .profile
            .strategies
            .iter()
            .map(|strategy| format!("{strategy:?}"))
            .collect::<Vec<_>>()
            .join(", ")
            .into(),
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
        strategies: profile
            .strategies
            .iter()
            .map(|strategy| format!("{strategy:?}"))
            .collect::<Vec<_>>()
            .join(", ")
            .into(),
        status: "Blocked".into(),
        diagnostics: error.to_string().into(),
        fatal: true,
        ..BuildExportTargetViewData::default()
    }
}
