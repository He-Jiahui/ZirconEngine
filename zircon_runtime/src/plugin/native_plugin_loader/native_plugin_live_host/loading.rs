use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::MutexGuard,
};

use crate::core::framework::bridge::BridgeInterfaceStatus;
use crate::plugin::{
    PluginModuleKind, RuntimePluginBridgeLifecycleEvent, RuntimePluginBridgeLifecycleState,
};

use super::super::loaded_native_plugin::NativePluginLifecycleTransitionError;
use super::super::{LoadedNativePlugin, NativePluginLoadReport};
use super::super::{NativePluginArtifactAuthority, NativePluginArtifactTarget};
use super::bridge_methods::{
    discovered_runtime_bridge_method_binding_diagnostics,
    discovered_runtime_bridge_method_binding_error_diagnostic,
    discovered_runtime_bridge_method_bindings_result, NativePluginBridgeMethodError,
    ValidatedRuntimeBridgeMethodBindings,
};
use super::diagnostics::{
    diagnostics_from_behavior_report, load_projected_report_diagnostics,
    NativePluginBehaviorDiagnosticError,
};
use super::keys::{live_key, module_kind_label, NativePluginLiveRegistry};
use super::reports::NativePluginLiveHostLoadReport;
use super::runtime_behavior::unload_behavior;
use super::{NativePluginLiveHost, ObservedLoadedNativePlugins};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginProjectActivationSelection {
    pub plugin_id: String,
    pub required: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativePluginProjectActivationSelectionStatus {
    Activated,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginProjectActivationSelectionResult {
    pub plugin_id: String,
    pub required: bool,
    pub status: NativePluginProjectActivationSelectionStatus,
    pub diagnostic: Option<String>,
}

/// Signed package selections are resolved in the lower live-host layer so candidate DLLs are
/// staged, gated, and published as one transaction.
pub struct NativePluginProjectActivationRequest<'a> {
    pub installed_root: &'a Path,
    pub target: NativePluginArtifactTarget,
    pub selections: &'a [NativePluginProjectActivationSelection],
    pub bridge_lifecycle: Option<&'a RuntimePluginBridgeLifecycleState>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NativePluginProjectActivationCleanupReceipt {
    pub retained_plugin_ids: Vec<String>,
    pub pending_bridge_rollback_plugin_ids: Vec<String>,
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct NativePluginProjectActivationResult {
    pub target: NativePluginArtifactTarget,
    pub committed: bool,
    pub selections: Vec<NativePluginProjectActivationSelectionResult>,
    pub load_report: NativePluginLiveHostLoadReport,
    pub bridge_lifecycle_receipts: Vec<super::reports::NativePluginLiveHostBridgeLifecycleReport>,
    pub cleanup_receipt: NativePluginProjectActivationCleanupReceipt,
}

impl NativePluginProjectActivationResult {
    pub fn has_required_failures(&self) -> bool {
        self.selections.iter().any(|selection| {
            selection.required
                && selection.status == NativePluginProjectActivationSelectionStatus::Failed
        })
    }

    pub fn required_failure_diagnostics(&self) -> Vec<String> {
        self.selections
            .iter()
            .filter(|selection| {
                selection.required
                    && selection.status == NativePluginProjectActivationSelectionStatus::Failed
            })
            .map(|selection| {
                selection.diagnostic.clone().unwrap_or_else(|| {
                    format!(
                        "required native plugin `{}` failed activation",
                        selection.plugin_id
                    )
                })
            })
            .collect()
    }
}

#[derive(Debug)]
pub(super) enum PendingNativePluginProjectActivationRecovery {
    RetainedPlugin {
        plugin: LoadedNativePlugin,
        module_kind: PluginModuleKind,
        disposition: &'static str,
        diagnostic: String,
    },
    BridgeRollback {
        lifecycle: RuntimePluginBridgeLifecycleState,
        plugin_ids: Vec<String>,
        retained_plugins: Vec<LoadedNativePlugin>,
        diagnostic: String,
    },
}

pub(super) type NativePluginLiveHostLoadingResult<T> =
    std::result::Result<T, NativePluginLiveHostLoadingError>;

#[derive(Debug)]
pub(super) enum NativePluginLiveHostLoadingError {
    LiveHostLockPoisoned,
    PluginBusy {
        plugin_id: String,
        module_kind: PluginModuleKind,
        source: NativePluginLifecycleTransitionError,
    },
    UnloadBeforeReload {
        plugin_id: String,
        module_kind: PluginModuleKind,
        source: NativePluginBehaviorDiagnosticError,
    },
    RuntimeBridgeMethodBindings {
        plugin_id: String,
        source: Box<NativePluginBridgeMethodError>,
    },
}

impl std::fmt::Display for NativePluginLiveHostLoadingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LiveHostLockPoisoned => {
                formatter.write_str("native plugin live host lock is poisoned")
            }
            Self::PluginBusy {
                plugin_id,
                module_kind,
                source,
            } => write!(
                formatter,
                "{} plugin {plugin_id} lifecycle is busy during load: {source}",
                module_kind_label(*module_kind)
            ),
            Self::UnloadBeforeReload {
                plugin_id,
                module_kind,
                source,
            } => write!(
                formatter,
                "{} plugin {plugin_id} unload before reload failed: {source}",
                module_kind_label(*module_kind)
            ),
            Self::RuntimeBridgeMethodBindings { plugin_id, source } => write!(
                formatter,
                "runtime plugin {plugin_id} bridge method binding install failed while loading: {source}"
            ),
        }
    }
}

impl std::error::Error for NativePluginLiveHostLoadingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PluginBusy { source, .. } => Some(source),
            Self::UnloadBeforeReload { source, .. } => Some(source),
            Self::RuntimeBridgeMethodBindings { source, .. } => Some(source.as_ref()),
            Self::LiveHostLockPoisoned => None,
        }
    }
}

impl NativePluginLiveHost {
    pub fn load_runtime_plugins_from_export_root(
        &self,
        export_root: impl AsRef<std::path::Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self
            .loader
            .load_runtime_from_load_manifest_with_authority(export_root, &self.artifact_authority);
        self.load_reported_plugins_result(report, PluginModuleKind::Runtime)
            .map_err(|error| error.to_string())
    }

