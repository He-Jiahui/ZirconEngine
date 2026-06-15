use std::path::PathBuf;

use crate::plugin::{
    NativeHostBridgeCallScope, PluginModuleKind, RuntimePluginBridgeLifecycleEvent,
    RuntimePluginBridgeLifecycleOutcome,
};

use super::super::super::runtime_plugin::{
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};
use super::super::{
    NativePluginBehaviorCallReport, NativePluginBehaviorValidationReport,
    ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use super::diagnostics::{combine_diagnostics, report_diagnostics, sorted_unique_diagnostics};
use super::keys::module_kind_label;

pub const NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND: &str = "play-mode.enter";
pub const NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND: &str = "play-mode.exit";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativePluginLiveHostCommand {
    Load,
    Unload,
    HotReload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginLiveHostBridgeLifecycleReport {
    pub plugin_id: String,
    pub module_kind: PluginModuleKind,
    pub command: NativePluginLiveHostCommand,
    pub event: RuntimePluginBridgeLifecycleEvent,
    pub outcome: RuntimePluginBridgeLifecycleOutcome,
}

impl NativePluginLiveHostBridgeLifecycleReport {
    pub fn is_applied(&self) -> bool {
        self.outcome.is_applied()
    }

    pub fn diagnostic(&self) -> String {
        format!(
            "native.live_host.bridge_lifecycle: {:?} {} plugin `{}` -> {}",
            self.command,
            module_kind_label(self.module_kind),
            self.plugin_id,
            self.outcome.diagnostic()
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginLiveHostOutcome {
    pub plugin_id: String,
    pub module_kind: PluginModuleKind,
    pub command: NativePluginLiveHostCommand,
    pub bridge_lifecycle_report: Option<NativePluginLiveHostBridgeLifecycleReport>,
    pub diagnostics: Vec<String>,
}

pub struct NativePluginLiveHostBridgeReloadReport {
    pub plugin_id: String,
    pub module_kind: PluginModuleKind,
    pub command: NativePluginLiveHostCommand,
    pub bridge_lifecycle_report: NativePluginLiveHostBridgeLifecycleReport,
    pub bridge_call_scope: NativeHostBridgeCallScope,
    pub diagnostics: Vec<String>,
}

impl NativePluginLiveHostBridgeReloadReport {
    pub fn diagnostic(&self) -> String {
        format!(
            "native.live_host.bridge_scope_reloaded: {:?} {} plugin `{}` rebuilt {} bridge method(s)",
            self.command,
            module_kind_label(self.module_kind),
            self.plugin_id,
            self.bridge_call_scope.method_count()
        )
    }
}

#[derive(Clone, Debug)]
pub struct NativePluginLiveHostLoadReport {
    pub module_kind: PluginModuleKind,
    pub loaded_plugin_ids: Vec<String>,
    pub runtime_plugin_registration_reports: Vec<RuntimePluginRegistrationReport>,
    pub runtime_plugin_feature_registration_reports: Vec<RuntimePluginFeatureRegistrationReport>,
    pub bridge_lifecycle_reports: Vec<NativePluginLiveHostBridgeLifecycleReport>,
    pub diagnostics: Vec<String>,
}

/// Manifest-driven runtime hot update report for NativeDynamic export roots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeHotUpdateReport {
    pub export_root: PathBuf,
    pub manifest_plugin_ids: Vec<String>,
    pub runtime_plugin_ids: Vec<String>,
    pub loaded_plugin_ids: Vec<String>,
    pub skipped_plugin_ids: Vec<String>,
    pub outcomes: Vec<NativePluginLiveHostOutcome>,
    pub diagnostics: Vec<String>,
}

impl NativePluginRuntimeHotUpdateReport {
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
            && self.runtime_plugin_ids.len() == self.loaded_plugin_ids.len()
            && self.skipped_plugin_ids.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeBehaviorDescriptor {
    pub plugin_id: String,
    pub is_stateless: Option<bool>,
    pub state_schema_version: Option<u32>,
    pub command_manifest_schema: Option<String>,
    pub event_manifest_schema: Option<String>,
    pub command_manifest: Option<String>,
    pub event_manifest: Option<String>,
    pub validation_report: Option<NativePluginBehaviorValidationReport>,
}

/// One ABI v2 runtime behavior callback result tied to its native package id.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeBehaviorCall {
    pub plugin_id: String,
    pub report: NativePluginBehaviorCallReport,
}

impl NativePluginRuntimeBehaviorCall {
    pub fn is_success(&self) -> bool {
        self.report.status_code == ZIRCON_NATIVE_PLUGIN_STATUS_OK
    }
}

/// Aggregate report for broadcasting a command to every loaded runtime native plugin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeCommandDispatchReport {
    pub command_name: String,
    pub calls: Vec<NativePluginRuntimeBehaviorCall>,
    pub diagnostics: Vec<String>,
}

impl NativePluginRuntimeCommandDispatchReport {
    pub fn failed_call_count(&self) -> usize {
        self.calls.iter().filter(|call| !call.is_success()).count()
    }

    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty() && self.failed_call_count() == 0
    }

    pub fn combined_diagnostics(&self) -> Vec<String> {
        let mut diagnostics = self.diagnostics.clone();
        for call in &self.calls {
            diagnostics.extend(report_diagnostics(
                &call.plugin_id,
                &self.command_name,
                &call.report,
            ));
        }
        sorted_unique_diagnostics(diagnostics)
    }
}

/// Serialized state captured from one loaded runtime native plugin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimePluginState {
    pub plugin_id: String,
    pub state_schema_version: Option<u32>,
    pub state: Vec<u8>,
}

/// Play-mode friendly snapshot of every stateful loaded runtime native plugin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeStateSnapshot {
    pub plugin_states: Vec<NativePluginRuntimePluginState>,
    pub diagnostics: Vec<String>,
}

