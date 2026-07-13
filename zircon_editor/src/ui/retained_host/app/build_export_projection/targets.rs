use crate::core::export::ExportPresetStore;
use crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::RetainedEditorHost;

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

    let export_dir = project_root.join("export");
    let mut preset_names = match std::fs::read_dir(&export_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                (path.extension().and_then(|value| value.to_str()) == Some("zpreset"))
                    .then(|| path.file_stem()?.to_str().map(str::to_owned))
                    .flatten()
            })
            .collect::<Vec<_>>(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => {
            diagnostics.push(format!("failed to enumerate export presets: {error}"));
            Vec::new()
        }
    };
    preset_names.sort();
    let store = ExportPresetStore::new(&project_root);
    let job_snapshots = host.desktop_export_jobs.snapshots();
    preset_names
        .into_iter()
        .filter_map(|preset_name| {
            let preset = match store.load(&preset_name) {
                Ok(preset) => preset,
                Err(error) => {
                    diagnostics.push(format!("export preset `{preset_name}` is invalid: {error}"));
                    return None;
                }
            };
            let Some(profile) = manifest
                .export_profiles
                .iter()
                .find(|profile| profile.name == preset.profile_ref)
                .cloned()
            else {
                diagnostics.push(format!(
                    "export preset `{preset_name}` references unknown profile `{}`",
                    preset.profile_ref
                ));
                return None;
            };
            Some(rows::build_export_target_for_preset(
                host,
                project_root.as_path(),
                &manifest,
                &preset_name,
                profile,
                job_snapshots.as_slice(),
            ))
        })
        .collect()
}
