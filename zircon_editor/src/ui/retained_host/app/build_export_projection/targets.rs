use std::path::Path;

use crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData;
use crate::ui::workbench::project::project_root_path;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use zircon_runtime::asset::project::ProjectManifest;

use super::super::{build_export_actions, RetainedEditorHost};

pub(super) fn build_export_targets(
    host: &RetainedEditorHost,
    chrome: &EditorChromeSnapshot,
    diagnostics: &mut Vec<String>,
) -> Vec<BuildExportTargetViewData> {
    project_root_path(&chrome.project_path)
        .map_err(|error| error.to_string())
        .and_then(|project_root| {
            let manifest_path = project_root.join("zircon-project.toml");
            ProjectManifest::load(&manifest_path)
                .map_err(|error| format!("desktop export panel needs a project manifest: {error}"))
                .map(|manifest| (project_root, manifest))
        })
        .map(|(project_root, manifest)| {
            let job_snapshots = host.desktop_export_jobs.snapshots();
            build_export_actions::desktop_export_profiles()
                .into_iter()
                .map(|profile| {
                    let mut manifest_for_profile = manifest.clone();
                    manifest_for_profile.export_profiles.push(profile.clone());
                    match host.editor_manager.generate_native_aware_export_plan(
                        &project_root,
                        &manifest_for_profile,
                        &profile.name,
                    ) {
                        Ok(plan) => {
                            let has_fatal_diagnostics = plan.has_fatal_diagnostics();
                            let profile_name = plan.profile.name.clone();
                            let diagnostics = plan
                                .fatal_diagnostics
                                .iter()
                                .chain(plan.diagnostics.iter())
                                .cloned()
                                .collect::<Vec<_>>()
                                .join("\n");
                            let output_root = host
                                .effective_desktop_export_output_root(&project_root, &profile_name);
                            let diagnostics = prepend_desktop_export_output_diagnostic(
                                output_root.as_path(),
                                diagnostics,
                            );
                            let mut target = BuildExportTargetViewData {
                                profile_name: profile_name.clone().into(),
                                platform: build_export_actions::export_platform_label(
                                    plan.profile.target_platform,
                                )
                                .into(),
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
                                enabled_plugins: plan
                                    .enabled_runtime_plugins
                                    .len()
                                    .to_string()
                                    .into(),
                                linked_runtime_crates: plan
                                    .linked_runtime_crates
                                    .len()
                                    .to_string()
                                    .into(),
                                native_dynamic_packages: plan
                                    .native_dynamic_packages
                                    .len()
                                    .to_string()
                                    .into(),
                                generated_files: plan.generated_files.len().to_string().into(),
                                diagnostics: diagnostics.into(),
                                fatal: has_fatal_diagnostics,
                            };
                            if let Some(summary) =
                                host.desktop_export_reports.get(profile_name.as_str())
                            {
                                build_export_actions::apply_summary_to_target(&mut target, summary);
                            }
                            if let Some(job) = job_snapshots
                                .iter()
                                .find(|job| job.profile_name == profile_name)
                            {
                                build_export_actions::apply_job_snapshot_to_target(
                                    &mut target,
                                    job,
                                );
                            }
                            target
                        }
                        Err(error) => {
                            let output_root = host
                                .effective_desktop_export_output_root(&project_root, &profile.name);
                            let diagnostics = prepend_desktop_export_output_diagnostic(
                                output_root.as_path(),
                                error.to_string(),
                            );
                            let mut target = BuildExportTargetViewData {
                                profile_name: profile.name.clone().into(),
                                platform: build_export_actions::export_platform_label(
                                    profile.target_platform,
                                )
                                .into(),
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
                            };
                            if let Some(summary) =
                                host.desktop_export_reports.get(profile.name.as_str())
                            {
                                build_export_actions::apply_summary_to_target(&mut target, summary);
                            }
                            if let Some(job) = job_snapshots
                                .iter()
                                .find(|job| job.profile_name == profile.name)
                            {
                                build_export_actions::apply_job_snapshot_to_target(
                                    &mut target,
                                    job,
                                );
                            }
                            target
                        }
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|error| {
            diagnostics.push(error);
            Vec::new()
        })
}

fn prepend_desktop_export_output_diagnostic(
    output_root: &Path,
    diagnostics: impl Into<String>,
) -> String {
    let diagnostics = diagnostics.into();
    if diagnostics.is_empty() {
        format!("Output: {}", output_root.display())
    } else {
        format!("Output: {}\n{diagnostics}", output_root.display())
    }
}
