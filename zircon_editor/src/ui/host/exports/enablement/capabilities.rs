use std::collections::HashSet;

use super::super::super::editor_capabilities::EditorCapabilitySnapshot;
use super::super::super::editor_manager::EditorManager;
use super::super::super::runtime_services::EditorCapabilityConfiguration;

impl EditorManager {
    pub fn set_editor_subsystem_enabled(
        &self,
        capability: &str,
        enabled: bool,
    ) -> Result<EditorCapabilitySnapshot, String> {
        self.set_editor_capabilities_enabled(&[capability.to_string()], enabled)
    }

    pub fn set_editor_capabilities_enabled(
        &self,
        target_capabilities: &[String],
        enabled: bool,
    ) -> Result<EditorCapabilitySnapshot, String> {
        let _transaction = self.lock_editor_capability_updates();
        self.set_editor_capabilities_with_previous(target_capabilities, enabled)
            .map(|(_, snapshot, _)| snapshot)
    }

    fn set_editor_capabilities_with_previous(
        &self,
        target_capabilities: &[String],
        enabled: bool,
    ) -> Result<
        (
            EditorCapabilityConfiguration,
            EditorCapabilitySnapshot,
            Vec<String>,
        ),
        String,
    > {
        let configuration = self
            .host
            .runtime_services
            .capability_configuration()
            .map_err(|error| error.to_string())?;
        let previous_capabilities = configuration.enabled_subsystems();
        let capabilities =
            project_editor_capabilities(&previous_capabilities, target_capabilities, enabled);
        configuration
            .store_enabled_subsystems(&capabilities)
            .map_err(|error| error.to_string())?;
        match self
            .host
            .apply_capability_report(configuration.subsystem_report())
        {
            Ok(snapshot) => Ok((configuration, snapshot, previous_capabilities)),
            Err(error) => {
                if let Err(restore_error) =
                    configuration.store_enabled_subsystems(&previous_capabilities)
                {
                    return Err(format!(
                        "{error}; restoring editor capabilities failed: {restore_error}"
                    ));
                }
                match self
                    .host
                    .apply_capability_report(configuration.subsystem_report())
                {
                    Ok(_) => Err(error.to_string()),
                    Err(rollback_error) => Err(format!(
                        "{error}; restoring editor capabilities failed: {rollback_error}"
                    )),
                }
            }
        }
    }

