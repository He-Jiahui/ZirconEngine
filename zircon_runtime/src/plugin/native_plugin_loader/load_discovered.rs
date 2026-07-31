use std::path::Path;

use libloading::Library;

use super::candidate_from_manifest::native_library_paths_for_candidate;
use super::compatibility::native_distribution_compatibility_diagnostic;
use super::native_plugin_abi::{
    call_native_plugin_entry, probe_native_plugin_descriptor, NativePluginDescriptor,
    NativePluginEntryReport,
};
use super::plugin_load_error::{
    PluginLoadError, PluginLoadResult, PluginLoadStage, ABI_CONTRACT_HINT,
};
use super::{LoadedNativePlugin, NativePluginLoadReport, NativePluginLoader};
use crate::{plugin::PluginModuleKind, plugin::PluginPackageManifest};

impl NativePluginLoader {
    pub fn load_discovered_all(&self, root: impl AsRef<Path>) -> NativePluginLoadReport {
        let report = self.discover(root);
        self.load_all_candidates(report)
    }

    pub fn load_discovered_runtime(&self, root: impl AsRef<Path>) -> NativePluginLoadReport {
        let report = self.discover(root);
        self.load_candidates_for_module_kinds(report, &[PluginModuleKind::Runtime])
    }

    pub fn load_discovered_editor(&self, root: impl AsRef<Path>) -> NativePluginLoadReport {
        let report = self.discover(root);
        self.load_candidates_for_module_kinds(report, &[PluginModuleKind::Editor])
    }

    pub(super) fn load_all_candidates(
        &self,
        report: NativePluginLoadReport,
    ) -> NativePluginLoadReport {
        self.load_candidates_for_module_kinds(
            report,
            &[PluginModuleKind::Runtime, PluginModuleKind::Editor],
        )
    }

    pub(super) fn load_candidates_for_module_kinds(
        &self,
        mut report: NativePluginLoadReport,
        module_kinds: &[PluginModuleKind],
    ) -> NativePluginLoadReport {
        let discovered = report.take_discovered();
        for candidate in &discovered {
            if !package_matches_module_kinds(&candidate.package_manifest, module_kinds) {
                continue;
            }
            if let Some(diagnostic) = native_distribution_compatibility_diagnostic(
                &candidate.plugin_id,
                &candidate.package_manifest,
            ) {
                report.push_diagnostic(diagnostic);
                continue;
            }
            for (library_path, library_module_kinds) in
                native_library_paths_for_candidate(&candidate, module_kinds)
            {
                load_candidate_library(
                    &mut report,
                    &candidate.plugin_id,
                    library_path,
                    &library_module_kinds,
                );
            }
        }
        report.restore_discovered(discovered);
        report
    }
}

fn load_candidate_library(
    report: &mut NativePluginLoadReport,
    plugin_id: &str,
    library_path: std::path::PathBuf,
    module_kinds: &[PluginModuleKind],
) {
    if !library_path.exists() {
        report.push_diagnostic(
            PluginLoadError::missing_artifact(plugin_id, &library_path, "native dist library")
                .to_string(),
        );
        return;
    }
    match unsafe { Library::new(&library_path) } {
        Ok(library) => {
            let descriptor =
                match unsafe { probe_native_plugin_descriptor(&library, &library_path, plugin_id) }
                {
                    Ok(descriptor) => descriptor,
                    Err(error) => {
                        report.push_diagnostic(error.to_string());
                        return;
                    }
                };
            let runtime_entry_report = retain_requested_entry(
                report,
                load_requested_entry(
                    &library,
                    &library_path,
                    plugin_id,
                    module_kinds,
                    PluginModuleKind::Runtime,
                    &descriptor,
                ),
            );
            let editor_entry_report = retain_requested_entry(
                report,
                load_requested_entry(
                    &library,
                    &library_path,
                    plugin_id,
                    module_kinds,
                    PluginModuleKind::Editor,
                    &descriptor,
                ),
            );
            report.push_loaded(LoadedNativePlugin {
                plugin_id: plugin_id.to_string(),
                library_path,
                descriptor: Some(descriptor),
                runtime_entry_report,
                editor_entry_report,
                library: LoadedNativePlugin::stable_library(library),
            });
        }
        Err(error) => report.push_diagnostic(
            PluginLoadError::library_open(plugin_id, &library_path, error).to_string(),
        ),
    }
}

fn retain_requested_entry<T>(
    report: &mut NativePluginLoadReport,
    entry: PluginLoadResult<Option<T>>,
) -> Option<T> {
    match entry {
        Ok(entry) => entry,
        Err(error) => {
            report.push_diagnostic(error.to_string());
            None
        }
    }
}