    pub fn load_editor_plugins_from_export_root(
        &self,
        export_root: impl AsRef<std::path::Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self
            .loader
            .load_editor_from_load_manifest_with_authority(export_root, &self.artifact_authority);
        self.load_reported_plugins_result(report, PluginModuleKind::Editor)
            .map_err(|error| error.to_string())
    }

    pub fn load_runtime_plugins_from_project_root(
        &self,
        root: impl AsRef<std::path::Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        self.load_runtime_plugins_from_project_root_with_authority(root, &self.artifact_authority)
    }

    pub fn load_runtime_plugins_from_project_root_with_authority(
        &self,
        root: impl AsRef<std::path::Path>,
        authority: &super::super::NativePluginArtifactAuthority,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self
            .loader
            .load_discovered_runtime_with_authority(root, authority);
        self.load_reported_plugins_result(report, PluginModuleKind::Runtime)
            .map_err(|error| error.to_string())
    }

    pub fn load_editor_plugins_from_project_root(
        &self,
        root: impl AsRef<std::path::Path>,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        self.load_editor_plugins_from_project_root_with_authority(root, &self.artifact_authority)
    }

    pub fn load_editor_plugins_from_project_root_with_authority(
        &self,
        root: impl AsRef<std::path::Path>,
        authority: &super::super::NativePluginArtifactAuthority,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        let report = self
            .loader
            .load_discovered_editor_with_authority(root, authority);
        self.load_reported_plugins_result(report, PluginModuleKind::Editor)
            .map_err(|error| error.to_string())
    }

    pub fn loaded_plugin_ids(&self, module_kind: PluginModuleKind) -> Result<Vec<String>, String> {
        let loaded = lock_loaded_native_plugins(&self.loaded).map_err(|error| error.to_string())?;
        Ok(loaded.plugin_ids(module_kind).map(str::to_string).collect())
    }

    pub(super) fn load_reported_plugins(
        &self,
        report: NativePluginLoadReport,
        module_kind: PluginModuleKind,
    ) -> Result<NativePluginLiveHostLoadReport, String> {
        self.load_reported_plugins_result(report, module_kind)
            .map_err(|error| error.to_string())
    }

    pub(super) fn load_reported_plugins_result(
        &self,
        mut report: NativePluginLoadReport,
        module_kind: PluginModuleKind,
    ) -> NativePluginLiveHostLoadingResult<NativePluginLiveHostLoadReport> {
        let projection = report.projection();
        let (runtime_plugin_registration_reports, runtime_plugin_feature_registration_reports) =
            match module_kind {
                PluginModuleKind::Runtime => (
                    projection.runtime_plugin_registration_reports(),
                    projection.runtime_plugin_feature_registration_reports(),
                ),
                PluginModuleKind::Editor | PluginModuleKind::Native | PluginModuleKind::Vm => {
                    (Vec::new(), Vec::new())
                }
            };
        let mut diagnostics = load_projected_report_diagnostics(&report, &projection);
        let mut loaded_plugin_ids = Vec::new();

        for plugin in report.take_loaded() {
            let plugin_id = plugin.plugin_id.clone();
            let key = live_key(module_kind, &plugin_id);
            let bridge_binding_update = if module_kind == PluginModuleKind::Runtime {
                match discovered_runtime_bridge_method_bindings_result(&plugin) {
                    Ok(Some(bindings)) => {
                        diagnostics.push(discovered_runtime_bridge_method_binding_diagnostics(
                            &plugin_id,
                            bindings.len(),
                        ));
                        Some((plugin_id.clone(), Some(bindings)))
                    }
                    Ok(None) => Some((plugin_id.clone(), None)),
                    Err(error) => {
                        diagnostics.push(
                            discovered_runtime_bridge_method_binding_error_diagnostic(
                                &plugin_id, &error,
                            ),
                        );
                        Some((plugin_id.clone(), None))
                    }
                }
            } else {
                None
            };
            let existing = {
                let mut loaded = lock_loaded_native_plugins(&self.loaded)?;
                let Some(existing) = loaded.get(&key) else {
                    if let Some((binding_plugin_id, bindings)) = bridge_binding_update {
                        self.publish_runtime_bridge_method_bindings_under_loaded_lock_result(
                            &loaded,
                            &binding_plugin_id,
                            bindings,
                        )
                        .map_err(|source| {
                            NativePluginLiveHostLoadingError::RuntimeBridgeMethodBindings {
                                plugin_id: binding_plugin_id,
                                source: Box::new(source),
                            }
                        })?;
                    }
                    loaded.insert(key, plugin);
                    if module_kind == PluginModuleKind::Runtime {
                        self.invalidate_runtime_registration_replay_generation(&plugin_id);
                    }
                    drop(loaded);
                    loaded_plugin_ids.push(plugin_id.clone());
                    continue;
                };
                if let Err(source) = existing.begin_lifecycle_transition() {
                    return Err(NativePluginLiveHostLoadingError::PluginBusy {
                        plugin_id,
                        module_kind,
                        source,
                    });
                }
                existing.clone()
            };
            match diagnostics_from_behavior_report(
                &format!("{} unload before reload", module_kind_label(module_kind)),
                unload_behavior(&existing, module_kind),
            ) {
                Ok(unload_diagnostics) => diagnostics.extend(unload_diagnostics),
                Err(error) => {
                    existing.cancel_lifecycle_transition();
                    return Err(NativePluginLiveHostLoadingError::UnloadBeforeReload {
                        plugin_id,
                        module_kind,
                        source: error,
                    });
                }
            }
            let mut loaded = match lock_loaded_native_plugins(&self.loaded) {
                Ok(loaded) => loaded,
                Err(error) => {
                    existing.cancel_lifecycle_transition();
                    return Err(error);
                }
            };
            if let Some((binding_plugin_id, bindings)) = bridge_binding_update {
                if let Err(source) = self
                    .publish_runtime_bridge_method_bindings_under_loaded_lock_result(
                        &loaded,
                        &binding_plugin_id,
                        bindings,
                    )
                {
                    existing.cancel_lifecycle_transition();
                    return Err(
                        NativePluginLiveHostLoadingError::RuntimeBridgeMethodBindings {
                            plugin_id: binding_plugin_id,
                            source: Box::new(source),
                        },
                    );
                }
            }
            loaded.insert(key, plugin);
            if module_kind == PluginModuleKind::Runtime {
                self.invalidate_runtime_registration_replay_generation(&plugin_id);
            }
            drop(loaded);
            loaded_plugin_ids.push(plugin_id);
        }

        loaded_plugin_ids.sort_unstable();
        loaded_plugin_ids.dedup();
        diagnostics.sort_unstable();
        diagnostics.dedup();
        Ok(NativePluginLiveHostLoadReport {
            module_kind,
            loaded_plugin_ids,
            runtime_plugin_registration_reports,
            runtime_plugin_feature_registration_reports,
            bridge_lifecycle_reports: Vec::new(),
            diagnostics,
        })
    }
}

