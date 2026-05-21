use crate::plugin::PluginModuleKind;

use super::abi_declarations::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3;
use super::behavior_calls::NativePluginBehavior;

pub const ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V3: &str = "zircon.native.command-manifest/3";
pub const ZIRCON_NATIVE_EVENT_MANIFEST_SCHEMA_V3: &str = "zircon.native.event-manifest/3";

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
    pub has_command_manifest: bool,
    pub has_event_manifest: bool,
    pub has_invoke_command: bool,
    pub has_save_state: bool,
    pub has_restore_state: bool,
    pub has_unload: bool,
    pub diagnostics: Vec<String>,
    pub health: NativePluginBehaviorHealth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiagnosticSeverity {
    Degraded,
    Invalid,
}

struct ValidationDiagnostic {
    severity: DiagnosticSeverity,
    message: String,
}

impl NativePluginBehaviorValidationReport {
    pub(super) fn from_behavior(
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
            ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V3,
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
            has_command_manifest: has_manifest_text(behavior.command_manifest.as_deref()),
            has_event_manifest: has_manifest_text(behavior.event_manifest.as_deref()),
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
            has_command_manifest: false,
            has_event_manifest: false,
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

fn validate_v3_schema(
    diagnostics: &mut Vec<ValidationDiagnostic>,
    abi_version: u32,
    plugin_id: &str,
    module_kind: PluginModuleKind,
    field_name: &str,
    schema: Option<&str>,
    manifest: Option<&str>,
    expected_schema: &str,
) {
    if abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
        return;
    }
    let Some(schema) = schema.map(str::trim).filter(|schema| !schema.is_empty()) else {
        return;
    };
    if schema != expected_schema {
        diagnostics.push(invalid_diagnostic(format!(
            "native plugin {plugin_id} {} {field_name} is unsupported: {schema}; expected {expected_schema}",
            module_kind_label(module_kind)
        )));
        return;
    }
    if !has_manifest_text(manifest) {
        diagnostics.push(invalid_diagnostic(format!(
            "native plugin {plugin_id} {} declares {field_name} {schema} but provides no manifest text",
            module_kind_label(module_kind)
        )));
    }
}

fn validate_callbacks(
    diagnostics: &mut Vec<ValidationDiagnostic>,
    plugin_id: &str,
    module_kind: PluginModuleKind,
    behavior: &NativePluginBehavior,
) {
    if behavior.is_stateless {
        if behavior.state_schema_version != 0 {
            diagnostics.push(degraded_diagnostic(format!(
                "native plugin {plugin_id} {} is stateless but declares state schema version {}",
                module_kind_label(module_kind),
                behavior.state_schema_version
            )));
        }
    } else {
        if !behavior.has_save_state() {
            diagnostics.push(invalid_diagnostic(format!(
                "native plugin {plugin_id} {} behavior callback save_state is missing for stateful behavior",
                module_kind_label(module_kind)
            )));
        }
        if !behavior.has_restore_state() {
            diagnostics.push(invalid_diagnostic(format!(
                "native plugin {plugin_id} {} behavior callback restore_state is missing for stateful behavior",
                module_kind_label(module_kind)
            )));
        }
    }

    if !behavior.has_unload() {
        diagnostics.push(degraded_diagnostic(format!(
            "native plugin {plugin_id} {} behavior callback unload is missing",
            module_kind_label(module_kind)
        )));
    }

    if !behavior.has_invoke_command() && !has_manifest_text(behavior.command_manifest.as_deref()) {
        diagnostics.push(degraded_diagnostic(format!(
            "native plugin {plugin_id} {} behavior callback invoke_command is missing",
            module_kind_label(module_kind)
        )));
    }
}

fn has_manifest_text(manifest: Option<&str>) -> bool {
    manifest
        .into_iter()
        .flat_map(str::lines)
        .map(str::trim)
        .any(|line| !line.is_empty())
}

fn invalid_diagnostic(message: String) -> ValidationDiagnostic {
    ValidationDiagnostic {
        severity: DiagnosticSeverity::Invalid,
        message,
    }
}

fn degraded_diagnostic(message: String) -> ValidationDiagnostic {
    ValidationDiagnostic {
        severity: DiagnosticSeverity::Degraded,
        message,
    }
}

fn health_from_diagnostics(diagnostics: &[ValidationDiagnostic]) -> NativePluginBehaviorHealth {
    if diagnostics.is_empty() {
        NativePluginBehaviorHealth::Clean
    } else if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Invalid)
    {
        NativePluginBehaviorHealth::Invalid
    } else {
        NativePluginBehaviorHealth::Degraded
    }
}

