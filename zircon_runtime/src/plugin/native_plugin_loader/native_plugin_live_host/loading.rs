use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

use crate::plugin::PluginModuleKind;

use super::super::{LoadedNativePlugin, NativePluginLoadReport};
use super::diagnostics::{diagnostics_from_behavior_report, load_report_diagnostics};
use super::keys::{live_key, live_key_prefix, module_kind_label};
use super::reports::NativePluginLiveHostLoadReport;
use super::runtime_behavior::unload_behavior;
use super::NativePluginLiveHost;

impl NativePluginLiveHost {
    pub fn load_runtime_plugins_from_export_root(
        &self,
        export_root: impl AsRef<std::path::Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self.loader.load_runtime_from_load_manifest(export_root);
        self.load_reported_plugins(report, PluginModuleKind::Runtime)
    }

    pub fn load_editor_plugins_from_export_root(
        &self,
        export_root: impl AsRef<std::path::Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self.loader.load_editor_from_load_manifest(export_root);
        self.load_reported_plugins(report, PluginModuleKind::Editor)
    }

    pub fn load_runtime_plugins_from_project_root(
        &self,
        root: impl AsRef<std::path::Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self.loader.load_discovered_runtime(root);
        self.load_reported_plugins(report, PluginModuleKind::Runtime)
    }

    pub fn load_editor_plugins_from_project_root(
        &self,
        root: impl AsRef<std::path::Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self.loader.load_discovered_editor(root);
        self.load_reported_plugins(report, PluginModuleKind::Editor)
    }

    pub fn loaded_plugin_ids(&self, module_kind: PluginModuleKind) -> Result<Vec<String>, String> {
        let loaded = lock_loaded_native_plugins(&self.loaded)?;
        let prefix = live_key_prefix(module_kind);
        Ok(loaded
            .keys()
            .filter_map(|key| key.strip_prefix(prefix))
            .map(str::to_string)
            .collect())
    }

    fn load_reported_plugins(
        &self,
        mut report: NativePluginLoadReport,
        module_kind: PluginModuleKind,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let runtime_plugin_registration_reports = match module_kind {
            PluginModuleKind::Runtime => report.runtime_plugin_registration_reports(),
            PluginModuleKind::Editor | PluginModuleKind::Native | PluginModuleKind::Vm => {
                Vec::new()
            }
        };
        let runtime_plugin_feature_registration_reports = match module_kind {
            PluginModuleKind::Runtime => report.runtime_plugin_feature_registration_reports(),
            PluginModuleKind::Editor | PluginModuleKind::Native | PluginModuleKind::Vm => {
                Vec::new()
            }
        };
        let mut diagnostics = load_report_diagnostics(&report);
        let mut loaded = lock_loaded_native_plugins(&self.loaded)?;
        let mut loaded_plugin_ids = Vec::new();

        for plugin in std::mem::take(&mut report.loaded) {
            let plugin_id = plugin.plugin_id.clone();
            let key = live_key(module_kind, &plugin_id);
            if let Some(existing) = loaded.remove(&key) {
                match diagnostics_from_behavior_report(
                    &format!("{} unload before reload", module_kind_label(module_kind)),
                    unload_behavior(&existing, module_kind),
                ) {
                    Ok(unload_diagnostics) => diagnostics.extend(unload_diagnostics),
                    Err(error) => {
                        loaded.insert(key, existing);
                        return Err(error);
                    }
                }
            }
            loaded.insert(key, plugin);
            loaded_plugin_ids.push(plugin_id);
        }

        loaded_plugin_ids.sort();
        loaded_plugin_ids.dedup();
        diagnostics.sort();
        diagnostics.dedup();
        Ok(NativePluginLiveHostLoadReport {
            module_kind,
            loaded_plugin_ids,
            runtime_plugin_registration_reports,
            runtime_plugin_feature_registration_reports,
            diagnostics,
        })
    }
}

pub(super) fn lock_loaded_native_plugins(
    loaded: &Mutex<BTreeMap<String, LoadedNativePlugin>>,
) -> Result<MutexGuard<'_, BTreeMap<String, LoadedNativePlugin>>, String> {
    loaded
        .lock()
        .map_err(|_| "native plugin live host lock is poisoned".to_string())
}
