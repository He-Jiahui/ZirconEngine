use std::path::Path;

use crate::plugin::{NativePluginCandidate, NativePluginLoadReport, PluginModuleKind};

use super::diagnostics::sorted_unique_diagnostics;
use super::reports::NativePluginRuntimeHotUpdateReport;
use super::NativePluginLiveHost;

impl NativePluginLiveHost {
    pub fn hot_reload_runtime_plugins_from_export_root(
        &self,
        export_root: impl AsRef<Path>,
    ) -> Result<NativePluginRuntimeHotUpdateReport, String> {
        let export_root = export_root.as_ref();
        let discovery_report = self.loader.discover_from_load_manifest(export_root);
        let mut manifest_plugin_ids = discovery_report
            .discovered
            .iter()
            .map(|candidate| candidate.plugin_id.clone())
            .collect::<Vec<_>>();
        let mut runtime_plugin_ids = Vec::new();
        let mut skipped_plugin_ids = Vec::new();
        let mut loaded_plugin_ids = Vec::new();
        let mut outcomes = Vec::new();
        let mut diagnostics = discovery_report.diagnostics;

        for candidate in discovery_report.discovered {
            if !native_candidate_has_module_kind(&candidate, PluginModuleKind::Runtime) {
                skipped_plugin_ids.push(candidate.plugin_id);
                continue;
            }
            runtime_plugin_ids.push(candidate.plugin_id.clone());
            let load_report = self.loader.load_candidates_for_module_kinds(
                NativePluginLoadReport {
                    discovered: vec![candidate.clone()],
                    loaded: Vec::new(),
                    diagnostics: Vec::new(),
                },
                &[PluginModuleKind::Runtime],
            );
            match self.hot_reload_reported_plugin(
                load_report,
                export_root,
                &candidate.plugin_id,
                PluginModuleKind::Runtime,
            ) {
                Ok(outcome) => {
                    loaded_plugin_ids.push(candidate.plugin_id);
                    diagnostics.extend(outcome.diagnostics.clone());
                    outcomes.push(outcome);
                }
                Err(error) => diagnostics.push(error),
            }
        }

        if !skipped_plugin_ids.is_empty() {
            diagnostics.push(format!(
                "native runtime hot update skipped non-runtime plugin package(s): {}",
                skipped_plugin_ids.join(", ")
            ));
        }

        manifest_plugin_ids.sort();
        manifest_plugin_ids.dedup();
        runtime_plugin_ids.sort();
        runtime_plugin_ids.dedup();
        skipped_plugin_ids.sort();
        skipped_plugin_ids.dedup();
        loaded_plugin_ids.sort();
        loaded_plugin_ids.dedup();
        outcomes.sort_by(|left, right| left.plugin_id.cmp(&right.plugin_id));
        diagnostics = sorted_unique_diagnostics(diagnostics);

        Ok(NativePluginRuntimeHotUpdateReport {
            export_root: export_root.to_path_buf(),
            manifest_plugin_ids,
            runtime_plugin_ids,
            loaded_plugin_ids,
            skipped_plugin_ids,
            outcomes,
            diagnostics,
        })
    }
}

fn native_candidate_has_module_kind(
    candidate: &NativePluginCandidate,
    module_kind: PluginModuleKind,
) -> bool {
    candidate
        .package_manifest
        .modules
        .iter()
        .any(|module| module.kind == module_kind)
        || candidate
            .package_manifest
            .feature_extensions
            .iter()
            .flat_map(|feature| feature.modules.iter())
            .any(|module| module.kind == module_kind)
}
