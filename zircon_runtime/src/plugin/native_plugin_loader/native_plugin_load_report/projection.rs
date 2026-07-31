use std::collections::HashMap;

use crate::plugin::{PluginModuleKind, PluginPackageManifest};

use super::NativePluginLoadReport;
use super::{diagnostics::mentioned_plugin_ids, manifests::projected_package_manifests};

pub struct NativePluginLoadProjection {
    package_manifests: Vec<PluginPackageManifest>,
    diagnostics_by_plugin: HashMap<String, PluginDiagnostics>,
    descriptor_diagnostics: Vec<String>,
    entry_diagnostics: Vec<String>,
    loaded_plugins: HashMap<String, LoadedPluginState>,
    #[cfg(test)]
    stats: ProjectionBuildStats,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ProjectionBuildStats {
    pub(super) projection_builds: usize,
    pub(super) manifest_sources_scanned: usize,
    pub(super) packages_projected: usize,
    pub(super) features_projected: usize,
    pub(super) loaded_plugins_scanned: usize,
    pub(super) raw_diagnostics_scanned: usize,
}

#[derive(Default)]
struct PluginDiagnostics {
    all: Vec<String>,
    runtime: Vec<String>,
    editor: Vec<String>,
}

#[derive(Clone, Copy)]
struct LoadedPluginState {
    has_descriptor: bool,
}

impl NativePluginLoadProjection {
    pub(super) fn new(report: &NativePluginLoadReport) -> Self {
        let mut stats = ProjectionBuildStats {
            projection_builds: 1,
            ..ProjectionBuildStats::default()
        };
        let package_manifests = projected_package_manifests(report, &mut stats);
        stats.packages_projected = package_manifests.len();
        stats.features_projected = package_manifests
            .iter()
            .map(|manifest| manifest.optional_features.len() + manifest.feature_extensions.len())
            .sum();
        let diagnostics = project_diagnostics(report, &mut stats);
        Self {
            package_manifests,
            diagnostics_by_plugin: diagnostics.by_plugin,
            descriptor_diagnostics: diagnostics.descriptor,
            entry_diagnostics: diagnostics.entry,
            loaded_plugins: diagnostics.loaded_plugins,
            #[cfg(test)]
            stats,
        }
    }

    pub fn package_manifests(&self) -> &[PluginPackageManifest] {
        &self.package_manifests
    }

    pub fn runtime_diagnostics_for_plugin(&self, plugin_id: &str) -> Vec<String> {
        self.diagnostics_by_plugin
            .get(plugin_id)
            .map(|diagnostics| diagnostics.runtime.clone())
            .unwrap_or_default()
    }

    pub fn diagnostics_for_plugin(&self, plugin_id: &str) -> Vec<String> {
        self.diagnostics_by_plugin
            .get(plugin_id)
            .map(|diagnostics| diagnostics.all.clone())
            .unwrap_or_default()
    }

    pub fn editor_diagnostics_for_plugin(&self, plugin_id: &str) -> Vec<String> {
        self.diagnostics_by_plugin
            .get(plugin_id)
            .map(|diagnostics| diagnostics.editor.clone())
            .unwrap_or_default()
    }

    pub fn descriptor_diagnostics(&self) -> &[String] {
        &self.descriptor_diagnostics
    }

    pub fn entry_diagnostics(&self) -> &[String] {
        &self.entry_diagnostics
    }

    pub fn is_loaded(&self, plugin_id: &str) -> bool {
        self.loaded_plugins.contains_key(plugin_id)
    }

    pub fn has_descriptor(&self, plugin_id: &str) -> bool {
        self.loaded_plugins
            .get(plugin_id)
            .is_some_and(|state| state.has_descriptor)
    }

    #[cfg(test)]
    pub(super) fn stats(&self) -> ProjectionBuildStats {
        self.stats
    }
}

struct DiagnosticProjection {
    by_plugin: HashMap<String, PluginDiagnostics>,
    descriptor: Vec<String>,
    entry: Vec<String>,
    loaded_plugins: HashMap<String, LoadedPluginState>,
}

fn project_diagnostics(
    report: &NativePluginLoadReport,
    stats: &mut ProjectionBuildStats,
) -> DiagnosticProjection {
    let mut diagnostics_by_plugin = HashMap::<String, PluginDiagnostics>::new();
    let mut descriptor_diagnostics = Vec::new();
    let mut entry_diagnostics = Vec::new();
    let mut loaded_plugins = HashMap::new();
    for message in &report.diagnostics {
        stats.raw_diagnostics_scanned += 1;
        for plugin_id in mentioned_plugin_ids(message) {
            let diagnostics = diagnostics_by_plugin
                .entry(plugin_id.to_string())
                .or_default();
            diagnostics.all.push(message.clone());
            diagnostics.runtime.push(message.clone());
            diagnostics.editor.push(message.clone());
        }
    }

    for plugin in &report.loaded {
        stats.loaded_plugins_scanned += 1;
        loaded_plugins
            .entry(plugin.plugin_id.clone())
            .and_modify(|state: &mut LoadedPluginState| {
                state.has_descriptor &= plugin.descriptor.is_some();
            })
            .or_insert(LoadedPluginState {
                has_descriptor: plugin.descriptor.is_some(),
            });
        let diagnostics = diagnostics_by_plugin
            .entry(plugin.plugin_id.clone())
            .or_default();
        if plugin.descriptor.is_none() {
            let message = format!(
                "native plugin {} has no ABI descriptor attached",
                plugin.plugin_id
            );
            diagnostics.all.push(message.clone());
            diagnostics.runtime.push(message.clone());
            diagnostics.editor.push(message.clone());
            descriptor_diagnostics.push(message);
        }
        for entry in plugin
            .runtime_entry_report
            .iter()
            .chain(plugin.editor_entry_report.iter())
        {
            for message in entry
                .diagnostics
                .iter()
                .chain(entry.behavior_validation.diagnostics.iter())
            {
                let message = format!("native plugin {}: {message}", plugin.plugin_id);
                entry_diagnostics.push(message.clone());
                diagnostics.all.push(message.clone());
                match entry.module_kind {
                    PluginModuleKind::Runtime => diagnostics.runtime.push(message),
                    PluginModuleKind::Editor => diagnostics.editor.push(message),
                    PluginModuleKind::Native | PluginModuleKind::Vm => {}
                }
            }
        }
    }

    for diagnostics in diagnostics_by_plugin.values_mut() {
        sort_dedup(&mut diagnostics.all);
        sort_dedup(&mut diagnostics.runtime);
        sort_dedup(&mut diagnostics.editor);
    }
    sort_dedup(&mut descriptor_diagnostics);
    sort_dedup(&mut entry_diagnostics);
    DiagnosticProjection {
        by_plugin: diagnostics_by_plugin,
        descriptor: descriptor_diagnostics,
        entry: entry_diagnostics,
        loaded_plugins,
    }
}

fn sort_dedup(values: &mut Vec<String>) {
    values.sort();
    values.dedup();
}
