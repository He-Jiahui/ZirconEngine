use crate::plugin::PluginModuleKind;

use super::super::abi_declarations::{
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3, ZIRCON_NATIVE_PLUGIN_STATUS_OK,
};
use super::super::behavior_calls::NativePluginBehavior;
use super::schema::{
    ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V3, ZIRCON_NATIVE_EVENT_MANIFEST_SCHEMA_V3,
};
use super::{NativePluginBehaviorHealth, NativePluginBehaviorValidationReport};

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

unsafe extern "C" fn noop_unload() -> super::super::abi_declarations::NativePluginCallbackStatusV2 {
    status_ok()
}

fn status_ok() -> super::super::abi_declarations::NativePluginCallbackStatusV2 {
    super::super::abi_declarations::NativePluginCallbackStatusV2 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: std::ptr::null(),
    }
}
