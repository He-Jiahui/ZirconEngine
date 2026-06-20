use std::path::Path;

use crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData;
use zircon_runtime::plugin::{ExportBuildPlan, ExportProfile};

use super::super::super::super::{build_export_actions, RetainedEditorHost};
use super::super::diagnostics::prepend_desktop_export_output_diagnostic;

pub(super) fn target_from_export_plan(
    host: &RetainedEditorHost,
    project_root: &Path,
    plan: ExportBuildPlan,
) -> BuildExportTargetViewData {
    let has_fatal_diagnostics = plan.has_fatal_diagnostics();
    let profile_name = plan.profile.name.clone();
    let diagnostics = plan
        .fatal_diagnostics
        .iter()
        .chain(plan.diagnostics.iter())
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");
    let output_root = host.effective_desktop_export_output_root(project_root, &profile_name);
    let diagnostics = prepend_desktop_export_output_diagnostic(output_root.as_path(), diagnostics);

    BuildExportTargetViewData {
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
    host: &RetainedEditorHost,
    project_root: &Path,
    profile: &ExportProfile,
    error: impl ToString,
) -> BuildExportTargetViewData {
    let output_root = host.effective_desktop_export_output_root(project_root, &profile.name);
    let diagnostics =
        prepend_desktop_export_output_diagnostic(output_root.as_path(), error.to_string());
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
        diagnostics: diagnostics.into(),
        fatal: true,
        ..BuildExportTargetViewData::default()
    }
}
