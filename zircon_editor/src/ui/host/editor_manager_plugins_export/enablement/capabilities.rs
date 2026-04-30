use super::super::super::editor_capabilities::EditorCapabilitySnapshot;
use super::super::super::editor_manager::EditorManager;
use super::super::super::editor_subsystems::EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY;

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
        let mut capabilities = self
            .host
            .core
            .load_config::<Vec<String>>(EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY)
            .unwrap_or_default();
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
        self.host.core.store_config_value(
            EDITOR_ENABLED_SUBSYSTEMS_CONFIG_KEY,
            serde_json::json!(capabilities),
        );
        self.host
            .refresh_capabilities()
            .map_err(|error| error.to_string())
    }

    pub fn set_editor_plugin_enabled(
        &self,
        plugin_id: &str,
        enabled: bool,
    ) -> Result<EditorCapabilitySnapshot, String> {
        let capabilities = self
            .editor_plugin_catalog()
            .capabilities_for_package(plugin_id);
        if capabilities.is_empty() {
            return Err(format!("plugin {plugin_id} has no editor capabilities"));
        }
        self.set_editor_capabilities_enabled(&capabilities, enabled)
    }
}
