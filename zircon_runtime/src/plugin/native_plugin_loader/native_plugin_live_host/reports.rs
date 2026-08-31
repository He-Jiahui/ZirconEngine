use std::path::{Path, PathBuf};

use crate::asset::pack::{ZrPackDeltaInstallReport, ZrPackInstallReceipt, ZrPackPromotionReport};
use crate::plugin::native::NativeHostBridgeCallScope;
use crate::plugin::{
    PluginModuleKind, RuntimePluginBridgeLifecycleEvent, RuntimePluginBridgeLifecycleOutcome,
};
use crate::scene::SystemStage;

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

/// Input paths for applying a zrpack delta before running a manifest-driven runtime hot update.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeDeltaHotUpdateRequest {
    pub export_root: PathBuf,
    pub base_pack: PathBuf,
    pub delta_pack: PathBuf,
    pub staged_pack: PathBuf,
    pub installed_pack: PathBuf,
    pub backup_pack: Option<PathBuf>,
    pub receipt_path: Option<PathBuf>,
}

impl NativePluginRuntimeDeltaHotUpdateRequest {
    pub fn new(
        export_root: impl AsRef<Path>,
        base_pack: impl AsRef<Path>,
        delta_pack: impl AsRef<Path>,
        staged_pack: impl AsRef<Path>,
        installed_pack: impl AsRef<Path>,
    ) -> Self {
        Self {
            export_root: export_root.as_ref().to_path_buf(),
            base_pack: base_pack.as_ref().to_path_buf(),
            delta_pack: delta_pack.as_ref().to_path_buf(),
            staged_pack: staged_pack.as_ref().to_path_buf(),
            installed_pack: installed_pack.as_ref().to_path_buf(),
            backup_pack: None,
            receipt_path: None,
        }
    }

    pub fn with_backup_pack(mut self, backup_pack: impl AsRef<Path>) -> Self {
        self.backup_pack = Some(backup_pack.as_ref().to_path_buf());
        self
    }

    pub fn with_receipt_path(mut self, receipt_path: impl AsRef<Path>) -> Self {
        self.receipt_path = Some(receipt_path.as_ref().to_path_buf());
        self
    }
}

/// Runtime hot-update report that binds a promoted zrpack delta to the plugin reload pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeDeltaHotUpdateReport {
    pub pack_install: ZrPackDeltaInstallReport,
    pub pack_promotion: ZrPackPromotionReport,
    pub pack_install_receipt: Option<ZrPackInstallReceipt>,
    pub plugin_hot_update: NativePluginRuntimeHotUpdateReport,
}

impl NativePluginRuntimeDeltaHotUpdateReport {
    pub fn is_clean(&self) -> bool {
        self.pack_install.delta_apply_verified
            && self.pack_promotion.installed_manifest == self.pack_install.target_manifest
            && self
                .pack_install_receipt
                .as_ref()
                .is_none_or(|receipt| receipt.promoted && receipt.delta_apply_verified)
            && self.plugin_hot_update.is_clean()
    }
}

impl NativePluginRuntimeHotUpdateReport {
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty()
            && self.runtime_plugin_ids.len() == self.loaded_plugin_ids.len()
            && self.skipped_plugin_ids.is_empty()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativePluginRuntimeRegistrationReplayReport {
    pub registered_systems: Vec<NativePluginRuntimeRegistrationSystemReplay>,
    pub skipped_plugin_ids: Vec<String>,
    pub diagnostics: Vec<String>,
}

impl NativePluginRuntimeRegistrationReplayReport {
    pub fn is_clean(&self) -> bool {
        self.diagnostics.is_empty() && self.skipped_plugin_ids.is_empty()
    }

    pub(super) fn append(&mut self, other: &mut Self) {
        self.registered_systems
            .append(&mut other.registered_systems);
        self.skipped_plugin_ids
            .append(&mut other.skipped_plugin_ids);
        self.diagnostics.append(&mut other.diagnostics);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeRegistrationSystemReplay {
    pub plugin_id: String,
    pub module: String,
    pub system_id: String,
    pub stage: SystemStage,
    pub order: i32,
    pub bridge_interface: String,
    pub bridge_method: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginRuntimeBehaviorDescriptor {
    pub plugin_id: String,
    pub is_stateless: Option<bool>,
    pub state_schema_version: Option<u32>,
    pub command_manifest_schema: Option<String>,
    pub event_manifest_schema: Option<String>,
    pub registration_manifest_schema: Option<String>,
    pub command_manifest: Option<String>,
    pub event_manifest: Option<String>,
    pub registration_manifest: Option<String>,
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
        let mut diagnostics =
            Vec::with_capacity(self.diagnostics.len() + report_diagnostic_capacity(&self.calls));
        diagnostics.extend(self.diagnostics.iter().cloned());
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
        let mut diagnostics =
            Vec::with_capacity(self.diagnostics.len() + report_diagnostic_capacity(&self.calls));
        diagnostics.extend(self.diagnostics.iter().cloned());
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

fn report_diagnostic_capacity(calls: &[NativePluginRuntimeBehaviorCall]) -> usize {
    calls
        .iter()
        .map(|call| {
            if call.report.diagnostics.is_empty() {
                usize::from(call.report.status_code != ZIRCON_NATIVE_PLUGIN_STATUS_OK)
            } else {
                call.report.diagnostics.len()
            }
        })
        .sum()
}

#[cfg(test)]
#[path = "reports/optimization_tests.rs"]
mod optimization_tests;

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