impl NativePluginLiveHost {
    /// Loads only the admitted ids, validates their runtime registration and bridge contracts,
    /// then publishes the staged generations together.
    pub fn activate_runtime_project_plugins(
        &self,
        request: NativePluginProjectActivationRequest<'_>,
        authority: &NativePluginArtifactAuthority,
    ) -> NativePluginProjectActivationResult {
        let mut report = self.loader.discover(request.installed_root);
        let mut allowed_plugin_ids = HashSet::with_capacity(request.selections.len());
        let mut selected_counts = HashMap::<&str, usize>::with_capacity(request.selections.len());
        for selection in request.selections {
            *selected_counts.entry(&selection.plugin_id).or_default() += 1;
        }
        for selection in request.selections {
            if selected_counts.get(selection.plugin_id.as_str()).copied() != Some(1) {
                report.push_diagnostic(format!(
                    "native plugin {}: project activation contains a duplicate selection",
                    selection.plugin_id
                ));
                continue;
            }
            match authority.expectation(&selection.plugin_id) {
                Some(expectation) if expectation.target == request.target => {
                    allowed_plugin_ids.insert(selection.plugin_id.as_str());
                }
                Some(expectation) => report.push_diagnostic(format!(
                    "native plugin {}: installed proof targets {:?}, requested activation targets {:?}",
                    selection.plugin_id, expectation.target, request.target
                )),
                None => report.push_diagnostic(format!(
                    "native plugin {}: installed authority has no selected package proof",
                    selection.plugin_id
                )),
            }
        }

        let mut discovered = report.take_discovered();
        discovered.retain(|candidate| allowed_plugin_ids.contains(candidate.plugin_id.as_str()));
        report.restore_discovered(discovered);
        let report = self.loader.load_candidates_for_module_kinds(
            report,
            &[PluginModuleKind::Runtime],
            authority,
        );
        self.activate_runtime_project_plugins_from_report(request, report)
    }

    /// Retry callbacks and bridge deactivations retained by a failed project activation rollback.
    pub fn retry_project_activation_recovery(&self) -> NativePluginProjectActivationCleanupReceipt {
        let mut pending = {
            let mut recovery = self
                .project_activation_recovery
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut *recovery)
        };
        let mut receipt = NativePluginProjectActivationCleanupReceipt::default();
        let mut remaining = Vec::new();

        for recovery in pending.drain(..) {
            match recovery {
                PendingNativePluginProjectActivationRecovery::RetainedPlugin {
                    plugin,
                    module_kind,
                    disposition,
                    diagnostic,
                } => {
                    let cleanup = diagnostics_from_behavior_report(
                        &format!("project activation {disposition}"),
                        unload_behavior(&plugin, module_kind),
                    );
                    match cleanup {
                        Ok(diagnostics) => receipt.diagnostics.extend(diagnostics),
                        Err(error) => {
                            let plugin_id = plugin.plugin_id.clone();
                            receipt.retained_plugin_ids.push(plugin_id.clone());
                            receipt.diagnostics.push(format!(
                                "{diagnostic}; retry for retained plugin `{plugin_id}` failed: {error}"
                            ));
                            remaining.push(
                                PendingNativePluginProjectActivationRecovery::RetainedPlugin {
                                    plugin,
                                    module_kind,
                                    disposition,
                                    diagnostic,
                                },
                            );
                        }
                    }
                }
                PendingNativePluginProjectActivationRecovery::BridgeRollback {
                    lifecycle,
                    plugin_ids,
                    retained_plugins,
                    diagnostic,
                } => {
                    let mut pending_plugin_ids = Vec::new();
                    for plugin_id in plugin_ids {
                        let outcome = lifecycle.apply_provider_lifecycle_event(
                            RuntimePluginBridgeLifecycleEvent::deactivate_provider(
                                plugin_id.clone(),
                            ),
                        );
                        if outcome.is_applied() {
                            receipt.diagnostics.push(format!(
                                "project activation bridge rollback completed for `{plugin_id}`"
                            ));
                        } else {
                            receipt
                                .pending_bridge_rollback_plugin_ids
                                .push(plugin_id.clone());
                            receipt.diagnostics.push(format!(
                                "{diagnostic}; retry for bridge provider `{plugin_id}` failed: {}",
                                outcome.diagnostic()
                            ));
                            pending_plugin_ids.push(plugin_id);
                        }
                    }
                    if pending_plugin_ids.is_empty() {
                        for plugin in retained_plugins {
                            retain_failed_activation_cleanup(
                                plugin,
                                PluginModuleKind::Runtime,
                                "candidate cleanup after bridge rollback",
                                &diagnostic,
                                &mut receipt,
                                &mut remaining,
                            );
                        }
                    } else {
                        remaining.push(
                            PendingNativePluginProjectActivationRecovery::BridgeRollback {
                                lifecycle,
                                plugin_ids: pending_plugin_ids,
                                retained_plugins,
                                diagnostic,
                            },
                        );
                    }
                }
            }
        }

