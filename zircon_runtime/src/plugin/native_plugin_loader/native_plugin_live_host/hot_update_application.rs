use std::path::Path;

use crate::asset::pack::ZrPackDeltaInstaller;
use crate::plugin::native::{NativePluginCandidate, NativePluginLoadReport};
use crate::plugin::PluginModuleKind;

use super::diagnostics::sorted_unique_diagnostics;
use super::reports::{
    NativePluginRuntimeDeltaHotUpdateReport, NativePluginRuntimeDeltaHotUpdateRequest,
    NativePluginRuntimeHotUpdateReport,
};
use super::NativePluginLiveHost;

impl NativePluginLiveHost {
    pub fn hot_reload_runtime_plugins_after_delta_pack_install(
        &self,
        request: NativePluginRuntimeDeltaHotUpdateRequest,
    ) -> Result<NativePluginRuntimeDeltaHotUpdateReport, String> {
        let pack_install = ZrPackDeltaInstaller::rebuild_to_staging(
            &request.base_pack,
            &request.delta_pack,
            &request.staged_pack,
        )
        .map_err(|error| format!("zrpack delta staging failed before hot update: {error}"))?;
        let pack_promotion = ZrPackDeltaInstaller::promote_staged_pack(
            &request.staged_pack,
            &request.installed_pack,
            request.backup_pack.as_ref(),
        )
        .map_err(|error| format!("zrpack delta promotion failed before hot update: {error}"))?;
        let pack_install_receipt = match request.receipt_path.as_ref() {
            Some(path) => Some(
                ZrPackDeltaInstaller::write_install_receipt(path, &pack_install, &pack_promotion)
                    .map_err(|error| {
                    format!("zrpack delta receipt write failed before hot update: {error}")
                })?,
            ),
            None => None,
        };
        let plugin_hot_update =
            self.hot_reload_runtime_plugins_from_export_root(&request.export_root)?;

        Ok(NativePluginRuntimeDeltaHotUpdateReport {
            pack_install,
            pack_promotion,
            pack_install_receipt,
            plugin_hot_update,
        })
    }

    pub fn hot_reload_runtime_plugins_from_export_root(
        &self,
        export_root: impl AsRef<Path>,
    ) -> Result<NativePluginRuntimeHotUpdateReport, String> {
        let export_root = export_root.as_ref();
        let discovery_report = self.loader.discover_from_load_manifest(export_root);
        let mut manifest_plugin_ids = discovery_report
            .discovered()
            .iter()
            .map(|candidate| candidate.plugin_id.clone())
            .collect::<Vec<_>>();
        let mut runtime_plugin_ids = Vec::new();
        let mut skipped_plugin_ids = Vec::new();
        let mut loaded_plugin_ids = Vec::new();
        let mut outcomes = Vec::new();
        let mut diagnostics = discovery_report.diagnostics().to_vec();

        let candidates = discovery_report.try_into_discovered().map_err(|report| {
            format!(
                "native runtime hot update expected a discovery-only report but found {} loaded plugin(s)",
                report.loaded().len()
            )
        })?;

        for candidate in candidates {
            if !native_candidate_has_module_kind(&candidate, PluginModuleKind::Runtime) {
                skipped_plugin_ids.push(candidate.plugin_id);
                continue;
            }
            let plugin_id = candidate.plugin_id.clone();
            runtime_plugin_ids.push(plugin_id.clone());
            let candidate_report = NativePluginLoadReport::from_discovered(vec![candidate]);
            let load_report = self
                .loader
                .load_candidates_for_module_kinds(candidate_report, &[PluginModuleKind::Runtime]);
            match self.hot_reload_reported_plugin(
                load_report,
                export_root,
                &plugin_id,
                PluginModuleKind::Runtime,
            ) {
                Ok(outcome) => {
                    loaded_plugin_ids.push(plugin_id);
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

#[cfg(test)]
mod tests {
    #[test]
    fn hot_update_moves_owned_candidates_into_the_single_plugin_load_report() {
        let source = include_str!("hot_update_application.rs");
        let deep_clone = ["discovered: vec![candidate", ".clone()]"].concat();

        assert!(!source.contains(&deep_clone));
        assert!(source.contains("NativePluginLoadReport::from_discovered(vec![candidate])"));
    }
}
