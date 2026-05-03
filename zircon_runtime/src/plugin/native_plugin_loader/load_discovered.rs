use std::path::Path;

use libloading::Library;

use super::candidate_from_manifest::native_library_paths_for_candidate;
use super::native_plugin_abi::{call_native_plugin_entry, probe_native_plugin_descriptor};
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
        for candidate in report.discovered.clone() {
            if !package_matches_module_kinds(&candidate.package_manifest, module_kinds) {
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
        report.diagnostics.push(format!(
            "native plugin {plugin_id} skipped because library is missing: {}",
            library_path.display()
        ));
        return;
    }
    match unsafe { Library::new(&library_path) } {
        Ok(library) => {
            let descriptor = match unsafe { probe_native_plugin_descriptor(&library) } {
                Ok(descriptor) => descriptor,
                Err(error) => {
                    report.diagnostics.push(format!(
                        "native plugin {} loaded but ABI descriptor is invalid: {error}",
                        plugin_id
                    ));
                    None
                }
            };
            if descriptor.is_none() {
                report.diagnostics.push(format!(
                    "native plugin {} loaded without {} descriptor symbol",
                    plugin_id,
                    String::from_utf8_lossy(
                        super::ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL
                            .strip_suffix(&[0])
                            .unwrap_or(super::ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL)
                    )
                ));
            }
            let runtime_entry_report = if module_kinds.contains(&PluginModuleKind::Runtime) {
                descriptor.as_ref().and_then(|descriptor| {
                    descriptor
                        .runtime_entry_name
                        .as_ref()
                        .and_then(|entry_name| {
                            match unsafe {
                                call_native_plugin_entry(
                                    &library,
                                    entry_name,
                                    plugin_id,
                                    PluginModuleKind::Runtime,
                                    descriptor,
                                )
                            } {
                                Ok(entry_report) => Some(entry_report),
                                Err(error) => {
                                    report.diagnostics.push(format!(
                                        "native plugin {} runtime entry failed: {error}",
                                        plugin_id
                                    ));
                                    None
                                }
                            }
                        })
                })
            } else {
                None
            };
            let editor_entry_report = if module_kinds.contains(&PluginModuleKind::Editor) {
                descriptor.as_ref().and_then(|descriptor| {
                    descriptor
                        .editor_entry_name
                        .as_ref()
                        .and_then(|entry_name| {
                            match unsafe {
                                call_native_plugin_entry(
                                    &library,
                                    entry_name,
                                    plugin_id,
                                    PluginModuleKind::Editor,
                                    descriptor,
                                )
                            } {
                                Ok(entry_report) => Some(entry_report),
                                Err(error) => {
                                    report.diagnostics.push(format!(
                                        "native plugin {} editor entry failed: {error}",
                                        plugin_id
                                    ));
                                    None
                                }
                            }
                        })
                })
            } else {
                None
            };
            report.loaded.push(LoadedNativePlugin {
                plugin_id: plugin_id.to_string(),
                library_path,
                descriptor,
                runtime_entry_report,
                editor_entry_report,
                library,
            });
        }
        Err(error) => report.diagnostics.push(format!(
            "native plugin {} failed to load from {}: {error}",
            plugin_id,
            library_path.display()
        )),
    }
}

fn package_matches_module_kinds(
    package_manifest: &PluginPackageManifest,
    module_kinds: &[PluginModuleKind],
) -> bool {
    package_manifest
        .modules
        .iter()
        .any(|module| module_kinds.contains(&module.kind))
}