        receipt.retained_plugin_ids.sort_unstable();
        receipt.retained_plugin_ids.dedup();
        receipt.pending_bridge_rollback_plugin_ids.sort_unstable();
        receipt.pending_bridge_rollback_plugin_ids.dedup();
        receipt.diagnostics.sort_unstable();
        receipt.diagnostics.dedup();
        self.project_activation_recovery
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend(remaining);
        receipt
    }

    fn activate_runtime_project_plugins_from_report(
        &self,
        request: NativePluginProjectActivationRequest<'_>,
        mut report: NativePluginLoadReport,
    ) -> NativePluginProjectActivationResult {
        let projection = report.projection();
        let registration_reports = projection.runtime_plugin_registration_reports();
        let feature_reports = projection.runtime_plugin_feature_registration_reports();
        let mut diagnostics = load_projected_report_diagnostics(&report, projection);
        let mut selection_results = request
            .selections
            .iter()
            .map(|selection| NativePluginProjectActivationSelectionResult {
                plugin_id: selection.plugin_id.clone(),
                required: selection.required,
                status: NativePluginProjectActivationSelectionStatus::Failed,
                diagnostic: None,
            })
            .collect::<Vec<_>>();
        let mut selection_counts = HashMap::<&str, usize>::with_capacity(request.selections.len());
        for selection in request.selections {
            *selection_counts.entry(&selection.plugin_id).or_default() += 1;
        }
        let loaded_counts = report.loaded().iter().fold(
            HashMap::<&str, usize>::with_capacity(report.loaded().len()),
            |mut counts, plugin| {
                *counts.entry(plugin.plugin_id.as_str()).or_default() += 1;
                counts
            },
        );
        let mut provider_ids_by_plugin = HashMap::<String, String>::new();

        for result in &mut selection_results {
            let plugin_id = result.plugin_id.as_str();
            let fail = if selection_counts.get(plugin_id).copied() != Some(1) {
                Some("project activation selection id is duplicated".to_string())
            } else if loaded_counts.get(plugin_id).copied().unwrap_or_default() != 1 {
                let selected_diagnostics = projection.runtime_diagnostics_for_plugin(plugin_id);
                Some(if selected_diagnostics.is_empty() {
                    "the selected installed runtime package was not loaded".to_string()
                } else {
                    selected_diagnostics.join("; ")
                })
            } else if let Some(registration) = registration_reports
                .iter()
                .find(|registration| registration.package_manifest.id == plugin_id)
            {
                if !registration.is_success() {
                    Some(if registration.diagnostics.is_empty() {
                        "runtime package registration failed".to_string()
                    } else {
                        registration.diagnostics.join("; ")
                    })
                } else if let Some(feature) = feature_reports.iter().find(|feature| {
                    feature.provider_package_id_or_owner() == plugin_id && !feature.is_success()
                }) {
                    Some(if feature.diagnostics.is_empty() {
                        "runtime feature registration failed".to_string()
                    } else {
                        feature.diagnostics.join("; ")
                    })
                } else if let Some(plugin) = report
                    .loaded()
                    .iter()
                    .find(|plugin| plugin.plugin_id == plugin_id)
                {
                    match validate_project_activation_bridge_provider(
                        plugin_id,
                        plugin,
                        request.bridge_lifecycle,
                    ) {
                        Ok(Some(provider_id)) => {
                            provider_ids_by_plugin.insert(plugin_id.to_string(), provider_id);
                            None
                        }
                        Ok(None) => None,
                        Err(reason) => Some(reason),
                    }
                } else {
                    Some("the loaded runtime plugin entry is unavailable".to_string())
                }
            } else {
                Some("runtime package registration report is missing".to_string())
            };

            if let Some(reason) = fail {
                result.status = NativePluginProjectActivationSelectionStatus::Failed;
                result.diagnostic = Some(format!(
                    "native plugin `{plugin_id}` runtime activation failed: {reason}"
                ));
            } else {
                result.status = NativePluginProjectActivationSelectionStatus::Activated;
            }
        }

        let staged_plugins = report.take_loaded();
        if selection_results.iter().any(|selection| {
            selection.required
                && selection.status == NativePluginProjectActivationSelectionStatus::Failed
        }) {
            let transaction_diagnostic =
                "project activation was aborted because a required selection failed";
            for selection in &mut selection_results {
                if selection.status == NativePluginProjectActivationSelectionStatus::Activated {
                    selection.status = NativePluginProjectActivationSelectionStatus::Failed;
                    selection.diagnostic = Some(format!(
                        "native plugin `{}` was not published: {transaction_diagnostic}",
                        selection.plugin_id
                    ));
                }
            }
            let cleanup_receipt = self.cleanup_staged_project_plugins(staged_plugins);
            diagnostics.extend(cleanup_receipt.diagnostics.iter().cloned());
            append_selection_diagnostics(&mut diagnostics, &selection_results);
            return NativePluginProjectActivationResult {
                target: request.target,
                committed: false,
                selections: selection_results,
                load_report: NativePluginLiveHostLoadReport {
                    module_kind: PluginModuleKind::Runtime,
                    loaded_plugin_ids: Vec::new(),
                    runtime_plugin_registration_reports: registration_reports,
                    runtime_plugin_feature_registration_reports: feature_reports,
                    bridge_lifecycle_reports: Vec::new(),
                    diagnostics: sorted_diagnostics(diagnostics),
                },
                bridge_lifecycle_receipts: Vec::new(),
                cleanup_receipt,
            };
        }

        if request.selections.is_empty() {
            let cleanup_receipt = self.cleanup_staged_project_plugins(staged_plugins);
            diagnostics.extend(cleanup_receipt.diagnostics.iter().cloned());
            return NativePluginProjectActivationResult {
                target: request.target,
                committed: true,
                selections: selection_results,
                load_report: NativePluginLiveHostLoadReport {
                    module_kind: PluginModuleKind::Runtime,
                    loaded_plugin_ids: Vec::new(),
                    runtime_plugin_registration_reports: registration_reports,
                    runtime_plugin_feature_registration_reports: feature_reports,
                    bridge_lifecycle_reports: Vec::new(),
                    diagnostics: sorted_diagnostics(diagnostics),
                },
                bridge_lifecycle_receipts: Vec::new(),
                cleanup_receipt,
            };
        }

        let mut staged_by_id = HashMap::<String, LoadedNativePlugin>::new();
        let mut rejected_staged_plugins = Vec::new();
        let staged_counts = staged_plugins.iter().fold(
            HashMap::<String, usize>::with_capacity(staged_plugins.len()),
            |mut counts, plugin| {
                *counts.entry(plugin.plugin_id.clone()).or_default() += 1;
                counts
            },
        );
        for plugin in staged_plugins {
            let selected = selection_results.iter().any(|selection| {
                selection.plugin_id == plugin.plugin_id
                    && selection.status == NativePluginProjectActivationSelectionStatus::Activated
            });
            if selected && staged_counts.get(plugin.plugin_id.as_str()).copied() == Some(1) {
                staged_by_id.insert(plugin.plugin_id.clone(), plugin);
            } else {
                rejected_staged_plugins.push(plugin);
            }
        }

        let mut prepared_bindings =
            HashMap::<String, Option<ValidatedRuntimeBridgeMethodBindings>>::new();
        let mut candidate_ids = staged_by_id.keys().cloned().collect::<Vec<_>>();
        candidate_ids.sort_unstable();
        for plugin_id in &candidate_ids {
            let plugin = staged_by_id
                .get(plugin_id)
                .expect("candidate id was collected from staged plugins");
            match discovered_runtime_bridge_method_bindings_result(plugin) {
                Ok(bindings) => {
                    prepared_bindings.insert(plugin_id.clone(), bindings);
                }
                Err(error) => {
                    set_project_activation_failure(
                        &mut selection_results,
                        plugin_id,
                        format!("runtime bridge method binding validation failed: {error}"),
                    );
                }
            }
        }
        if selection_results.iter().any(|selection| {
            selection.required
                && selection.status == NativePluginProjectActivationSelectionStatus::Failed
        }) {
            let transaction_diagnostic =
                "project activation was aborted because a required bridge binding failed";
            for selection in &mut selection_results {
                if selection.status == NativePluginProjectActivationSelectionStatus::Activated {
                    selection.status = NativePluginProjectActivationSelectionStatus::Failed;
                    selection.diagnostic = Some(format!(
                        "native plugin `{}` was not published: {transaction_diagnostic}",
                        selection.plugin_id
                    ));
                }
            }
            rejected_staged_plugins.extend(staged_by_id.into_values());
            let cleanup_receipt = self.cleanup_staged_project_plugins(rejected_staged_plugins);
            diagnostics.extend(cleanup_receipt.diagnostics.iter().cloned());
            append_selection_diagnostics(&mut diagnostics, &selection_results);
            return NativePluginProjectActivationResult {
                target: request.target,
                committed: false,
                selections: selection_results,
                load_report: NativePluginLiveHostLoadReport {
                    module_kind: PluginModuleKind::Runtime,
                    loaded_plugin_ids: Vec::new(),
                    runtime_plugin_registration_reports: registration_reports,
                    runtime_plugin_feature_registration_reports: feature_reports,
                    bridge_lifecycle_reports: Vec::new(),
                    diagnostics: sorted_diagnostics(diagnostics),
                },
                bridge_lifecycle_receipts: Vec::new(),
                cleanup_receipt,
            };
        }

        for plugin_id in candidate_ids {
            let still_selected = selection_results.iter().any(|selection| {
                selection.plugin_id == plugin_id
                    && selection.status == NativePluginProjectActivationSelectionStatus::Activated
            });
            if !still_selected {
                if let Some(plugin) = staged_by_id.remove(&plugin_id) {
                    rejected_staged_plugins.push(plugin);
                }
                prepared_bindings.remove(&plugin_id);
            }
        }

        let Ok(mut loaded) = lock_loaded_native_plugins(&self.loaded) else {
            set_all_activated_project_selections_failed(
                &mut selection_results,
                "native plugin live host is unavailable",
            );
            rejected_staged_plugins.extend(staged_by_id.into_values());
            let cleanup_receipt = self.cleanup_staged_project_plugins(rejected_staged_plugins);
            diagnostics.extend(cleanup_receipt.diagnostics.iter().cloned());
            append_selection_diagnostics(&mut diagnostics, &selection_results);
            return NativePluginProjectActivationResult {
                target: request.target,
                committed: false,
                selections: selection_results,
                load_report: NativePluginLiveHostLoadReport {
                    module_kind: PluginModuleKind::Runtime,
                    loaded_plugin_ids: Vec::new(),
                    runtime_plugin_registration_reports: registration_reports,
                    runtime_plugin_feature_registration_reports: feature_reports,
                    bridge_lifecycle_reports: Vec::new(),
                    diagnostics: sorted_diagnostics(diagnostics),
                },
                bridge_lifecycle_receipts: Vec::new(),
                cleanup_receipt,
            };
        };
        let mut installed_bindings = self
            .runtime_bridge_method_bindings
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut transitioning = HashMap::<String, LoadedNativePlugin>::new();
        for plugin_id in staged_by_id.keys() {
            let key = live_key(PluginModuleKind::Runtime, plugin_id);
            if let Some(existing) = loaded.get(&key) {
                if let Err(error) = existing.begin_lifecycle_transition() {
                    for previous in transitioning.values() {
                        previous.cancel_lifecycle_transition();
                    }
                    drop(installed_bindings);
                    drop(loaded);
                    set_all_activated_project_selections_failed(
                        &mut selection_results,
                        &format!("previous runtime generation is busy: {error}"),
                    );
                    rejected_staged_plugins.extend(staged_by_id.into_values());
                    let cleanup_receipt =
                        self.cleanup_staged_project_plugins(rejected_staged_plugins);
                    diagnostics.extend(cleanup_receipt.diagnostics.iter().cloned());
                    append_selection_diagnostics(&mut diagnostics, &selection_results);
                    return NativePluginProjectActivationResult {
                        target: request.target,
                        committed: false,
                        selections: selection_results,
                        load_report: NativePluginLiveHostLoadReport {
                            module_kind: PluginModuleKind::Runtime,
                            loaded_plugin_ids: Vec::new(),
                            runtime_plugin_registration_reports: registration_reports,
                            runtime_plugin_feature_registration_reports: feature_reports,
                            bridge_lifecycle_reports: Vec::new(),
                            diagnostics: sorted_diagnostics(diagnostics),
                        },
                        bridge_lifecycle_receipts: Vec::new(),
                        cleanup_receipt,
                    };
                }
                transitioning.insert(plugin_id.clone(), existing.clone());
            }
        }

        let mut bridge_lifecycle_receipts = Vec::new();
        let mut activated_for_rollback = Vec::new();
        let mut bridge_failure_required = false;
        if let Some(lifecycle) = request.bridge_lifecycle {
            let mut bridge_candidate_ids = staged_by_id.keys().cloned().collect::<Vec<_>>();
            bridge_candidate_ids.sort_unstable();
            for plugin_id in &bridge_candidate_ids {
                let Some(provider_id) = provider_ids_by_plugin.get(plugin_id) else {
                    continue;
                };
                let plugin = staged_by_id
                    .get(plugin_id)
                    .expect("bridge candidate remains staged until lifecycle gates finish");
                let was_enabled = bridge_provider_was_enabled(lifecycle, plugin);
                let event =
                    RuntimePluginBridgeLifecycleEvent::activate_provider(provider_id.clone());
                let outcome = lifecycle.apply_provider_lifecycle_event(event.clone());
                let receipt = super::reports::NativePluginLiveHostBridgeLifecycleReport {
                    plugin_id: plugin_id.clone(),
                    module_kind: PluginModuleKind::Runtime,
                    command: super::reports::NativePluginLiveHostCommand::Load,
                    event,
                    outcome,
                };
                diagnostics.push(receipt.diagnostic());
                if receipt.is_applied() {
                    if !was_enabled {
                        activated_for_rollback.push(provider_id.clone());
                    }
                } else {
                    set_project_activation_failure(
                        &mut selection_results,
                        plugin_id,
                        format!("runtime bridge lifecycle failed: {}", receipt.diagnostic()),
                    );
                    bridge_failure_required |= selection_results.iter().any(|selection| {
                        selection.plugin_id.as_str() == plugin_id.as_str()
                            && selection.required
                            && selection.status
                                == NativePluginProjectActivationSelectionStatus::Failed
                    });
                }
                bridge_lifecycle_receipts.push(receipt);
            }
        }

        if bridge_failure_required {
            let mut rollback_ids = Vec::new();
            let mut rollback_failed = Vec::new();
            if let Some(lifecycle) = request.bridge_lifecycle {
                for provider_id in activated_for_rollback.into_iter().rev() {
                    let outcome = lifecycle.apply_provider_lifecycle_event(
                        RuntimePluginBridgeLifecycleEvent::deactivate_provider(provider_id.clone()),
                    );
                    if outcome.is_applied() {
                        diagnostics.push(format!(
                            "project activation rolled back bridge provider `{provider_id}`"
                        ));
                    } else {
                        rollback_ids.push(provider_id.clone());
                        rollback_failed.push(format!(
                            "project activation could not roll back bridge provider `{provider_id}`: {}",
                            outcome.diagnostic()
                        ));
                    }
                }
            }
            for previous in transitioning.values() {
                previous.cancel_lifecycle_transition();
            }
            drop(installed_bindings);
            drop(loaded);
            set_all_activated_project_selections_failed(
                &mut selection_results,
                "project activation was aborted because a required bridge lifecycle gate failed",
            );
            if !rollback_ids.is_empty() {
                let retained_plugins = staged_by_id.into_values().collect::<Vec<_>>();
                let mut retained_plugins = retained_plugins;
                retained_plugins.extend(rejected_staged_plugins);
                self.project_activation_recovery
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(
                        PendingNativePluginProjectActivationRecovery::BridgeRollback {
                            lifecycle: request
                                .bridge_lifecycle
                                .expect("bridge failure requires lifecycle state")
                                .clone(),
                            plugin_ids: rollback_ids.clone(),
                            retained_plugins,
                            diagnostic: rollback_failed.join("; "),
                        },
                    );
                let mut cleanup_receipt = NativePluginProjectActivationCleanupReceipt {
                    pending_bridge_rollback_plugin_ids: rollback_ids,
                    diagnostics: rollback_failed,
                    ..NativePluginProjectActivationCleanupReceipt::default()
                };
                cleanup_receipt
                    .pending_bridge_rollback_plugin_ids
                    .sort_unstable();
                append_selection_diagnostics(&mut diagnostics, &selection_results);
                return NativePluginProjectActivationResult {
                    target: request.target,
                    committed: false,
                    selections: selection_results,
                    load_report: NativePluginLiveHostLoadReport {
                        module_kind: PluginModuleKind::Runtime,
                        loaded_plugin_ids: Vec::new(),
                        runtime_plugin_registration_reports: registration_reports,
                        runtime_plugin_feature_registration_reports: feature_reports,
                        bridge_lifecycle_reports: bridge_lifecycle_receipts.clone(),
                        diagnostics: sorted_diagnostics(diagnostics),
                    },
                    bridge_lifecycle_receipts,
                    cleanup_receipt,
                };
            }
            rejected_staged_plugins.extend(staged_by_id.into_values());
            let cleanup_receipt = self.cleanup_staged_project_plugins(rejected_staged_plugins);
            diagnostics.extend(cleanup_receipt.diagnostics.iter().cloned());
            append_selection_diagnostics(&mut diagnostics, &selection_results);
            return NativePluginProjectActivationResult {
                target: request.target,
                committed: false,
                selections: selection_results,
                load_report: NativePluginLiveHostLoadReport {
                    module_kind: PluginModuleKind::Runtime,
                    loaded_plugin_ids: Vec::new(),
                    runtime_plugin_registration_reports: registration_reports,
                    runtime_plugin_feature_registration_reports: feature_reports,
                    bridge_lifecycle_reports: bridge_lifecycle_receipts.clone(),
                    diagnostics: sorted_diagnostics(diagnostics),
                },
                bridge_lifecycle_receipts,
                cleanup_receipt,
            };
        }

        let bridge_failed_ids = staged_by_id
            .keys()
            .filter(|plugin_id| {
                selection_results.iter().any(|selection| {
                    selection.plugin_id.as_str() == plugin_id.as_str()
                        && selection.status == NativePluginProjectActivationSelectionStatus::Failed
                })
            })
            .cloned()
            .collect::<Vec<_>>();
        for plugin_id in bridge_failed_ids {
            if let Some(plugin) = staged_by_id.remove(&plugin_id) {
                rejected_staged_plugins.push(plugin);
            }
            prepared_bindings.remove(&plugin_id);
            if let Some(previous) = transitioning.remove(&plugin_id) {
                previous.cancel_lifecycle_transition();
            }
        }

        let mut commit_ids = staged_by_id.keys().cloned().collect::<Vec<_>>();
        commit_ids.sort_unstable();
        let mut committed_plugin_ids = Vec::with_capacity(staged_by_id.len());
        let mut retired_plugins = Vec::new();
        for plugin_id in commit_ids {
            let plugin = staged_by_id
                .remove(&plugin_id)
                .expect("commit id was collected from staged plugins");
            if let Some(bindings) = prepared_bindings.remove(&plugin_id).flatten() {
                installed_bindings
                    .insert(live_key(PluginModuleKind::Runtime, &plugin_id), bindings);
            } else {
                installed_bindings.remove(&live_key(PluginModuleKind::Runtime, &plugin_id));
            }
            let old = loaded.insert(live_key(PluginModuleKind::Runtime, &plugin_id), plugin);
            if let Some(old) = old {
                retired_plugins.push(old);
            }
            self.invalidate_runtime_registration_replay_generation(&plugin_id);
            committed_plugin_ids.push(plugin_id);
        }
        drop(installed_bindings);
        drop(loaded);

        let mut cleanup_receipt = self.cleanup_staged_project_plugins(rejected_staged_plugins);
        for old in retired_plugins {
            match diagnostics_from_behavior_report(
                "runtime plugin old generation retirement",
                unload_behavior(&old, PluginModuleKind::Runtime),
            ) {
                Ok(retirement_diagnostics) => diagnostics.extend(retirement_diagnostics),
                Err(error) => {
                    let plugin_id = old.plugin_id.clone();
                    cleanup_receipt.retained_plugin_ids.push(plugin_id.clone());
                    cleanup_receipt.diagnostics.push(format!(
                        "runtime plugin `{plugin_id}` old generation retirement is pending: {error}"
                    ));
                    self.project_activation_recovery
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push(
                            PendingNativePluginProjectActivationRecovery::RetainedPlugin {
                                plugin: old,
                                module_kind: PluginModuleKind::Runtime,
                                disposition: "old generation retirement retry",
                                diagnostic: format!(
                                    "runtime plugin `{plugin_id}` retirement failed"
                                ),
                            },
                        );
                }
            }
        }
        diagnostics.extend(cleanup_receipt.diagnostics.iter().cloned());
        append_selection_diagnostics(&mut diagnostics, &selection_results);
        committed_plugin_ids.sort_unstable();
        cleanup_receipt.retained_plugin_ids.sort_unstable();
        cleanup_receipt.retained_plugin_ids.dedup();
        cleanup_receipt.diagnostics.sort_unstable();
        cleanup_receipt.diagnostics.dedup();
        NativePluginProjectActivationResult {
            target: request.target,
            committed: !committed_plugin_ids.is_empty(),
            selections: selection_results,
            load_report: NativePluginLiveHostLoadReport {
                module_kind: PluginModuleKind::Runtime,
                loaded_plugin_ids: committed_plugin_ids,
                runtime_plugin_registration_reports: registration_reports,
                runtime_plugin_feature_registration_reports: feature_reports,
                bridge_lifecycle_reports: bridge_lifecycle_receipts.clone(),
                diagnostics: sorted_diagnostics(diagnostics),
            },
            bridge_lifecycle_receipts,
            cleanup_receipt,
        }
    }

    fn cleanup_staged_project_plugins(
        &self,
        plugins: Vec<LoadedNativePlugin>,
    ) -> NativePluginProjectActivationCleanupReceipt {
        let mut receipt = NativePluginProjectActivationCleanupReceipt::default();
        let mut retained = Vec::new();
        for plugin in plugins {
            match diagnostics_from_behavior_report(
                "runtime plugin staged activation cleanup",
                unload_behavior(&plugin, PluginModuleKind::Runtime),
            ) {
                Ok(diagnostics) => receipt.diagnostics.extend(diagnostics),
                Err(error) => {
                    let plugin_id = plugin.plugin_id.clone();
                    receipt.retained_plugin_ids.push(plugin_id.clone());
                    receipt.diagnostics.push(format!(
                        "runtime plugin `{plugin_id}` staged cleanup is pending: {error}"
                    ));
                    retained.push(
                        PendingNativePluginProjectActivationRecovery::RetainedPlugin {
                            plugin,
                            module_kind: PluginModuleKind::Runtime,
                            disposition: "staged candidate cleanup retry",
                            diagnostic: format!(
                                "runtime plugin `{plugin_id}` staged cleanup failed"
                            ),
                        },
                    );
                }
            }
        }
        if !retained.is_empty() {
            self.project_activation_recovery
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .extend(retained);
        }
        receipt.retained_plugin_ids.sort_unstable();
        receipt.retained_plugin_ids.dedup();
        receipt.diagnostics.sort_unstable();
        receipt.diagnostics.dedup();
        receipt
    }
}