fn load_requested_entry(
    library: &Library,
    library_path: &Path,
    plugin_id: &str,
    module_kinds: &[PluginModuleKind],
    module_kind: PluginModuleKind,
    descriptor: &NativePluginDescriptor,
) -> PluginLoadResult<Option<NativePluginEntryReport>> {
    if !module_kinds.contains(&module_kind) {
        return Ok(None);
    }
    let entry_name = match module_kind {
        PluginModuleKind::Runtime => descriptor.runtime_entry_name.as_deref(),
        PluginModuleKind::Editor => descriptor.editor_entry_name.as_deref(),
        PluginModuleKind::Native | PluginModuleKind::Vm => return Ok(None),
    }
    .ok_or_else(|| {
        PluginLoadError::contract_mismatch(
            plugin_id,
            PluginLoadStage::from(module_kind),
            "descriptor.entry_symbol",
            "entry symbol name",
            "missing",
            library_path,
            ABI_CONTRACT_HINT,
        )
    })?;
    unsafe {
        call_native_plugin_entry(
            library,
            library_path,
            entry_name,
            plugin_id,
            module_kind,
            descriptor,
        )
    }
    .map(Some)
}

fn package_matches_module_kinds(
    package_manifest: &PluginPackageManifest,
    module_kinds: &[PluginModuleKind],
) -> bool {
    package_manifest
        .modules
        .iter()
        .any(|module| module_kinds.contains(&module.kind))
        || package_manifest
            .feature_extensions
            .iter()
            .flat_map(|feature| feature.modules.iter())
            .any(|module| module_kinds.contains(&module.kind))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::plugin::{PluginDistributionManifest, PluginModuleManifest, PluginPackageManifest};

    #[test]
    fn native_loader_skips_distribution_with_incompatible_engine_range_before_library_probe() {
        let package_manifest = PluginPackageManifest::new("future_native", "Future Native")
            .with_runtime_module(PluginModuleManifest::runtime(
                "future_native.runtime",
                "zircon_plugin_future_native_runtime",
            ))
            .with_distribution(PluginDistributionManifest {
                forms: vec!["dist".to_string()],
                abi_version: Some(super::super::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3),
                engine_compat: ">=99.0, <100.0".to_string(),
                dist_crate: "zircon_plugin_future_native_runtime".to_string(),
                ..PluginDistributionManifest::default()
            });
        let report =
            NativePluginLoadReport::from_discovered(vec![super::super::NativePluginCandidate {
                plugin_id: "future_native".to_string(),
                package_manifest,
                manifest_path: PathBuf::from("future_native/plugin.toml"),
                library_path: PathBuf::from("future_native/native/future_native.dll"),
            }]);

        let report = NativePluginLoader
            .load_candidates_for_module_kinds(report, &[PluginModuleKind::Runtime]);

        assert!(report.loaded().is_empty());
        assert!(report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("engine_compat")));
        assert!(!report
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("library is missing")));
    }

    #[test]
    fn entry_failure_preserves_successful_sibling_result() {
        let mut report = NativePluginLoadReport::default();

        let runtime_entry = retain_requested_entry(&mut report, Ok(Some("runtime entry")));
        let editor_entry: Option<&str> = retain_requested_entry(
            &mut report,
            Err(PluginLoadError::missing_artifact(
                "partial-entry-plugin",
                Path::new("partial-entry-plugin.dll"),
                "editor entry",
            )),
        );

        assert_eq!(runtime_entry, Some("runtime entry"));
        assert_eq!(editor_entry, None);
        assert!(report.diagnostics().iter().any(|diagnostic| {
            diagnostic.contains("partial-entry-plugin") && diagnostic.contains("editor entry")
        }));

        let source = include_str!("load_discovered.rs");
        let runtime_entry = source
            .find("let runtime_entry_report = retain_requested_entry")
            .expect("runtime result should be retained");
        let editor_entry = source
            .find("let editor_entry_report = retain_requested_entry")
            .expect("editor result should be retained");
        let loaded = source
            .find("report.push_loaded(LoadedNativePlugin")
            .expect("both entry results should be retained in the load report");
        assert!(runtime_entry < editor_entry && editor_entry < loaded);
    }

    #[test]
    fn candidate_loading_preserves_discovery_without_cloning_the_report() {
        let source = include_str!("load_discovered.rs");
        let deep_clone = ["report.discovered", ".clone()"].concat();

        assert!(!source.contains(&deep_clone));
        assert!(source.contains("report.take_discovered()"));
    }
}