fn module_kind_label(module_kind: PluginModuleKind) -> &'static str {
    match module_kind {
        PluginModuleKind::Runtime => "runtime",
        PluginModuleKind::Editor => "editor",
        PluginModuleKind::Native => "native",
        PluginModuleKind::Vm => "vm",
    }
}

#[cfg(test)]
mod tests {
    use crate::plugin::PluginModuleKind;

    use super::super::abi_declarations::{
        ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
    };
    use super::super::behavior_calls::NativePluginBehavior;
    use super::{
        NativePluginBehaviorHealth, NativePluginBehaviorValidationReport,
        ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V3, ZIRCON_NATIVE_EVENT_MANIFEST_SCHEMA_V3,
    };

    #[test]
    fn clean_v3_stateful_behavior_reports_no_diagnostics() {
        let report = validate(runtime_behavior());

        assert_eq!(report.health, NativePluginBehaviorHealth::Clean);
        assert!(report.diagnostics.is_empty());
        assert_eq!(report.is_stateless, Some(false));
        assert_eq!(report.state_schema_version, Some(3));
        assert!(report.has_command_manifest);
        assert!(report.has_event_manifest);
        assert!(report.has_invoke_command);
        assert!(report.has_save_state);
        assert!(report.has_restore_state);
        assert!(report.has_unload);
    }

    #[test]
    fn stateless_editor_behavior_may_omit_state_callbacks() {
        let report = validate(NativePluginBehavior {
            is_stateless: true,
            state_schema_version: 0,
            command_manifest_schema: Some(ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V3.to_string()),
            event_manifest_schema: Some(ZIRCON_NATIVE_EVENT_MANIFEST_SCHEMA_V3.to_string()),
            command_manifest: Some("command=open;payload=bytes".to_string()),
            event_manifest: Some("event=opened".to_string()),
            invoke_command: Some(noop_invoke_command),
            save_state: None,
            restore_state: None,
            unload: Some(noop_unload),
        });

        assert_eq!(report.health, NativePluginBehaviorHealth::Clean);
        assert!(report.diagnostics.is_empty());
        assert!(!report.has_save_state);
        assert!(!report.has_restore_state);
    }

