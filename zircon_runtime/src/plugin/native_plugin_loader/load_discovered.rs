use std::path::Path;

use libloading::Library;

use super::native_plugin_abi::{call_native_plugin_entry, probe_native_plugin_descriptor};
use super::{LoadedNativePlugin, NativePluginLoadReport, NativePluginLoader};
use crate::PluginModuleKind;

impl NativePluginLoader {
    pub fn load_discovered(&self, root: impl AsRef<Path>) -> NativePluginLoadReport {
        let report = self.discover(root);
        self.load_candidates(report)
    }

    pub(super) fn load_candidates(
        &self,
        mut report: NativePluginLoadReport,
    ) -> NativePluginLoadReport {
        for candidate in report.discovered.clone() {
            if !candidate.library_path.exists() {
                report.diagnostics.push(format!(
                    "native plugin {} skipped because library is missing: {}",
                    candidate.plugin_id,
                    candidate.library_path.display()
                ));
                continue;
            }
            match unsafe { Library::new(&candidate.library_path) } {
                Ok(library) => {
                    let descriptor = match unsafe { probe_native_plugin_descriptor(&library) } {
                        Ok(descriptor) => descriptor,
                        Err(error) => {
                            report.diagnostics.push(format!(
                                "native plugin {} loaded but ABI descriptor is invalid: {error}",
                                candidate.plugin_id
                            ));
                            None
                        }
                    };
                    if descriptor.is_none() {
                        report.diagnostics.push(format!(
                            "native plugin {} loaded without {} descriptor symbol",
                            candidate.plugin_id,
                            String::from_utf8_lossy(
                                super::ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL
                                    .strip_suffix(&[0])
                                    .unwrap_or(super::ZIRCON_NATIVE_PLUGIN_DESCRIPTOR_SYMBOL)
                            )
                        ));
                    }
                    let runtime_entry_report = descriptor
                        .as_ref()
                        .and_then(|descriptor| descriptor.runtime_entry_name.as_ref())
                        .and_then(|entry_name| {
                            match unsafe {
                                call_native_plugin_entry(
                                    &library,
                                    entry_name,
                                    &candidate.plugin_id,
                                    PluginModuleKind::Runtime,
                                )
                            } {
                                Ok(entry_report) => Some(entry_report),
                                Err(error) => {
                                    report.diagnostics.push(format!(
                                        "native plugin {} runtime entry failed: {error}",
                                        candidate.plugin_id
                                    ));
                                    None
                                }
                            }
                        });
                    let editor_entry_report = descriptor
                        .as_ref()
                        .and_then(|descriptor| descriptor.editor_entry_name.as_ref())
                        .and_then(|entry_name| {
                            match unsafe {
                                call_native_plugin_entry(
                                    &library,
                                    entry_name,
                                    &candidate.plugin_id,
                                    PluginModuleKind::Editor,
                                )
                            } {
                                Ok(entry_report) => Some(entry_report),
                                Err(error) => {
                                    report.diagnostics.push(format!(
                                        "native plugin {} editor entry failed: {error}",
                                        candidate.plugin_id
                                    ));
                                    None
                                }
                            }
                        });
                    report.loaded.push(LoadedNativePlugin {
                        plugin_id: candidate.plugin_id,
                        library_path: candidate.library_path,
                        descriptor,
                        runtime_entry_report,
                        editor_entry_report,
                        library,
                    });
                }
                Err(error) => report.diagnostics.push(format!(
                    "native plugin {} failed to load from {}: {error}",
                    candidate.plugin_id,
                    candidate.library_path.display()
                )),
            }
        }
        report
    }
}
