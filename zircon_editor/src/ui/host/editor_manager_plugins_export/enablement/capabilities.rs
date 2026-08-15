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
        let mut capabilities = previous_capabilities.clone();
        capabilities.retain(|existing| {
            !target_capabilities
                .iter()
                .any(|capability| capability == existing)
        });
        if enabled {
            capabilities.extend(target_capabilities.iter().cloned());
            capabilities.sort();
            capabilities.dedup();
        }
        configuration.store_enabled_subsystems(&capabilities);
        match self
            .host
            .apply_capability_report(configuration.subsystem_report())
        {
            Ok(snapshot) => Ok((configuration, snapshot, previous_capabilities)),
            Err(error) => {
                configuration.store_enabled_subsystems(&previous_capabilities);
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

    fn restore_editor_capabilities(
        &self,
        configuration: &EditorCapabilityConfiguration,
        previous_capabilities: &[String],
    ) -> Result<(), String> {
        configuration.store_enabled_subsystems(previous_capabilities);
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