    #[test]
    fn malformed_command_schema_marks_behavior_invalid() {
        let report = validate(NativePluginBehavior {
            command_manifest_schema: Some("zircon.native.command-manifest/99".to_string()),
            ..runtime_behavior()
        });

        assert_eq!(report.health, NativePluginBehaviorHealth::Invalid);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("command_manifest_schema is unsupported")));
    }

    #[test]
    fn malformed_event_schema_marks_behavior_invalid() {
        let report = validate(NativePluginBehavior {
            event_manifest_schema: Some("zircon.native.event-manifest/99".to_string()),
            ..runtime_behavior()
        });

        assert_eq!(report.health, NativePluginBehaviorHealth::Invalid);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("event_manifest_schema is unsupported")));
    }

    #[test]
    fn declared_schema_without_manifest_marks_behavior_invalid() {
        let report = validate(NativePluginBehavior {
            command_manifest: Some("   \n".to_string()),
            ..runtime_behavior()
        });

        assert_eq!(report.health, NativePluginBehaviorHealth::Invalid);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("provides no manifest text")));
    }

    #[test]
    fn stateful_missing_save_or_restore_marks_behavior_invalid() {
        let missing_save = validate(NativePluginBehavior {
            save_state: None,
            ..runtime_behavior()
        });
        let missing_restore = validate(NativePluginBehavior {
            restore_state: None,
            ..runtime_behavior()
        });

        assert_eq!(missing_save.health, NativePluginBehaviorHealth::Invalid);
        assert!(missing_save
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("save_state is missing")));
        assert_eq!(missing_restore.health, NativePluginBehaviorHealth::Invalid);
        assert!(missing_restore
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("restore_state is missing")));
    }

    #[test]
    fn missing_unload_or_state_only_command_callback_degrades_behavior() {
        let missing_unload = validate(NativePluginBehavior {
            unload: None,
            ..runtime_behavior()
        });
        let state_only = validate(NativePluginBehavior {
            command_manifest_schema: None,
            command_manifest: None,
            invoke_command: None,
            ..runtime_behavior()
        });

        assert_eq!(missing_unload.health, NativePluginBehaviorHealth::Degraded);
        assert!(missing_unload
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("unload is missing")));
        assert_eq!(state_only.health, NativePluginBehaviorHealth::Degraded);
        assert!(state_only
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("invoke_command is missing")));
    }

    #[test]
    fn missing_behavior_reports_invalid_without_callbacks() {
        let report = NativePluginBehaviorValidationReport::from_behavior(
            "fixture",
            PluginModuleKind::Runtime,
            ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            None,
        );

        assert_eq!(report.health, NativePluginBehaviorHealth::Invalid);
        assert_eq!(report.is_stateless, None);
        assert!(!report.has_invoke_command);
        assert!(report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.contains("runtime behavior is missing")));
    }

    #[test]
    fn abi_v2_validation_does_not_require_v3_schema_strings() {
        let report = NativePluginBehaviorValidationReport::from_behavior(
            "fixture",
            PluginModuleKind::Runtime,
            ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V2,
            Some(&NativePluginBehavior {
                command_manifest_schema: None,
                event_manifest_schema: None,
                ..runtime_behavior()
            }),
        );

        assert_eq!(report.health, NativePluginBehaviorHealth::Clean);
        assert!(report.diagnostics.is_empty());
    }

    fn validate(behavior: NativePluginBehavior) -> NativePluginBehaviorValidationReport {
        NativePluginBehaviorValidationReport::from_behavior(
            "fixture",
            PluginModuleKind::Runtime,
            ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            Some(&behavior),
        )
    }

    fn runtime_behavior() -> NativePluginBehavior {
        NativePluginBehavior {
            is_stateless: false,
            state_schema_version: 3,
            command_manifest_schema: Some(ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V3.to_string()),
            event_manifest_schema: Some(ZIRCON_NATIVE_EVENT_MANIFEST_SCHEMA_V3.to_string()),
            command_manifest: Some("command=echo;payload=bytes".to_string()),
            event_manifest: Some("event=echoed;payload=bytes".to_string()),
            invoke_command: Some(noop_invoke_command),
            save_state: Some(noop_save_state),
            restore_state: Some(noop_restore_state),
            unload: Some(noop_unload),
        }
    }

    unsafe extern "C" fn noop_invoke_command(
        _command_name: *const std::ffi::c_char,
        _payload: super::super::abi_declarations::NativePluginByteSliceV2,
        _output: *mut super::super::abi_declarations::NativePluginOwnedByteBufferV2,
    ) -> super::super::abi_declarations::NativePluginCallbackStatusV2 {
        status_ok()
    }

    unsafe extern "C" fn noop_save_state(
        _output: *mut super::super::abi_declarations::NativePluginOwnedByteBufferV2,
    ) -> super::super::abi_declarations::NativePluginCallbackStatusV2 {
        status_ok()
    }

    unsafe extern "C" fn noop_restore_state(
        _state: super::super::abi_declarations::NativePluginByteSliceV2,
    ) -> super::super::abi_declarations::NativePluginCallbackStatusV2 {
        status_ok()
    }

    unsafe extern "C" fn noop_unload(
    ) -> super::super::abi_declarations::NativePluginCallbackStatusV2 {
        status_ok()
    }

    fn status_ok() -> super::super::abi_declarations::NativePluginCallbackStatusV2 {
        super::super::abi_declarations::NativePluginCallbackStatusV2 {
            code: super::super::abi_declarations::ZIRCON_NATIVE_PLUGIN_STATUS_OK,
            diagnostics: std::ptr::null(),
        }
    }
}