fn validate_project_activation_bridge_provider(
    plugin_id: &str,
    plugin: &LoadedNativePlugin,
    lifecycle: Option<&RuntimePluginBridgeLifecycleState>,
) -> Result<Option<String>, String> {
    let Some(entry) = plugin.runtime_entry_report.as_ref() else {
        return Ok(None);
    };
    let Some(manifest) = entry.package_manifest.as_ref() else {
        if entry.bridge_method_bindings.is_empty() {
            return Ok(None);
        }
        return Err("bridge method bindings have no installed package manifest".to_string());
    };
    let mut interface_ids = manifest
        .provides_interfaces
        .iter()
        .map(|interface| interface.id.clone())
        .collect::<Vec<_>>();
    let binding_interface_ids = entry
        .bridge_method_bindings
        .iter()
        .map(|binding| binding.interface_id().to_string())
        .collect::<Vec<_>>();
    if binding_interface_ids
        .iter()
        .any(|interface_id| !interface_ids.contains(interface_id))
    {
        return Err(
            "native bridge method bindings are not declared by the installed package".to_string(),
        );
    }
    if interface_ids.is_empty() {
        return Ok(None);
    }
    interface_ids.sort_unstable();
    interface_ids.dedup();

    let lifecycle = lifecycle.ok_or_else(|| {
        "runtime bridge lifecycle is unavailable for a bridge-bearing selection".to_string()
    })?;
    let registry = &lifecycle.extension_report().registry;
    for interface_id in interface_ids {
        let snapshot = lifecycle
            .bridge_table()
            .interface_snapshot_by_id(&interface_id)
            .ok_or_else(|| {
                format!("runtime bridge lifecycle has no interface slot for `{interface_id}`")
            })?;
        let owner = registry
            .plugin_interfaces()
            .find_map(|(owner, export)| (export.interface_id() == interface_id).then_some(owner))
            .ok_or_else(|| {
                format!("runtime bridge lifecycle has no registered provider for `{interface_id}`")
            })?;
        if snapshot.owner != owner {
            return Err(format!(
                "runtime bridge slot owner does not match the registered provider for `{interface_id}`"
            ));
        }
        let module_name = registry.plugin_module_name(owner).ok_or_else(|| {
            format!("runtime bridge provider module is missing for `{interface_id}`")
        })?;
        let provider_id = lifecycle
            .provider_package_id_for_runtime_module(module_name)
            .ok_or_else(|| {
                format!("runtime bridge provider package is missing for `{interface_id}`")
            })?;
        if provider_id != plugin_id {
            return Err(format!(
                "runtime bridge interface `{interface_id}` belongs to `{provider_id}`, not selected package `{plugin_id}`"
            ));
        }
    }
    Ok(Some(plugin_id.to_string()))
}

