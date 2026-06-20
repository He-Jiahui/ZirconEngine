use crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::{build_export_actions, RetainedEditorHost};

mod diagnostics;
mod project;
mod rows;

pub(super) fn build_export_targets(
    host: &RetainedEditorHost,
    chrome: &EditorChromeSnapshot,
    diagnostics: &mut Vec<String>,
) -> Vec<BuildExportTargetViewData> {
    let (project_root, manifest) = match project::load_active_project_manifest(&chrome.project_path)
    {
        Ok(project) => project,
        Err(error) => {
            diagnostics.push(error);
            return Vec::new();
        }
    };

    let job_snapshots = host.desktop_export_jobs.snapshots();
    build_export_actions::desktop_export_profiles()
        .into_iter()
        .map(|profile| {
            rows::build_export_target_for_profile(
                host,
                project_root.as_path(),
                &manifest,
                profile,
                job_snapshots.as_slice(),
            )
        })
        .collect()
}
