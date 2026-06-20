use std::path::Path;

use crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData;
use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::plugin::ExportProfile;

use super::super::super::{build_export_actions, RetainedEditorHost};

mod constructors;
mod overlays;

use constructors::{blocked_target_from_profile, target_from_export_plan};
use overlays::apply_target_overlays;

pub(super) fn build_export_target_for_profile(
    host: &RetainedEditorHost,
    project_root: &Path,
    manifest: &ProjectManifest,
    profile: ExportProfile,
    job_snapshots: &[build_export_actions::DesktopExportJobSnapshot],
) -> BuildExportTargetViewData {
    let mut manifest_for_profile = manifest.clone();
    manifest_for_profile.export_profiles.push(profile.clone());
    match host.editor_manager.generate_native_aware_export_plan(
        project_root,
        &manifest_for_profile,
        &profile.name,
    ) {
        Ok(plan) => {
            let profile_name = plan.profile.name.clone();
            let mut target = target_from_export_plan(host, project_root, plan);
            apply_target_overlays(host, profile_name.as_str(), job_snapshots, &mut target);
            target
        }
        Err(error) => {
            let mut target = blocked_target_from_profile(host, project_root, &profile, error);
            apply_target_overlays(host, profile.name.as_str(), job_snapshots, &mut target);
            target
        }
    }
}