fn bridge_provider_was_enabled(
    lifecycle: &RuntimePluginBridgeLifecycleState,
    plugin: &LoadedNativePlugin,
) -> bool {
    let Some(interfaces) = plugin
        .runtime_entry_report
        .as_ref()
        .and_then(|entry| entry.package_manifest.as_ref())
        .map(|manifest| &manifest.provides_interfaces)
    else {
        return false;
    };
    !interfaces.is_empty()
        && interfaces.iter().all(|interface| {
            lifecycle
                .bridge_table()
                .interface_snapshot_by_id(&interface.id)
                .is_some_and(|snapshot| {
                    snapshot.provider_installed && snapshot.status == BridgeInterfaceStatus::Enabled
                })
        })
}

fn set_project_activation_failure(
    selections: &mut [NativePluginProjectActivationSelectionResult],
    plugin_id: &str,
    reason: String,
) {
    for selection in selections
        .iter_mut()
        .filter(|selection| selection.plugin_id == plugin_id)
    {
        selection.status = NativePluginProjectActivationSelectionStatus::Failed;
        selection.diagnostic = Some(format!(
            "native plugin `{plugin_id}` runtime activation failed: {reason}"
        ));
    }
}

fn set_all_activated_project_selections_failed(
    selections: &mut [NativePluginProjectActivationSelectionResult],
    reason: &str,
) {
    for selection in selections.iter_mut().filter(|selection| {
        selection.status == NativePluginProjectActivationSelectionStatus::Activated
    }) {
        selection.status = NativePluginProjectActivationSelectionStatus::Failed;
        selection.diagnostic = Some(format!(
            "native plugin `{}` runtime activation failed: {reason}",
            selection.plugin_id
        ));
    }
}

