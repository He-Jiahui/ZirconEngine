use std::collections::HashMap;

use zircon_runtime::core::framework::project::ExportProfile;

use crate::core::export::ExportPresetStore;
use crate::ui::layouts::windows::workbench_host_window::BuildExportTargetViewData;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::RetainedEditorHost;
use super::cache::BuildExportBaseProjection;

mod diagnostics;
mod project;
mod rows;

pub(super) fn rebuild_export_targets(
    host: &RetainedEditorHost,
    chrome: &EditorChromeSnapshot,
) -> BuildExportBaseProjection {
    let (project_root, manifest) = match project::load_active_project_manifest(&chrome.project_path)
    {
        Ok(project) => project,
        Err(error) => return BuildExportBaseProjection::uncacheable(error),
    };

    let export_dir = project_root.join("export");
    let mut diagnostics = Vec::new();
    let mut preset_entries = match std::fs::read_dir(&export_dir) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let path = entry.path();
                (path.extension().and_then(|value| value.to_str()) == Some("zpreset"))
                    .then(|| Some((path.file_stem()?.to_str()?.to_owned(), path)))
                    .flatten()
            })
            .collect::<Vec<_>>(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => {
            diagnostics.push(format!("failed to enumerate export presets: {error}"));
            Vec::new()
        }
    };
    preset_entries.sort_by(|left, right| left.0.cmp(&right.0));
    let preset_paths = preset_entries
        .iter()
        .map(|(_, path)| path.clone())
        .collect::<Vec<_>>();
    let store = ExportPresetStore::new(&project_root);
    let export_profiles_by_name = export_profiles_by_name(&manifest.export_profiles);
    let targets = preset_entries
        .into_iter()
        .filter_map(|(preset_name, _)| {
            let preset = match store.load(&preset_name) {
                Ok(preset) => preset,
                Err(error) => {
                    diagnostics.push(format!("export preset `{preset_name}` is invalid: {error}"));
                    return None;
                }
            };
            let Some(profile) = export_profiles_by_name
                .get(preset.profile_ref.as_str())
                .copied()
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
            ))
        })
        .collect();
    BuildExportBaseProjection {
        project_root,
        targets,
        diagnostics,
        preset_paths,
        cacheable: true,
    }
}

pub(super) fn apply_export_target_overlays(
    host: &RetainedEditorHost,
    base: &BuildExportBaseProjection,
) -> Vec<BuildExportTargetViewData> {
    let job_snapshots = host.desktop_export_jobs.snapshots();
    let mut targets = base.targets.clone();
    for target in &mut targets {
        rows::apply_target_overlays(host, &base.project_root, &job_snapshots, target);
    }
    targets
}

fn export_profiles_by_name(profiles: &[ExportProfile]) -> HashMap<&str, &ExportProfile> {
    // EDITOR78_EXPORT_PROFILE_HASH_INDEX_BENCH_V1
    let mut by_name = HashMap::with_capacity(profiles.len());
    for profile in profiles {
        by_name.entry(profile.name.as_str()).or_insert(profile);
    }
    by_name
}

#[cfg(test)]
#[path = "targets/profile_index_tests.rs"]
mod profile_index_tests;
