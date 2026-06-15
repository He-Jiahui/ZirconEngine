use std::path::Path;

use crate::plugin::{
    PluginModuleKind, RuntimePluginBridgeLifecycleEvent, RuntimePluginBridgeLifecycleState,
};

use super::reports::{
    NativePluginLiveHostBridgeLifecycleReport, NativePluginLiveHostCommand,
    NativePluginLiveHostLoadReport, NativePluginLiveHostOutcome,
    NativePluginRuntimeHotUpdateReport,
};
use super::NativePluginLiveHost;

impl NativePluginLiveHost {
    pub fn hot_reload_runtime_plugins_from_export_root_with_bridge_lifecycle(
        &self,
        export_root: impl AsRef<Path>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginRuntimeHotUpdateReport, String> {
        let mut report = self.hot_reload_runtime_plugins_from_export_root(export_root)?;
        report.apply_runtime_bridge_lifecycle(lifecycle);
        Ok(report)
    }

    pub fn load_runtime_plugins_from_export_root_with_bridge_lifecycle(
        &self,
        export_root: impl AsRef<Path>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let mut report = self.load_runtime_plugins_from_export_root(export_root)?;
        report.apply_runtime_bridge_lifecycle(lifecycle);
        Ok(report)
    }

    pub fn load_runtime_plugins_from_project_root_with_bridge_lifecycle(
        &self,
        root: impl AsRef<Path>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let mut report = self.load_runtime_plugins_from_project_root(root)?;
        report.apply_runtime_bridge_lifecycle(lifecycle);
        Ok(report)
    }

    pub fn unload_runtime_plugin_with_bridge_lifecycle(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        let plugin_id = plugin_id.as_ref();
        let bridge_report = runtime_bridge_lifecycle_report(
            plugin_id,
            NativePluginLiveHostCommand::Unload,
            RuntimePluginBridgeLifecycleEvent::deactivate_provider(plugin_id),
            lifecycle,
        );
        if !bridge_report.is_applied() {
            return Err(bridge_report.diagnostic());
        }

        match self.unload_runtime_plugin(plugin_id) {
            Ok(mut outcome) => {
                outcome.attach_bridge_lifecycle_report(bridge_report);
                Ok(outcome)
            }
            Err(error) => {
                let rollback_report = runtime_bridge_lifecycle_report(
                    plugin_id,
                    NativePluginLiveHostCommand::HotReload,
                    RuntimePluginBridgeLifecycleEvent::activate_provider(plugin_id),
                    lifecycle,
                );
                Err(format!("{error}; {}", rollback_report.diagnostic()))
            }
        }
    }

    pub fn hot_reload_runtime_plugin_with_bridge_lifecycle(
        &self,
        root: impl AsRef<Path>,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        let plugin_id = plugin_id.as_ref();
        let mut outcome = self.hot_reload_runtime_plugin(root, plugin_id)?;
        let bridge_report = runtime_bridge_lifecycle_report(
            plugin_id,
            NativePluginLiveHostCommand::HotReload,
            RuntimePluginBridgeLifecycleEvent::reload_provider(plugin_id),
            lifecycle,
        );
        outcome.attach_bridge_lifecycle_report(bridge_report);
        Ok(outcome)
    }
}

impl NativePluginLiveHostLoadReport {
    pub fn apply_runtime_bridge_lifecycle(
        &mut self,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) {
        if self.module_kind != PluginModuleKind::Runtime {
            return;
        }

        for plugin_id in self.loaded_plugin_ids.clone() {
            let report = runtime_bridge_lifecycle_report(
                &plugin_id,
                NativePluginLiveHostCommand::Load,
                RuntimePluginBridgeLifecycleEvent::activate_provider(plugin_id.clone()),
                lifecycle,
            );
            self.diagnostics.push(report.diagnostic());
            self.bridge_lifecycle_reports.push(report);
        }
        self.diagnostics.sort();
        self.diagnostics.dedup();
    }
}

impl NativePluginRuntimeHotUpdateReport {
    pub fn apply_runtime_bridge_lifecycle(
        &mut self,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) {
        for outcome in &mut self.outcomes {
            if outcome.module_kind != PluginModuleKind::Runtime
                || outcome.command != NativePluginLiveHostCommand::HotReload
                || outcome.bridge_lifecycle_report.is_some()
            {
                continue;
            }

            let report = runtime_bridge_lifecycle_report(
                &outcome.plugin_id,
                NativePluginLiveHostCommand::HotReload,
                RuntimePluginBridgeLifecycleEvent::reload_provider(outcome.plugin_id.clone()),
                lifecycle,
            );
            outcome.attach_bridge_lifecycle_report(report);
            self.diagnostics.extend(outcome.diagnostics.clone());
        }
        self.diagnostics.sort();
        self.diagnostics.dedup();
    }
}

impl NativePluginLiveHostOutcome {
    pub fn attach_bridge_lifecycle_report(
        &mut self,
        report: NativePluginLiveHostBridgeLifecycleReport,
    ) {
        self.diagnostics.push(report.diagnostic());
        self.diagnostics.sort();
        self.diagnostics.dedup();
        self.bridge_lifecycle_report = Some(report);
    }
}

fn runtime_bridge_lifecycle_report(
    plugin_id: &str,
    command: NativePluginLiveHostCommand,
    event: RuntimePluginBridgeLifecycleEvent,
    lifecycle: &RuntimePluginBridgeLifecycleState,
) -> NativePluginLiveHostBridgeLifecycleReport {
    let outcome = lifecycle.apply_provider_lifecycle_event(event.clone());
    NativePluginLiveHostBridgeLifecycleReport {
        plugin_id: plugin_id.to_string(),
        module_kind: PluginModuleKind::Runtime,
        command,
        event,
        outcome,
    }
}
