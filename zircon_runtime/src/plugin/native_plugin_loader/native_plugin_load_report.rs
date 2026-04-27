use std::collections::BTreeMap;

use crate::{PluginPackageManifest, RuntimePluginRegistrationReport};

use super::{LoadedNativePlugin, NativePluginCandidate};

#[derive(Debug, Default)]
pub struct NativePluginLoadReport {
    pub discovered: Vec<NativePluginCandidate>,
    pub loaded: Vec<LoadedNativePlugin>,
    pub diagnostics: Vec<String>,
}

impl NativePluginLoadReport {
    pub fn has_failures(&self) -> bool {
        !self.diagnostics.is_empty()
    }

    pub fn package_manifests(&self) -> Vec<PluginPackageManifest> {
        let mut manifests = self
            .discovered
            .iter()
            .map(|candidate| {
                (
                    candidate.package_manifest.id.clone(),
                    candidate.package_manifest.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        for manifest in self.loaded.iter().filter_map(|plugin| {
            plugin
                .runtime_entry_report
                .as_ref()
                .and_then(|report| report.package_manifest.clone())
                .or_else(|| {
                    plugin
                        .editor_entry_report
                        .as_ref()
                        .and_then(|report| report.package_manifest.clone())
                })
                .or_else(|| {
                    plugin
                        .descriptor
                        .as_ref()
                        .and_then(|descriptor| descriptor.package_manifest.clone())
                })
        }) {
            manifests.insert(manifest.id.clone(), manifest);
        }
        manifests.into_values().collect()
    }

    pub fn runtime_plugin_registration_reports(&self) -> Vec<RuntimePluginRegistrationReport> {
        self.package_manifests()
            .into_iter()
            .map(|manifest| {
                let plugin_id = manifest.id.clone();
                let mut report =
                    RuntimePluginRegistrationReport::from_native_package_manifest(manifest);
                report
                    .diagnostics
                    .extend(self.diagnostics_for_plugin(&plugin_id));
                report.diagnostics.sort();
                report.diagnostics.dedup();
                report
            })
            .collect()
    }

    pub fn entry_diagnostics(&self) -> Vec<String> {
        self.loaded
            .iter()
            .flat_map(|plugin| {
                plugin
                    .runtime_entry_report
                    .iter()
                    .chain(plugin.editor_entry_report.iter())
                    .flat_map(|report| {
                        report
                            .diagnostics
                            .iter()
                            .map(|message| format!("native plugin {}: {message}", plugin.plugin_id))
                    })
            })
            .collect()
    }

    pub fn descriptor_diagnostics(&self) -> Vec<String> {
        self.loaded
            .iter()
            .filter(|plugin| plugin.descriptor.is_none())
            .map(|plugin| {
                format!(
                    "native plugin {} has no ABI descriptor attached",
                    plugin.plugin_id
                )
            })
            .collect()
    }

    pub fn diagnostics_for_plugin(&self, plugin_id: &str) -> Vec<String> {
        let mut diagnostics = self
            .diagnostics
            .iter()
            .filter(|message| diagnostic_mentions_plugin(message, plugin_id))
            .cloned()
            .collect::<Vec<_>>();
        diagnostics.extend(
            self.loaded
                .iter()
                .filter(|plugin| plugin.plugin_id == plugin_id && plugin.descriptor.is_none())
                .map(|plugin| {
                    format!(
                        "native plugin {} has no ABI descriptor attached",
                        plugin.plugin_id
                    )
                }),
        );
        diagnostics.extend(
            self.loaded
                .iter()
                .filter(|plugin| plugin.plugin_id == plugin_id)
                .flat_map(|plugin| {
                    plugin
                        .runtime_entry_report
                        .iter()
                        .chain(plugin.editor_entry_report.iter())
                        .flat_map(|report| {
                            report.diagnostics.iter().map(|message| {
                                format!("native plugin {}: {message}", plugin.plugin_id)
                            })
                        })
                }),
        );
        diagnostics
    }
}

fn diagnostic_mentions_plugin(message: &str, plugin_id: &str) -> bool {
    message.contains(&format!("native plugin {plugin_id} "))
        || message.contains(&format!("native plugin {plugin_id}:"))
}
