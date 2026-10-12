use std::sync::atomic::{AtomicUsize, Ordering};

use super::super::abi_declarations::NativePluginSchemaVersionsV3;
use super::*;

fn minimal_behavior(abi_version: u32) -> NativePluginBehaviorV4 {
    NativePluginBehaviorV4 {
        abi_version,
        is_stateless: 1,
        schema_versions: NativePluginSchemaVersionsV3 {
            state_schema_version: 0,
            command_manifest_schema: std::ptr::null(),
            event_manifest_schema: std::ptr::null(),
            registration_manifest_schema: std::ptr::null(),
        },
        command_manifest: std::ptr::null(),
        event_manifest: std::ptr::null(),
        registration_manifest: std::ptr::null(),
        invoke_command: None,
        save_state: None,
        restore_state: None,
        unload: None,
    }
}

#[test]
fn native_behavior_reports_unsupported_abi_version_with_typed_error() {
    let behavior = minimal_behavior(ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4 + 1);
    let error = unsafe { NativePluginBehavior::from_abi_v4(&behavior) }
        .expect_err("unsupported behavior ABI should report typed error");

    assert!(matches!(
        error,
        NativePluginBehaviorError::UnsupportedAbiVersion { actual, expected }
            if actual == ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4 + 1
                && expected == ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4
    ));
}

#[test]
// 该用例锁定名称只在宿主解析、跨 ABI 只传 slot 的约定；内嵌 NUL 用于区分旧 C 字符串协议。
fn native_behavior_v4_resolves_dense_slot_without_c_string_or_plugin_owned_buffer() {
    unsafe extern "C" fn write_echo(
        slot: u32,
        _payload: NativePluginByteSliceV3,
        output: NativePluginOutputSinkV4,
    ) -> NativePluginCallbackStatusV3 {
        assert_eq!(slot, 0);
        let bytes = b"host-owned";
        unsafe {
            output.write.expect("host writer")(
                output.context,
                NativePluginByteSliceV3 {
                    data: bytes.as_ptr(),
                    len: bytes.len(),
                },
            )
        }
    }

    let command_manifest = r#"
            schema = "zircon.native.command-manifest/4"
            [[commands]]
            name = "nul\u0000safe"
            slot = 0
            payload_schema = "bytes"
            max_output_bytes = 32
        "#;
    let behavior = NativePluginBehavior {
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: Some(ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V4.to_string()),
        event_manifest_schema: None,
        registration_manifest_schema: None,
        command_manifest: Some(command_manifest.to_string()),
        event_manifest: None,
        registration_manifest: None,
        command_table: Some(Arc::new(
            NativePluginCommandTable::from_manifest_v4(command_manifest).unwrap(),
        )),
        invoke_command: Some(write_echo),
        save_state: None,
        restore_state: None,
        unload: None,
    };

    let callbacks = behavior.callback_snapshot();
    assert!(callbacks.has_invoke_command());
    assert!(callbacks.declares_command("nul\0safe"));
    assert!(!callbacks.declares_command("undeclared"));

    let report = callbacks.invoke_command("nul\0safe", b"ignored");
    assert_eq!(report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert_eq!(report.payload.as_deref(), Some(&b"host-owned"[..]));
}

#[test]
fn native_behavior_v4_rejects_callback_that_ignores_host_sink_failure() {
    unsafe extern "C" fn ignore_sink_failure(
        _slot: u32,
        _payload: NativePluginByteSliceV3,
        output: NativePluginOutputSinkV4,
    ) -> NativePluginCallbackStatusV3 {
        let bytes = b"exceeds-limit";
        let _ = unsafe {
            output.write.expect("host writer")(
                output.context,
                NativePluginByteSliceV3 {
                    data: bytes.as_ptr(),
                    len: bytes.len(),
                },
            )
        };
        NativePluginCallbackStatusV3 {
            code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
            diagnostics: std::ptr::null(),
        }
    }

    let command_manifest = r#"
            schema = "zircon.native.command-manifest/4"
            [[commands]]
            name = "bounded"
            slot = 0
            payload_schema = "bytes"
            max_output_bytes = 4
        "#;
    let behavior = NativePluginBehavior {
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: Some(ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V4.to_string()),
        event_manifest_schema: None,
        registration_manifest_schema: None,
        command_manifest: Some(command_manifest.to_string()),
        event_manifest: None,
        registration_manifest: None,
        command_table: Some(Arc::new(
            NativePluginCommandTable::from_manifest_v4(command_manifest).unwrap(),
        )),
        invoke_command: Some(ignore_sink_failure),
        save_state: None,
        restore_state: None,
        unload: None,
    };

    let report = behavior.callback_snapshot().invoke_command("bounded", b"");

    assert_eq!(report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
    assert!(report.payload.is_none());
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("exceeded its declared 4 byte limit")));
}

