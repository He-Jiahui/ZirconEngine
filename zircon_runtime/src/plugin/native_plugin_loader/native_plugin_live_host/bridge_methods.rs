use crate::plugin::native::{
    native_bridge_method_descriptors_from_manifest, NativeBridgeMethodBinding,
    NativeHostBridgeCallScope,
};
use crate::plugin::{
    PluginModuleKind, PluginPackageManifest, RuntimePluginBridgeLifecycleEvent,
    RuntimePluginBridgeLifecycleState,
};

use super::diagnostics::unloaded_plugin_error;
use super::keys::live_key;
use super::loading::lock_loaded_native_plugins;
use super::reports::{
    NativePluginLiveHostBridgeLifecycleReport, NativePluginLiveHostBridgeReloadReport,
    NativePluginLiveHostCommand,
};
use super::NativePluginLiveHost;

impl NativePluginLiveHost {
    pub fn install_discovered_runtime_bridge_method_bindings(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<usize, String> {
        let plugin_id = plugin_id.as_ref();
        let loaded = lock_loaded_native_plugins(&self.loaded)?;
        let plugin = loaded
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .ok_or_else(|| unloaded_plugin_error(plugin_id, PluginModuleKind::Runtime))?;
        let bindings = discovered_runtime_bridge_method_bindings(plugin)?.ok_or_else(|| {
            format!("runtime plugin {plugin_id} exposes no native bridge method table")
        })?;
        let binding_count = bindings.len();
        drop(loaded);
        self.replace_runtime_bridge_method_bindings(plugin_id, Some(bindings))?;
        Ok(binding_count)
    }

    pub fn install_runtime_bridge_method_bindings(
        &self,
        plugin_id: impl AsRef<str>,
        bindings: impl IntoIterator<Item = NativeBridgeMethodBinding>,
    ) -> Result<(), String> {
        let plugin_id = plugin_id.as_ref();
        let bindings = bindings.into_iter().collect::<Vec<_>>();
        let manifest = self.loaded_runtime_package_manifest_required(plugin_id)?;
        native_bridge_method_descriptors_from_manifest(&manifest, bindings.clone())
            .map_err(|error| error.to_string())?;

        self.replace_runtime_bridge_method_bindings(plugin_id, Some(bindings))?;
        Ok(())
    }

    pub fn clear_runtime_bridge_method_bindings(
        &self,
        plugin_id: impl AsRef<str>,
    ) -> Result<bool, String> {
        self.runtime_bridge_method_bindings
            .lock()
            .map_err(|_| "native live host bridge method bindings lock poisoned".to_string())
            .map(|mut bindings| bindings.remove(plugin_id.as_ref()).is_some())
    }

    pub fn runtime_bridge_call_scope_from_loaded_manifest(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
        bindings: impl IntoIterator<Item = NativeBridgeMethodBinding>,
    ) -> Result<NativeHostBridgeCallScope, String> {
        let plugin_id = plugin_id.as_ref();
        let manifest = self.loaded_runtime_package_manifest_required(plugin_id)?;
        let descriptors = native_bridge_method_descriptors_from_manifest(&manifest, bindings)
            .map_err(|error| error.to_string())?;
        NativeHostBridgeCallScope::from_method_descriptors(
            lifecycle.bridge_table().clone(),
            descriptors,
        )
        .map_err(|error| error.to_string())
    }

    pub fn runtime_bridge_call_scope_from_installed_bindings(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativeHostBridgeCallScope, String> {
        let plugin_id = plugin_id.as_ref();
        let bindings = self.installed_runtime_bridge_method_bindings(plugin_id)?;
        self.runtime_bridge_call_scope_from_loaded_manifest(plugin_id, lifecycle, bindings)
    }

    pub fn reload_runtime_bridge_provider_and_scope_from_installed_bindings(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostBridgeReloadReport, String> {
        let plugin_id = plugin_id.as_ref();
        let event = RuntimePluginBridgeLifecycleEvent::reload_provider(plugin_id);
        let bridge_lifecycle_report = NativePluginLiveHostBridgeLifecycleReport {
            plugin_id: plugin_id.to_string(),
            module_kind: PluginModuleKind::Runtime,
            command: NativePluginLiveHostCommand::HotReload,
            event: event.clone(),
            outcome: lifecycle.apply_provider_lifecycle_event(event),
        };
        if !bridge_lifecycle_report.is_applied() {
            return Err(bridge_lifecycle_report.diagnostic());
        }

        let bridge_call_scope =
            self.runtime_bridge_call_scope_from_installed_bindings(plugin_id, lifecycle)?;
        let mut report = NativePluginLiveHostBridgeReloadReport {
            plugin_id: plugin_id.to_string(),
            module_kind: PluginModuleKind::Runtime,
            command: NativePluginLiveHostCommand::HotReload,
            bridge_lifecycle_report,
            bridge_call_scope,
            diagnostics: Vec::new(),
        };
        report
            .diagnostics
            .push(report.bridge_lifecycle_report.diagnostic());
        report.diagnostics.push(report.diagnostic());
        report.diagnostics.sort();
        report.diagnostics.dedup();
        Ok(report)
    }

    fn loaded_runtime_package_manifest_required(
        &self,
        plugin_id: &str,
    ) -> Result<PluginPackageManifest, String> {
        self.loaded_runtime_package_manifest(plugin_id)?
            .ok_or_else(|| format!("runtime plugin {plugin_id} has no package manifest"))
    }

    fn installed_runtime_bridge_method_bindings(
        &self,
        plugin_id: &str,
    ) -> Result<Vec<NativeBridgeMethodBinding>, String> {
        self.runtime_bridge_method_bindings
            .lock()
            .map_err(|_| "native live host bridge method bindings lock poisoned".to_string())?
            .get(plugin_id)
            .cloned()
            .ok_or_else(|| {
                format!("runtime plugin {plugin_id} has no installed native bridge method bindings")
            })
    }

    pub(super) fn replace_runtime_bridge_method_bindings(
        &self,
        plugin_id: &str,
        bindings: Option<Vec<NativeBridgeMethodBinding>>,
    ) -> Result<(), String> {
        let mut installed_bindings = self
            .runtime_bridge_method_bindings
            .lock()
            .map_err(|_| "native live host bridge method bindings lock poisoned".to_string())?;
        match bindings {
            Some(bindings) if !bindings.is_empty() => {
                installed_bindings.insert(plugin_id.to_string(), bindings);
            }
            Some(_) | None => {
                installed_bindings.remove(plugin_id);
            }
        }
        Ok(())
    }

    fn loaded_runtime_package_manifest(
        &self,
        plugin_id: &str,
    ) -> Result<Option<PluginPackageManifest>, String> {
        let loaded = lock_loaded_native_plugins(&self.loaded)?;
        let plugin = loaded
            .get(&live_key(PluginModuleKind::Runtime, plugin_id))
            .ok_or_else(|| unloaded_plugin_error(plugin_id, PluginModuleKind::Runtime))?;
        Ok(runtime_package_manifest(plugin).cloned())
    }
}

pub(super) fn discovered_runtime_bridge_method_bindings(
    plugin: &super::super::LoadedNativePlugin,
) -> Result<Option<Vec<NativeBridgeMethodBinding>>, String> {
    let Some(report) = plugin.runtime_entry_report.as_ref() else {
        return Ok(None);
    };
    if report.bridge_method_bindings.is_empty() {
        return Ok(None);
    }
    let manifest = runtime_package_manifest(plugin).ok_or_else(|| {
        format!(
            "runtime plugin {} has no package manifest",
            plugin.plugin_id
        )
    })?;
    native_bridge_method_descriptors_from_manifest(manifest, report.bridge_method_bindings.clone())
        .map_err(|error| error.to_string())?;
    Ok(Some(report.bridge_method_bindings.clone()))
}

pub(super) fn discovered_runtime_bridge_method_binding_diagnostics(
    plugin_id: &str,
    bindings: &[NativeBridgeMethodBinding],
) -> String {
    format!(
        "native.live_host.bridge_bindings_discovered: Runtime plugin `{plugin_id}` installed {} bridge method(s)",
        bindings.len()
    )
}

pub(super) fn discovered_runtime_bridge_method_binding_error_diagnostic(
    plugin_id: &str,
    error: &str,
) -> String {
    format!(
        "native.live_host.bridge_bindings_discovery_failed: Runtime plugin `{plugin_id}` bridge method table rejected: {error}"
    )
}

fn runtime_package_manifest(
    plugin: &super::super::LoadedNativePlugin,
) -> Option<&PluginPackageManifest> {
    plugin
        .runtime_entry_report
        .as_ref()
        .and_then(|report| report.package_manifest.as_ref())
        .or_else(|| {
            plugin
                .descriptor
                .as_ref()
                .and_then(|descriptor| descriptor.package_manifest.as_ref())
        })
}
