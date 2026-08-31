use crate::plugin::PluginModuleKind;

use super::super::behavior_calls::NativePluginBehavior;
use super::callbacks::validate_callbacks;
use super::diagnostics::{health_from_diagnostics, module_kind_label};
use super::schema::{
    expected_registration_manifest_schema, has_manifest_text, validate_v3_schema,
    ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V4, ZIRCON_NATIVE_EVENT_MANIFEST_SCHEMA_V3,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativePluginBehaviorHealth {
    Clean,
    Degraded,
    Invalid,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativePluginBehaviorValidationReport {
    pub abi_version: u32,
    pub module_kind: PluginModuleKind,
    pub plugin_id: String,
    pub is_stateless: Option<bool>,
    pub state_schema_version: Option<u32>,
    pub command_manifest_schema: Option<String>,
    pub event_manifest_schema: Option<String>,
    pub registration_manifest_schema: Option<String>,
    pub has_command_manifest: bool,
    pub has_event_manifest: bool,
    pub has_registration_manifest: bool,
    pub has_invoke_command: bool,
    pub has_save_state: bool,
    pub has_restore_state: bool,
    pub has_unload: bool,
    pub diagnostics: Vec<String>,
    pub health: NativePluginBehaviorHealth,
}

impl NativePluginBehaviorValidationReport {
    pub(in crate::plugin::native_plugin_loader) fn from_behavior(
        plugin_id: &str,
        module_kind: PluginModuleKind,
        abi_version: u32,
        behavior: Option<&NativePluginBehavior>,
    ) -> Self {
        let Some(behavior) = behavior else {
            return Self::missing_behavior(plugin_id, module_kind, abi_version);
        };

        let mut diagnostics = Vec::new();
        validate_v3_schema(
            &mut diagnostics,
            abi_version,
            plugin_id,
            module_kind,
            "command_manifest_schema",
            behavior.command_manifest_schema.as_deref(),
            behavior.command_manifest.as_deref(),
            ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V4,
        );
        validate_v3_schema(
            &mut diagnostics,
            abi_version,
            plugin_id,
            module_kind,
            "event_manifest_schema",
            behavior.event_manifest_schema.as_deref(),
            behavior.event_manifest.as_deref(),
            ZIRCON_NATIVE_EVENT_MANIFEST_SCHEMA_V3,
        );
        validate_v3_schema(
            &mut diagnostics,
            abi_version,
            plugin_id,
            module_kind,
            "registration_manifest_schema",
            behavior.registration_manifest_schema.as_deref(),
            behavior.registration_manifest.as_deref(),
            expected_registration_manifest_schema(module_kind),
        );
        validate_callbacks(&mut diagnostics, plugin_id, module_kind, behavior);

        let health = health_from_diagnostics(&diagnostics);
        Self {
            abi_version,
            module_kind,
            plugin_id: plugin_id.to_string(),
            is_stateless: Some(behavior.is_stateless),
            state_schema_version: Some(behavior.state_schema_version),
            command_manifest_schema: behavior.command_manifest_schema.clone(),
            event_manifest_schema: behavior.event_manifest_schema.clone(),
            registration_manifest_schema: behavior.registration_manifest_schema.clone(),
            has_command_manifest: has_manifest_text(behavior.command_manifest.as_deref()),
            has_event_manifest: has_manifest_text(behavior.event_manifest.as_deref()),
            has_registration_manifest: has_manifest_text(behavior.registration_manifest.as_deref()),
            has_invoke_command: behavior.has_invoke_command(),
            has_save_state: behavior.has_save_state(),
            has_restore_state: behavior.has_restore_state(),
            has_unload: behavior.has_unload(),
            diagnostics: diagnostics
                .into_iter()
                .map(|diagnostic| diagnostic.message)
                .collect(),
            health,
        }
    }

    fn missing_behavior(plugin_id: &str, module_kind: PluginModuleKind, abi_version: u32) -> Self {
        Self {
            abi_version,
            module_kind,
            plugin_id: plugin_id.to_string(),
            is_stateless: None,
            state_schema_version: None,
            command_manifest_schema: None,
            event_manifest_schema: None,
            registration_manifest_schema: None,
            has_command_manifest: false,
            has_event_manifest: false,
            has_registration_manifest: false,
            has_invoke_command: false,
            has_save_state: false,
            has_restore_state: false,
            has_unload: false,
            diagnostics: vec![format!(
                "native plugin {plugin_id} {} behavior is missing",
                module_kind_label(module_kind)
            )],
            health: NativePluginBehaviorHealth::Invalid,
        }
    }
}
