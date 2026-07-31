use crate::plugin::PluginModuleKind;

use super::super::abi_declarations::{
    NativePluginByteSliceV2, NativePluginCallbackStatusV2, NativePluginOutputSinkV4,
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use super::super::behavior_calls::{NativePluginBehavior, NativePluginCommandTable};
use super::schema::{
    ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V4, ZIRCON_NATIVE_EVENT_MANIFEST_SCHEMA_V3,
    ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3,
};
use super::{NativePluginBehaviorHealth, NativePluginBehaviorValidationReport};
use zircon_runtime_interface::SERIALIZED_EDITOR_CONTRIBUTION_BATCH_SCHEMA_V1;

#[test]
fn clean_v4_stateful_behavior_reports_no_diagnostics() {
    let report = validate(runtime_behavior());

    assert_eq!(report.health, NativePluginBehaviorHealth::Clean);
    assert!(report.diagnostics.is_empty());
    assert_eq!(report.is_stateless, Some(false));
    assert_eq!(report.state_schema_version, Some(3));
    assert!(report.has_command_manifest);
    assert!(report.has_event_manifest);
    assert!(report.has_registration_manifest);
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
        command_manifest_schema: Some(ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V4.to_string()),
        event_manifest_schema: Some(ZIRCON_NATIVE_EVENT_MANIFEST_SCHEMA_V3.to_string()),
        registration_manifest_schema: None,
        command_manifest: Some(command_manifest().to_string()),
        event_manifest: Some("event=opened".to_string()),
        registration_manifest: None,
        command_table: Some(command_table()),
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
fn malformed_registration_schema_marks_behavior_invalid() {
    let report = validate(NativePluginBehavior {
        registration_manifest_schema: Some("zircon.native.registration-manifest/99".to_string()),
        ..runtime_behavior()
    });

    assert_eq!(report.health, NativePluginBehaviorHealth::Invalid);
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("registration_manifest_schema is unsupported")));
}

#[test]
fn editor_contribution_schema_is_accepted_only_for_editor_entries() {
    let behavior = NativePluginBehavior {
        registration_manifest_schema: Some(
            SERIALIZED_EDITOR_CONTRIBUTION_BATCH_SCHEMA_V1.to_string(),
        ),
        registration_manifest: Some(r#"{"package_id":"fixture","contributions":[]}"#.to_string()),
        ..runtime_behavior()
    };

    let editor = NativePluginBehaviorValidationReport::from_behavior(
        "fixture",
        PluginModuleKind::Editor,
        ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        Some(&behavior),
    );
    let runtime = validate(behavior);

    assert_eq!(editor.health, NativePluginBehaviorHealth::Clean);
    assert!(editor.diagnostics.is_empty());
    assert_eq!(runtime.health, NativePluginBehaviorHealth::Invalid);
    assert!(runtime
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("registration_manifest_schema is unsupported")));
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
        command_manifest_schema: Some(ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V4.to_string()),
        event_manifest_schema: Some(ZIRCON_NATIVE_EVENT_MANIFEST_SCHEMA_V3.to_string()),
        registration_manifest_schema: Some(
            ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3.to_string(),
        ),
        command_manifest: Some(command_manifest().to_string()),
        event_manifest: Some("event=echoed;payload=bytes".to_string()),
        registration_manifest: Some(
            "schema = \"zircon.native.registration-manifest/3\"".to_string(),
        ),
        command_table: Some(command_table()),
        invoke_command: Some(noop_invoke_command),
        save_state: Some(noop_save_state),
        restore_state: Some(noop_restore_state),
        unload: Some(noop_unload),
    }
}

unsafe extern "C" fn noop_invoke_command(
    _command_slot: u32,
    _payload: NativePluginByteSliceV2,
    _output: NativePluginOutputSinkV4,
) -> NativePluginCallbackStatusV2 {
    status_ok()
}

unsafe extern "C" fn noop_save_state(
    _output: *mut super::super::abi_declarations::NativePluginOwnedByteBufferV2,
) -> NativePluginCallbackStatusV2 {
    status_ok()
}

unsafe extern "C" fn noop_restore_state(
    _state: super::super::abi_declarations::NativePluginByteSliceV2,
) -> NativePluginCallbackStatusV2 {
    status_ok()
}

unsafe extern "C" fn noop_unload() -> NativePluginCallbackStatusV2 {
    status_ok()
}

fn status_ok() -> NativePluginCallbackStatusV2 {
    NativePluginCallbackStatusV2 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: std::ptr::null(),
    }
}

fn command_manifest() -> &'static str {
    r#"
        schema = "zircon.native.command-manifest/4"
        [[commands]]
        name = "echo"
        slot = 0
        payload_schema = "bytes"
        max_output_bytes = 16
    "#
}

fn command_table() -> std::sync::Arc<NativePluginCommandTable> {
    std::sync::Arc::new(NativePluginCommandTable::from_manifest_v4(command_manifest()).unwrap())
}
