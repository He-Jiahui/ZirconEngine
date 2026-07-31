use super::super::super::editor_capabilities::EditorCapabilitySnapshot;
use super::super::super::editor_manager::EditorManager;
use super::super::super::editor_subsystems::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY;
use zircon_runtime::core::CoreHandle;

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
    ) -> Result<(CoreHandle, EditorCapabilitySnapshot, Vec<String>), String> {
        let core = self
            .host
            .runtime_core()
            .map_err(|error| error.to_string())?;
        let previous_capabilities = core
            .load_config::<Vec<String>>(EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY)
            .unwrap_or_default();
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
        core.store_config_value(
            EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
            serde_json::json!(capabilities),
        );
        match self.host.refresh_capabilities_from_core(&core) {
            Ok(snapshot) => Ok((core, snapshot, previous_capabilities)),
            Err(error) => {
                core.store_config_value(
                    EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
                    serde_json::json!(&previous_capabilities),
                );
                match self.host.refresh_capabilities_from_core(&core) {
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
        core: &CoreHandle,
        previous_capabilities: &[String],
    ) -> Result<(), String> {
        core.store_config_value(
            EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
            serde_json::json!(previous_capabilities),
        );
        self.host
            .refresh_capabilities_from_core(core)
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
        let (core, snapshot, previous_capabilities) =
            self.set_editor_capabilities_with_previous(capabilities, enabled)?;
        if let Err(error) = self.update_editor_plugin_state_unpublished(plugin_id, enabled) {
            return match self.restore_editor_capabilities(&core, &previous_capabilities) {
                Ok(()) => Err(error),
                Err(rollback_error) => Err(format!(
                    "{error}; restoring editor capabilities after state publication failed: {rollback_error}"
                )),
            };
        }
        Ok(snapshot)
    }
}