impl NativePluginRuntimeStateSnapshot {
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn combined_diagnostics(&self) -> Vec<String> {
        sorted_unique_diagnostics(self.diagnostics.clone())
    }
}

/// Aggregate restore report for a previously captured native runtime snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeStateRestoreReport {
    pub calls: Vec<NativePluginRuntimeBehaviorCall>,
    pub skipped_plugin_ids: Vec<String>,
    pub diagnostics: Vec<String>,
}

impl NativePluginRuntimeStateRestoreReport {
    pub fn failed_call_count(&self) -> usize {
        self.calls.iter().filter(|call| !call.is_success()).count()
    }

    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
            && self.skipped_plugin_ids.is_empty()
            && self.failed_call_count() == 0
    }

    pub fn combined_diagnostics(&self) -> Vec<String> {
        let mut diagnostics = self.diagnostics.clone();
        for call in &self.calls {
            diagnostics.extend(report_diagnostics(
                &call.plugin_id,
                "restore-state",
                &call.report,
            ));
        }
        sorted_unique_diagnostics(diagnostics)
    }
}

/// Native runtime plugin state captured when editor play mode begins.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimePlayModeSnapshot {
    pub state_snapshot: NativePluginRuntimeStateSnapshot,
    pub enter_report: NativePluginRuntimeCommandDispatchReport,
}

impl NativePluginRuntimePlayModeSnapshot {
    pub fn is_clean(&self) -> bool {
        self.state_snapshot.is_clean() && self.enter_report.is_clean()
    }

    pub fn combined_diagnostics(&self) -> Vec<String> {
        combine_diagnostics([
            self.state_snapshot.combined_diagnostics(),
            self.enter_report.combined_diagnostics(),
        ])
    }
}

/// Exit report that pairs the play-mode exit broadcast with state restoration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimePlayModeExitReport {
    pub exit_report: NativePluginRuntimeCommandDispatchReport,
    pub restore_report: NativePluginRuntimeStateRestoreReport,
}

impl NativePluginRuntimePlayModeExitReport {
    pub fn is_clean(&self) -> bool {
        self.exit_report.is_clean() && self.restore_report.is_clean()
    }

    pub fn combined_diagnostics(&self) -> Vec<String> {
        combine_diagnostics([
            self.exit_report.combined_diagnostics(),
            self.restore_report.combined_diagnostics(),
        ])
    }
}