    /// Only runtime configuration/lifecycle work is inside this guard. Manifest, receipt and
    /// source-revision I/O belong to transaction.rs outside it.
    pub(super) fn apply_bound_plugin_enablement(
        &self,
        plugin_id: &str,
        enabled: bool,
        targets: &[String],
        owns_editor_state: bool,
        binding: &super::transaction::RuntimeEnablementBinding,
    ) -> Result<EditorCapabilitySnapshot, super::transaction::RuntimeApplyFailure> {
        use super::super::reports::PluginEnablementCauseKind as Kind;
        use super::transaction::RuntimeApplyFailure as Failure;
        // Discovery identity resolution can perform filesystem reads; do it before the guard.
        if zircon_runtime::plugin::native::discovery::native_plugin_discovery_generation(
            &binding.native_root,
        ) != binding.native_generation
        {
            return Err(Failure {
                kind: Kind::CatalogChanged,
                primary: "native catalog changed".into(),
                rollback: None,
                unknown: false,
            });
        }
        let _transaction = self.lock_editor_capability_updates();
        let state = self.plugin_manager().state_snapshot();
        let configuration = self
            .host
            .runtime_services
            .capability_configuration()
            .map_err(|e| Failure {
                kind: Kind::CapabilityApply,
                primary: e.to_string(),
                rollback: None,
                unknown: false,
            })?;
        if state.generation() != binding.manager_generation
            || state.catalog_generation() != binding.catalog_generation
            || configuration.enabled_subsystems() != binding.capabilities
            || self.capability_snapshot() != binding.snapshot
        {
            return Err(Failure {
                kind: Kind::CatalogChanged,
                primary: "runtime or catalog changed after preparation".into(),
                rollback: None,
                unknown: false,
            });
        }
        if owns_editor_state {
            self.validate_editor_plugin_state(plugin_id, enabled)
                .map_err(|e| Failure {
                    kind: Kind::LifecycleApply,
                    primary: e,
                    rollback: None,
                    unknown: false,
                })?;
        }
        // Catch inside the guard: a plugin panic must not poison the capability mutex.
        // Its logical effects remain unproven and require runtime repair even after caps restore.
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if !targets.is_empty() {
                let desired = project_editor_capabilities(&binding.capabilities, targets, enabled);
                configuration
                    .store_enabled_subsystems(&desired)
                    .map_err(|e| Failure {
                        kind: Kind::CapabilityApply,
                        primary: e.to_string(),
                        rollback: None,
                        unknown: false,
                    })?;
                self.host
                    .apply_capability_report(configuration.subsystem_report())
                    .map_err(|e| Failure {
                        kind: Kind::CapabilityApply,
                        primary: e.to_string(),
                        rollback: None,
                        unknown: false,
                    })?;
            }
            #[cfg(test)]
            self.run_enablement_runtime_checkpoint_for_test();
            if self.plugin_manager().state_snapshot().generation() != binding.manager_generation {
                return Err(Failure {
                    kind: Kind::CatalogChanged,
                    primary: "lifecycle generation changed before publication".into(),
                    rollback: None,
                    unknown: true,
                });
            }
            if owns_editor_state {
                self.update_editor_plugin_state_unpublished(plugin_id, enabled)
                    .map_err(|e| Failure {
                        kind: Kind::LifecycleApply,
                        primary: e,
                        rollback: None,
                        unknown: true,
                    })?;
            }
            Ok(self.capability_snapshot())
        }));
        let mut failure = match outcome {
            Ok(Ok(snapshot)) => return Ok(snapshot),
            Ok(Err(failure)) => failure,
            Err(_) => Failure {
                kind: Kind::Panic,
                primary: "runtime enablement panicked".into(),
                rollback: None,
                unknown: true,
            },
        };
        if configuration.enabled_subsystems() != binding.capabilities
            || self.capability_snapshot() != binding.snapshot
        {
            let rollback = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.restore_editor_capabilities(&configuration, &binding.capabilities)
            }));
            match rollback {
                Ok(Ok(())) if self.capability_snapshot() == binding.snapshot => {}
                Ok(Ok(())) => {
                    failure.rollback = Some("restored capability snapshot differs".into());
                    failure.unknown = true;
                }
                Ok(Err(e)) => {
                    failure.rollback = Some(e);
                    failure.unknown = true;
                }
                Err(_) => {
                    failure.rollback = Some("capability rollback panicked".into());
                    failure.unknown = true;
                }
            }
        }
        Err(failure)
    }

    fn restore_editor_capabilities(
        &self,
        configuration: &EditorCapabilityConfiguration,
        previous_capabilities: &[String],
    ) -> Result<(), String> {
        configuration
            .store_enabled_subsystems(previous_capabilities)
            .map_err(|error| error.to_string())?;
        self.host
            .apply_capability_report(configuration.subsystem_report())
            .map(|_| ())
            .map_err(|error| error.to_string())
    }

    pub fn set_editor_plugin_enabled(
        &self,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<EditorCapabilitySnapshot, String> {
        let snapshot = self.set_editor_plugin_enabled_unpublished(plugin_id, enabled)?;
        self.refresh_builtin_plugin_status();
        Ok(snapshot)
    }

    pub(in crate::ui::host) fn set_editor_plugin_enabled_unpublished(
        &self,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<EditorCapabilitySnapshot, String> {
        let _transaction = self.lock_editor_capability_updates();
        let catalog = self.editor_plugin_catalog();
        let capabilities = catalog.capabilities_for_package(plugin_id);
        if capabilities.is_empty() {
            return Err(format!("plugin {plugin_id} has no editor capabilities"));
        }
        self.validate_editor_plugin_state(plugin_id, enabled)?;
        let (configuration, snapshot, previous_capabilities) =
            self.set_editor_capabilities_with_previous(capabilities, enabled)?;
        if let Err(error) = self.update_editor_plugin_state_unpublished(plugin_id, enabled) {
            return match self.restore_editor_capabilities(&configuration, &previous_capabilities) {
                Ok(()) => Err(error),
                Err(rollback_error) => Err(format!(
                    "{error}; restoring editor capabilities after state publication failed: {rollback_error}"
                )),
            };
        }
        Ok(snapshot)
    }
}

fn project_editor_capabilities(
    previous: &[String],
    targets: &[String],
    enabled: bool,
) -> Vec<String> {
    let target_index: HashSet<&str> = targets.iter().map(String::as_str).collect();
    let extra_capacity = if enabled { targets.len() } else { 0 };
    let mut capabilities = Vec::with_capacity(previous.len() + extra_capacity);
    capabilities.extend(
        previous
            .iter()
            .filter(|existing| !target_index.contains(existing.as_str()))
            .cloned(),
    );
    if enabled {
        capabilities.extend(targets.iter().cloned());
        capabilities.sort();
        capabilities.dedup();
    }
    capabilities
}

#[cfg(test)]
#[path = "tests/indexed_projection.rs"]
mod indexed_projection_tests;