fn append_selection_diagnostics(
    diagnostics: &mut Vec<String>,
    selections: &[NativePluginProjectActivationSelectionResult],
) {
    diagnostics.extend(
        selections
            .iter()
            .filter_map(|selection| selection.diagnostic.clone()),
    );
}

fn sorted_diagnostics(mut diagnostics: Vec<String>) -> Vec<String> {
    diagnostics.sort_unstable();
    diagnostics.dedup();
    diagnostics
}

fn retain_failed_activation_cleanup(
    plugin: LoadedNativePlugin,
    module_kind: PluginModuleKind,
    disposition: &'static str,
    diagnostic: &str,
    receipt: &mut NativePluginProjectActivationCleanupReceipt,
    remaining: &mut Vec<PendingNativePluginProjectActivationRecovery>,
) {
    let cleanup = diagnostics_from_behavior_report(
        &format!("project activation {disposition}"),
        unload_behavior(&plugin, module_kind),
    );
    match cleanup {
        Ok(diagnostics) => receipt.diagnostics.extend(diagnostics),
        Err(error) => {
            let plugin_id = plugin.plugin_id.clone();
            receipt.retained_plugin_ids.push(plugin_id.clone());
            receipt.diagnostics.push(format!(
                "{diagnostic}; retained plugin `{plugin_id}` cleanup retry failed: {error}"
            ));
            remaining.push(
                PendingNativePluginProjectActivationRecovery::RetainedPlugin {
                    plugin,
                    module_kind,
                    disposition,
                    diagnostic: diagnostic.to_string(),
                },
            );
        }
    }
}

pub(super) fn lock_loaded_native_plugins(
    loaded: &ObservedLoadedNativePlugins,
) -> NativePluginLiveHostLoadingResult<MutexGuard<'_, NativePluginLiveRegistry<LoadedNativePlugin>>>
{
    loaded
        .lock()
        .map_err(|_| NativePluginLiveHostLoadingError::LiveHostLockPoisoned)
}

#[cfg(test)]
#[path = "tests/loading_project_activation_transaction_tests.rs"]
mod project_activation_transaction_tests;
