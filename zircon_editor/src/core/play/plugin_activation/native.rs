use std::{path::Path, sync::Mutex};

use zircon_runtime::plugin::native::{NativePluginHostHandle, NativePluginRuntimePlayModeSnapshot};
use zircon_runtime::plugin::RuntimePluginBridgeLifecycleState;

use super::{PluginBridgeActivation, PluginBridgeActivationReport};

pub struct NativePluginBridgeActivation {
    live_host: NativePluginHostHandle,
    bridge_lifecycle: Option<RuntimePluginBridgeLifecycleState>,
    transition_gate: Mutex<()>,
    active_snapshot: Mutex<Option<NativePluginRuntimePlayModeSnapshot>>,
}

impl NativePluginBridgeActivation {
    pub fn new(live_host: NativePluginHostHandle) -> Self {
        Self {
            live_host,
            bridge_lifecycle: None,
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
            transition_gate: Mutex::new(()),
            active_snapshot: Mutex::new(None),
        }
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
            let load = if let Some(bridge_lifecycle) = &self.bridge_lifecycle {
                self.live_host
                    .load_runtime_plugins_from_project_root_with_bridge_lifecycle(
                        project_root,
                        bridge_lifecycle,
                    )?
            } else {
                self.live_host
                    .load_runtime_plugins_from_project_root(project_root)?
            };
            diagnostics.extend(load.diagnostics);
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

#[cfg(test)]
mod performance_source_guards {
    #[test]
    fn successful_deactivation_moves_the_snapshot_instead_of_cloning_it() {
        let source = include_str!("native.rs");
        let body = source
            .split("fn deactivate")
            .nth(1)
            .and_then(|body| body.split("#[cfg(test)]").next())
            .expect("deactivate body should remain available");

        assert!(!body.contains(".clone()"));
    }
}