#[test]
fn native_behavior_v4_rejects_non_dense_duplicate_and_oversized_command_metadata() {
    for manifest in [
        r#"schema = "zircon.native.command-manifest/4"
[[commands]]
name = "first"
slot = 1
payload_schema = "bytes"
max_output_bytes = 1"#,
        r#"schema = "zircon.native.command-manifest/4"
[[commands]]
name = "first"
slot = 0
payload_schema = "bytes"
max_output_bytes = 1
[[commands]]
name = "first"
slot = 1
payload_schema = "bytes"
max_output_bytes = 1"#,
        r#"schema = "zircon.native.command-manifest/4"
[[commands]]
name = "first"
slot = 0
payload_schema = "bytes"
max_output_bytes = 268435457"#,
    ] {
        assert!(NativePluginCommandTable::from_manifest_v4(manifest).is_err());
    }
}

#[test]
fn native_behavior_v4_rejects_unknown_command_manifest_fields() {
    for manifest in [
        r#"schema = "zircon.native.command-manifest/4"
unexpected_root_field = true"#,
        r#"schema = "zircon.native.command-manifest/4"
[[commands]]
name = "first"
slot = 0
payload_schema = "bytes"
max_output_bytes = 1
unexpected_command_field = true"#,
    ] {
        assert!(NativePluginCommandTable::from_manifest_v4(manifest).is_err());
    }
}

#[test]
fn native_behavior_rejects_malformed_owned_buffer_before_copying_or_freeing() {
    let backing = *b"ok";
    let buffer = NativePluginOwnedByteBufferV3 {
        data: backing.as_ptr() as *mut u8,
        len: backing.len(),
        capacity: backing.len() - 1,
        owner_token: 0,
        free: None,
    };
    let mut diagnostics = Vec::new();

    let payload = take_owned_bytes(buffer, &mut diagnostics);

    assert!(payload.is_none());
    assert_eq!(
        diagnostics,
        vec!["native plugin owned buffer was malformed: len 2 exceeds capacity 1"]
    );
}

#[test]
// 故意提供不可读的地址配合不可分配长度；用例依赖容量拒绝先于外来内存读取，不能调整这一次序。
fn native_behavior_host_output_rejects_unallocatable_chunk_before_reading_it() {
    let mut output = NativePluginHostOutput::new(usize::MAX);

    let status = unsafe {
        write_host_output_v4(
            (&mut output as *mut NativePluginHostOutput).cast(),
            NativePluginByteSliceV3 {
                data: std::ptr::NonNull::<u8>::dangling().as_ptr(),
                len: usize::MAX,
            },
        )
    };

    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
    assert!(output.bytes.is_empty());
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("could not reserve")));
}

#[test]
fn native_behavior_typed_error_preserves_unsupported_abi_message() {
    let error = NativePluginBehaviorError::UnsupportedAbiVersion {
        actual: ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4 + 2,
        expected: ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4,
    };

    assert_eq!(
        error.to_string(),
        format!(
            "unsupported native plugin behavior ABI version {}; expected {}",
            ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4 + 2,
            ZIRCON_NATIVE_PLUGIN_BEHAVIOR_ABI_VERSION_V4
        )
    );
}
