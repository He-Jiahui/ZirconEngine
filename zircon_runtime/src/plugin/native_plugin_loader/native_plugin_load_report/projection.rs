//! 一次性物化报告的清单、诊断和加载状态，避免多个编辑器视图重复扫描同一批插件。

use std::collections::HashMap;

use crate::plugin::native_plugin_loader::NativePluginCandidate;
use crate::plugin::{PluginModuleKind, PluginPackageManifest};

use super::NativePluginLoadReport;
use super::{
    diagnostics::mentioned_plugin_ids,
    manifests::{projected_package_manifests, shader_module_sources_from_candidate},
};
use crate::plugin::PluginShaderModuleSource;

#[cfg(test)]
#[path = "projection/tests/capacity_tests.rs"]
mod capacity_tests;

/// 报告当前代的只读索引；借用期间不能修改原报告，因而同一操作可共享一致的视图。
pub struct NativePluginLoadProjection {
    package_manifests: Vec<PluginPackageManifest>,
    shader_module_candidates_by_plugin: HashMap<String, NativePluginCandidate>,
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
    pub(super) manifest_package_index_lookups: usize,
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
    /// 从同一份报告建立包、shader 候选和诊断索引；仅报告拥有者在首次读取时调用。
    pub(super) fn new(report: &NativePluginLoadReport) -> Self {
        let mut stats = ProjectionBuildStats {
            projection_builds: 1,
            ..ProjectionBuildStats::default()
        };
        let package_manifests = projected_package_manifests(report, &mut stats);
        let shader_module_candidates_by_plugin = report
            .discovered
            .iter()
            .cloned()
            .map(|candidate| (candidate.plugin_id.clone(), candidate))
            .collect();
        stats.packages_projected = package_manifests.len();
        stats.features_projected = package_manifests
            .iter()
            .map(|manifest| manifest.optional_features.len() + manifest.feature_extensions.len())
            .sum();
        let diagnostics = project_diagnostics(report, &mut stats);
        Self {
            package_manifests,
            shader_module_candidates_by_plugin,
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

    /// 注册报告按插件请求时才读取包内 shader；无对应发现候选时不产生源码或文件诊断。
    pub(crate) fn shader_module_sources_for_plugin(
        &self,
        plugin_id: &str,
    ) -> (Vec<PluginShaderModuleSource>, Vec<String>) {
        self.shader_module_candidates_by_plugin
            .get(plugin_id)
            .map(shader_module_sources_from_candidate)
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

    /// 编辑器状态只判断本次报告是否包含已加载实例，不依赖是否存在描述符。
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

// 原始诊断先按文本中的插件 ID 分发，已加载实例再补齐描述符及入口诊断。
// 最终去重排序使 live host 和编辑器读取同一稳定结果。
fn project_diagnostics(
    report: &NativePluginLoadReport,
    stats: &mut ProjectionBuildStats,
) -> DiagnosticProjection {
    let mut diagnostics_by_plugin =
        HashMap::<String, PluginDiagnostics>::with_capacity(report.loaded.len());
    let mut descriptor_diagnostics = Vec::new();
    let mut entry_diagnostics = Vec::new();
    let mut loaded_plugins = HashMap::with_capacity(report.loaded.len());
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
