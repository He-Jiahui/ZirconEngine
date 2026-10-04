use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use zircon_runtime::core::framework::{platform::RuntimeTargetMode, project::ExportTargetPlatform};
use zircon_runtime::plugin::native::{
    host::NativePluginHostHandle, NativePluginArtifactAuthority, NativePluginArtifactTarget,
    NativePluginProjectActivationRequest, NativePluginProjectActivationSelection,
    NativePluginRuntimePlayModeSnapshot,
};
use zircon_runtime::plugin::package_service::{NativePluginAdmission, NativePluginSelectionStatus};
use zircon_runtime::plugin::RuntimePluginBridgeLifecycleState;

use super::{PluginBridgeActivation, PluginBridgeActivationReport};

pub type NativePluginArtifactAuthorityResolver =
    Arc<dyn Fn(&Path) -> Result<NativePluginArtifactAuthority, String> + Send + Sync>;

/// Resolves signed installed-package admission for an explicit Runtime or Play target.
pub type NativePluginAdmissionResolver = Arc<
    dyn Fn(&Path, NativePluginArtifactTarget) -> Result<NativePluginAdmission, String>
        + Send
        + Sync,
>;

pub struct NativePluginBridgeActivation {
    live_host: NativePluginHostHandle,
    bridge_lifecycle: Option<RuntimePluginBridgeLifecycleState>,
    authority_resolver: Option<NativePluginArtifactAuthorityResolver>,
    admission_resolver: Option<NativePluginAdmissionResolver>,
    transition_gate: Mutex<()>,
    active_snapshot: Mutex<Option<NativePluginRuntimePlayModeSnapshot>>,
}

impl NativePluginBridgeActivation {
    pub fn new(live_host: NativePluginHostHandle) -> Self {
        Self {
            live_host,
            bridge_lifecycle: None,
            authority_resolver: None,
            admission_resolver: None,
            transition_gate: Mutex::new(()),
            active_snapshot: Mutex::new(None),
        }
    }

    pub fn new_with_bridge_lifecycle(
        live_host: NativePluginHostHandle,
        bridge_lifecycle: RuntimePluginBridgeLifecycleState,
    ) -> Self {
        Self {
            live_host,
            bridge_lifecycle: Some(bridge_lifecycle),
            authority_resolver: None,
            admission_resolver: None,
            transition_gate: Mutex::new(()),
            active_snapshot: Mutex::new(None),
        }
    }

    pub fn with_authority_resolver(
        mut self,
        authority_resolver: NativePluginArtifactAuthorityResolver,
    ) -> Self {
        self.authority_resolver = Some(authority_resolver);
        self
    }

    pub fn with_admission_resolver(
        mut self,
        admission_resolver: NativePluginAdmissionResolver,
    ) -> Self {
        self.admission_resolver = Some(admission_resolver);
        self
    }
}

