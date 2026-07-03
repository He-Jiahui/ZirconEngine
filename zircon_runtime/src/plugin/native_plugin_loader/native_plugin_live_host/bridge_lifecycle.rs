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

pub(super) type NativePluginBridgeLifecycleResult<T> =
    std::result::Result<T, NativePluginBridgeLifecycleError>;

#[derive(Debug)]
pub(super) enum NativePluginBridgeLifecycleError {
    Load {
        diagnostic: String,
    },
    HotReload {
        diagnostic: String,
    },
    Unload {
        diagnostic: String,
    },
    BridgeLifecycleRejected {
        diagnostic: String,
    },
    UnloadRollback {
        diagnostic: String,
        rollback_diagnostic: String,
    },
}

impl std::fmt::Display for NativePluginBridgeLifecycleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Load { diagnostic }
            | Self::HotReload { diagnostic }
            | Self::Unload { diagnostic }
            | Self::BridgeLifecycleRejected { diagnostic } => formatter.write_str(diagnostic),
            Self::UnloadRollback {
                diagnostic,
                rollback_diagnostic,
            } => write!(formatter, "{diagnostic}; {rollback_diagnostic}"),
        }
    }
}

impl std::error::Error for NativePluginBridgeLifecycleError {}

impl NativePluginLiveHost {
    pub fn hot_reload_runtime_plugins_from_export_root_with_bridge_lifecycle(
        &self,
        export_root: impl AsRef<Path>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginRuntimeHotUpdateReport, String> {
        self.hot_reload_runtime_plugins_from_export_root_with_bridge_lifecycle_result(
            export_root,
            lifecycle,
        )
        .map_err(|error| error.to_string())
    }

    pub(super) fn hot_reload_runtime_plugins_from_export_root_with_bridge_lifecycle_result(
        &self,
        export_root: impl AsRef<Path>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> NativePluginBridgeLifecycleResult<NativePluginRuntimeHotUpdateReport> {
        let mut report = self
            .hot_reload_runtime_plugins_from_export_root(export_root)
            .map_err(|diagnostic| NativePluginBridgeLifecycleError::HotReload { diagnostic })?;
        report.apply_runtime_bridge_lifecycle(lifecycle);
        Ok(report)
    }

    pub fn load_runtime_plugins_from_export_root_with_bridge_lifecycle(
        &self,
        export_root: impl AsRef<Path>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        self.load_runtime_plugins_from_export_root_with_bridge_lifecycle_result(
            export_root,
            lifecycle,
        )
        .map_err(|error| error.to_string())
    }

    pub(super) fn load_runtime_plugins_from_export_root_with_bridge_lifecycle_result(
        &self,
        export_root: impl AsRef<Path>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> NativePluginBridgeLifecycleResult<NativePluginLiveHostLoadReport> {
        let mut report = self
            .load_runtime_plugins_from_export_root(export_root)
            .map_err(|diagnostic| NativePluginBridgeLifecycleError::Load { diagnostic })?;
        report.apply_runtime_bridge_lifecycle(lifecycle);
        Ok(report)
    }

    pub fn load_runtime_plugins_from_project_root_with_bridge_lifecycle(
        &self,
        root: impl AsRef<Path>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        self.load_runtime_plugins_from_project_root_with_bridge_lifecycle_result(root, lifecycle)
            .map_err(|error| error.to_string())
    }

    pub(super) fn load_runtime_plugins_from_project_root_with_bridge_lifecycle_result(
        &self,
        root: impl AsRef<Path>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> NativePluginBridgeLifecycleResult<NativePluginLiveHostLoadReport> {
        let mut report = self
            .load_runtime_plugins_from_project_root(root)
            .map_err(|diagnostic| NativePluginBridgeLifecycleError::Load { diagnostic })?;
        report.apply_runtime_bridge_lifecycle(lifecycle);
        Ok(report)
    }

    pub fn unload_runtime_plugin_with_bridge_lifecycle(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.unload_runtime_plugin_with_bridge_lifecycle_result(plugin_id, lifecycle)
            .map_err(|error| error.to_string())
    }

    pub(super) fn unload_runtime_plugin_with_bridge_lifecycle_result(
        &self,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> NativePluginBridgeLifecycleResult<NativePluginLiveHostOutcome> {
        let plugin_id = plugin_id.as_ref();
        let bridge_report = runtime_bridge_lifecycle_report(
            plugin_id,
            NativePluginLiveHostCommand::Unload,
            RuntimePluginBridgeLifecycleEvent::deactivate_provider(plugin_id),
            lifecycle,
        );
        if !bridge_report.is_applied() {
            return Err(NativePluginBridgeLifecycleError::BridgeLifecycleRejected {
                diagnostic: bridge_report.diagnostic(),
            });
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
                Err(NativePluginBridgeLifecycleError::UnloadRollback {
                    diagnostic: error,
                    rollback_diagnostic: rollback_report.diagnostic(),
                })
            }
        }
    }

    pub fn hot_reload_runtime_plugin_with_bridge_lifecycle(
        &self,
        root: impl AsRef<Path>,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> Result<NativePluginLiveHostOutcome, String> {
        self.hot_reload_runtime_plugin_with_bridge_lifecycle_result(root, plugin_id, lifecycle)
            .map_err(|error| error.to_string())
    }

    pub(super) fn hot_reload_runtime_plugin_with_bridge_lifecycle_result(
        &self,
        root: impl AsRef<Path>,
        plugin_id: impl AsRef<str>,
        lifecycle: &RuntimePluginBridgeLifecycleState,
    ) -> NativePluginBridgeLifecycleResult<NativePluginLiveHostOutcome> {
        let plugin_id = plugin_id.as_ref();
        let mut outcome = self
            .hot_reload_runtime_plugin(root, plugin_id)
            .map_err(|diagnostic| NativePluginBridgeLifecycleError::HotReload { diagnostic })?;
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