impl PluginBridgeActivation for NativePluginBridgeActivation {
    fn activate(
        &self,
        project_root: Option<&Path>,
    ) -> Result<PluginBridgeActivationReport, String> {
        let _transition = self
            .transition_gate
            .lock()
            .map_err(|_| "plugin bridge transition lock is poisoned".to_string())?;
        if self
            .active_snapshot
            .lock()
            .map_err(|_| "plugin bridge activation lock is poisoned".to_string())?
            .is_some()
        {
            return Err("plugin bridge activation already has an active snapshot".to_string());
        }

        let mut diagnostics = Vec::new();
        if let Some(project_root) = project_root {
            let resolver = self.admission_resolver.as_ref().ok_or_else(|| {
                if self.authority_resolver.is_some() {
                    "native plugin admission resolver is required; project-root authority is not accepted"
                } else {
                    "native plugin admission resolver is unavailable"
                }
                .to_string()
            })?;
            let admission = resolver(project_root, play_native_plugin_target())?;
            diagnostics.extend(admission.diagnostics());
            if let Some(reason) = admission.required_failure_diagnostic() {
                return Err(reason);
            }
            let selections = admission
                .outcomes()
                .iter()
                .filter(|outcome| outcome.status == NativePluginSelectionStatus::Admitted)
                .map(|outcome| NativePluginProjectActivationSelection {
                    plugin_id: outcome.plugin_id.clone(),
                    required: outcome.required,
                })
                .collect::<Vec<_>>();
            if let Some(installed_root) = admission.installed_root() {
                let result = self.live_host.activate_runtime_project_plugins(
                    NativePluginProjectActivationRequest {
                        installed_root,
                        target: play_native_plugin_target(),
                        selections: &selections,
                        bridge_lifecycle: self.bridge_lifecycle.as_ref(),
                    },
                    admission.authority(),
                );
                diagnostics.extend(result.load_report.diagnostics.iter().cloned());
                if result.has_required_failures() {
                    diagnostics.sort();
                    diagnostics.dedup();
                    return Err(format!(
                        "{}; plugin selection results: {}",
                        result.required_failure_diagnostics().join("; "),
                        diagnostics.join("; ")
                    ));
                }
            } else if !selections.is_empty() {
                let missing_root =
                    "verified native plugin selections have no installed runtime root";
                let required = selections
                    .iter()
                    .filter(|selection| selection.required)
                    .map(|selection| {
                        format!(
                            "native plugin `{}` runtime activation failed: {missing_root}",
                            selection.plugin_id
                        )
                    })
                    .collect::<Vec<_>>();
                diagnostics.extend(selections.iter().map(|selection| {
                    format!(
                        "native plugin `{}` runtime activation failed: {missing_root}",
                        selection.plugin_id
                    )
                }));
                if !required.is_empty() {
                    diagnostics.sort();
                    diagnostics.dedup();
                    return Err(format!(
                        "{}; plugin selection results: {}",
                        required.join("; "),
                        diagnostics.join("; ")
                    ));
                }
            }
        } else {
            diagnostics.push(
                "runtime native plugin load skipped because the editor project root is unavailable"
                    .to_string(),
            );
        }

        let snapshot = self.live_host.enter_runtime_play_mode()?;
        diagnostics.extend(snapshot.combined_diagnostics());
        diagnostics.sort();
        diagnostics.dedup();
        let bridge_diagnostics = self
            .bridge_lifecycle
            .as_ref()
            .map(|lifecycle| lifecycle.bridge_table().diagnostics_matrix());
        *self
            .active_snapshot
            .lock()
            .map_err(|_| "plugin bridge activation lock is poisoned".to_string())? = Some(snapshot);
        Ok(PluginBridgeActivationReport {
            diagnostics,
            bridge_diagnostics,
        })
    }

    fn deactivate(&self) -> Result<PluginBridgeActivationReport, String> {
        let _transition = self
            .transition_gate
            .lock()
            .map_err(|_| "plugin bridge transition lock is poisoned".to_string())?;
        let snapshot = self
            .active_snapshot
            .lock()
            .map_err(|_| "plugin bridge activation lock is poisoned".to_string())?
            .take();
        let Some(snapshot) = snapshot else {
            return Ok(PluginBridgeActivationReport {
                diagnostics: vec![
                    "plugin bridge activation had no active snapshot to restore".to_string()
                ],
                bridge_diagnostics: None,
            });
        };

        let report = match self.live_host.exit_runtime_play_mode(&snapshot) {
            Ok(report) => report,
            Err(error) => {
                *self
                    .active_snapshot
                    .lock()
                    .map_err(|_| "plugin bridge activation lock is poisoned".to_string())? =
                    Some(snapshot);
                return Err(error);
            }
        };
        Ok(PluginBridgeActivationReport {
            diagnostics: report.combined_diagnostics(),
            bridge_diagnostics: self
                .bridge_lifecycle
                .as_ref()
                .map(|lifecycle| lifecycle.bridge_table().diagnostics_matrix()),
        })
    }
}

fn play_native_plugin_target() -> NativePluginArtifactTarget {
    NativePluginArtifactTarget::new(
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
    )
}

#[cfg(test)]
#[path = "tests/native.rs"]
mod tests;
