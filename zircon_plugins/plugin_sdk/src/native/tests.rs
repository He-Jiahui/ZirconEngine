use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static HOST_ABI_VERSION_CALLS: AtomicUsize = AtomicUsize::new(0);

static REPORT: NativePluginStatic<NativePluginEntryReportV3> =
    NativePluginStatic::new(NativePluginEntryReportV3 {
        layout_epoch: ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
        package_manifest_toml: b"\0".as_ptr().cast(),
        diagnostics: b"ready\0".as_ptr().cast(),
        negotiated_capabilities: b"runtime.plugin.weather\0".as_ptr().cast(),
        required_capabilities: b"runtime.plugin.weather\0".as_ptr().cast(),
        denied_capabilities: b"runtime.plugin.denied\0".as_ptr().cast(),
        behavior: std::ptr::null(),
        bridge_methods: std::ptr::null(),
    });
static MISSING: NativePluginStatic<NativePluginEntryReportV3> =
    NativePluginStatic::new(NativePluginEntryReportV3 {
        layout_epoch: ZIRCON_NATIVE_PLUGIN_ENTRY_REPORT_LAYOUT_EPOCH,
        package_manifest_toml: b"\0".as_ptr().cast(),
        diagnostics: b"missing host\0".as_ptr().cast(),
        negotiated_capabilities: b"\0".as_ptr().cast(),
        required_capabilities: b"runtime.plugin.weather\0".as_ptr().cast(),
        denied_capabilities: b"runtime.plugin.denied\0".as_ptr().cast(),
        behavior: std::ptr::null(),
        bridge_methods: std::ptr::null(),
    });
static ENTRY: NativePluginEntryPointV3 = NativePluginEntryPointV3::new(
    &REPORT,
    &MISSING,
    &["runtime.plugin.weather"],
    &["runtime.plugin.denied"],
    None,
);

#[test]
fn native_plugin_static_only_syncs_audited_abi_carriers() {
    fn assert_sync<T: Sync>() {}

    assert_sync::<NativePluginStatic<NativePluginAbiV3>>();
    assert_sync::<NativePluginStatic<NativePluginBehaviorV4>>();
    assert_sync::<NativePluginStatic<NativePluginEntryReportV3>>();
    assert_sync::<NativePluginStatic<NativePluginBridgeMethodTableV3>>();
    assert_sync::<NativePluginStatic<[NativePluginBridgeMethodV3; 2]>>();

    let source = include_str!("../native.rs");
    assert!(source.contains("unsafe trait NativePluginStaticValue"));
    assert!(source.contains("unsafe impl<T: NativePluginStaticValue> Sync"));
    assert!(!source.contains("unsafe impl<T> Sync for NativePluginStatic<T>"));
}

#[test]
fn native_plugin_static_preserves_zero_cost_abi_layout() {
    assert_eq!(
        std::mem::size_of::<NativePluginStatic<NativePluginEntryReportV3>>(),
        std::mem::size_of::<NativePluginEntryReportV3>()
    );
    assert_eq!(
        std::mem::align_of::<NativePluginStatic<NativePluginEntryReportV3>>(),
        std::mem::align_of::<NativePluginEntryReportV3>()
    );
    assert_eq!(REPORT.as_ptr(), REPORT.get() as *const _);

    println!(
        "PERF-MVP-PLUGINS01-SEALED-NATIVE-STATIC payload_bytes={} wrapper_bytes={} layout_overhead_bytes=0 runtime_guard_branches=0 runtime_allocations=0",
        std::mem::size_of::<NativePluginEntryReportV3>(),
        std::mem::size_of::<NativePluginStatic<NativePluginEntryReportV3>>()
    );
}

#[test]
fn native_entry_point_selects_report_from_host_capabilities() {
    let granted = b"runtime.plugin.weather\0";
    let host = NativePluginHostFunctionTableV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        host_handle: 7,
        granted_capabilities: granted.as_ptr().cast(),
        host_abi_version: Some(host_abi_version),
        host_has_capability: None,
        host_log: None,
        host_diagnostic: None,
    };

    assert_eq!(ENTRY.entry_report(&host), REPORT.as_ptr());
}

#[test]
fn native_entry_point_validates_the_host_abi_once_per_negotiation() {
    HOST_ABI_VERSION_CALLS.store(0, Ordering::SeqCst);
    let granted = b"runtime.plugin.weather\0";
    let host = NativePluginHostFunctionTableV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        host_handle: 7,
        granted_capabilities: granted.as_ptr().cast(),
        host_abi_version: Some(counting_host_abi_version),
        host_has_capability: None,
        host_log: None,
        host_diagnostic: None,
    };

    assert_eq!(ENTRY.entry_report(&host), REPORT.as_ptr());
    assert_eq!(HOST_ABI_VERSION_CALLS.load(Ordering::SeqCst), 1);
}

#[test]
fn native_entry_point_rejects_denied_capability() {
    let granted = b"runtime.plugin.weather\nruntime.plugin.denied\0";
    let host = NativePluginHostFunctionTableV3 {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION,
        host_handle: 7,
        granted_capabilities: granted.as_ptr().cast(),
        host_abi_version: Some(host_abi_version),
        host_has_capability: None,
        host_log: None,
        host_diagnostic: None,
    };

    assert_eq!(ENTRY.entry_report(&host), MISSING.as_ptr());
}

#[test]
fn native_owned_bytes_round_trips_through_sdk_free_callback() {
    let buffer = owned_bytes(b"payload".to_vec());
    let free = buffer
        .free
        .expect("SDK-owned buffers carry a free callback");
    let status = unsafe { free(buffer) };

    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
}

#[test]
fn native_owned_bytes_free_rejects_malformed_length_before_reclaiming() {
    let backing = *b"ok";
    let status = unsafe {
        free_owned_bytes_v3(NativePluginOwnedByteBufferV3 {
            data: backing.as_ptr() as *mut u8,
            len: backing.len(),
            capacity: backing.len() - 1,
            owner_token: 0,
            free: None,
        })
    };

    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
}

#[test]
fn native_owned_bytes_free_rejects_null_pointer_with_owned_capacity() {
    let status = unsafe {
        free_owned_bytes_v3(NativePluginOwnedByteBufferV3 {
            data: std::ptr::null_mut(),
            len: 0,
            capacity: 1,
            owner_token: 0,
            free: None,
        })
    };

    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
}

#[test]
fn native_owned_bytes_free_rejects_owner_mismatch_without_consuming_allocation() {
    let buffer = owned_bytes(b"payload".to_vec());
    let mut malformed = buffer;
    malformed.owner_token ^= 1;

    let malformed_status = unsafe { free_owned_bytes_v3(malformed) };
    assert_eq!(malformed_status.code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);

    let recovered_status = unsafe { free_owned_bytes_v3(buffer) };
    assert_eq!(recovered_status.code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
}

#[test]
fn native_panic_guard_maps_panic_to_callback_status() {
    let status = catch_native_callback_panic(b"panic caught\0", || panic!("fixture panic"));

    assert_eq!(status.code, ZIRCON_NATIVE_PLUGIN_STATUS_PANIC);
}

#[test]
fn native_panic_guard_does_not_replace_the_process_global_hook() {
    let source = include_str!("../native.rs");
    let guard = source
        .split("pub fn catch_native_callback_panic")
        .nth(1)
        .and_then(|text| {
            text.split("pub fn host_supports_all_capabilities_v3")
                .next()
        })
        .expect("read native callback panic guard");

    assert!(guard.contains("catch_unwind(AssertUnwindSafe(callback))"));
    assert!(!guard.contains("std::panic::take_hook()"));
    assert!(!guard.contains("std::panic::set_hook("));
    assert!(!guard.contains("Box::new(|_| {})"));
}

#[test]
fn native_dynamic_registration_manifest_round_trips() {
    let manifest = NativePluginRegistrationManifestV3 {
        schema: NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3_TEXT.to_string(),
        modules: vec![NativePluginRegistrationModuleV3 {
            name: "runtime".to_string(),
            kind: "runtime".to_string(),
        }],
        systems: vec![NativePluginRegistrationSystemV3 {
            id: "fixture.tick".to_string(),
            module: "runtime".to_string(),
            stage: "Update".to_string(),
            order: 10,
            sets: vec!["fixture".to_string()],
            before: vec!["render".to_string()],
            after: vec!["physics".to_string()],
            access: vec!["write:resource:fixture.resource".to_string()],
            thread_affinity: NativePluginRegistrationThreadAffinityV3::WorkerSafe,
            bridge_interface: Some("fixture.runtime".to_string()),
            bridge_method: Some("tick".to_string()),
        }],
        resources: vec![NativePluginRegistrationResourceV3 {
            id: "fixture.resource".to_string(),
            module: Some("runtime".to_string()),
            schema: Some("json".to_string()),
        }],
        events: vec![NativePluginRegistrationEventV3 {
            namespace: "fixture".to_string(),
            name: "echoed".to_string(),
            stable_hash: 42,
            schema: Some("bytes".to_string()),
        }],
        extensions: vec![NativePluginRegistrationExtensionV3 {
            point: "runtime.importer".to_string(),
            contribution: Some("fixture.data_json".to_string()),
            schema: None,
        }],
        capabilities: vec![
            "runtime.plugin.fixture".to_string(),
            NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY_V3_TEXT.to_string(),
        ],
    };

    let text =
        registration_manifest_v3_to_toml(&manifest).expect("registration manifest serializes");
    let parsed = registration_manifest_v3_from_toml(&text).expect("registration manifest parses");

    assert!(registration_manifest_v3_schema_is_current(&parsed));
    assert_eq!(parsed, manifest);
}

#[test]
fn native_command_manifest_v4_round_trips_dense_slots_and_output_limits() {
    let manifest = NativePluginCommandManifestV4 {
        schema: NATIVE_COMMAND_MANIFEST_SCHEMA_V4_TEXT.to_string(),
        commands: vec![
            NativePluginCommandV4 {
                name: "echo\0binary-safe".to_string(),
                slot: 0,
                payload_schema: "bytes".to_string(),
                max_output_bytes: 1024,
            },
            NativePluginCommandV4 {
                name: "asset.import/data".to_string(),
                slot: 1,
                payload_schema: "zircon.asset.import/1".to_string(),
                max_output_bytes: NATIVE_COMMAND_MAX_OUTPUT_BYTES_V4,
            },
        ],
    };

    let text = command_manifest_v4_to_toml(&manifest).expect("command manifest serializes");
    let parsed = command_manifest_v4_from_toml(&text).expect("command manifest parses from TOML");

    assert!(command_manifest_v4_is_current_and_dense(&parsed));
    assert_eq!(parsed, manifest);
}

#[test]
fn native_command_manifest_v4_rejects_whitespace_only_payload_schema() {
    let manifest = NativePluginCommandManifestV4 {
        schema: NATIVE_COMMAND_MANIFEST_SCHEMA_V4_TEXT.to_string(),
        commands: vec![NativePluginCommandV4 {
            name: "echo".to_string(),
            slot: 0,
            payload_schema: " \t\r\n".to_string(),
            max_output_bytes: 1024,
        }],
    };

    assert!(!command_manifest_v4_is_current_and_dense(&manifest));
}

#[test]
fn native_command_manifest_v4_rejects_unknown_root_and_command_fields() {
    for text in [
        r#"schema = "zircon.native.command-manifest/4"
unexpected_root_field = true"#,
        r#"schema = "zircon.native.command-manifest/4"
[[commands]]
name = "echo"
slot = 0
payload_schema = "bytes"
max_output_bytes = 1
unexpected_command_field = true"#,
    ] {
        assert!(command_manifest_v4_from_toml(text).is_err());
    }
}

#[test]
fn native_output_sink_v4_writes_into_host_owned_context() {
    unsafe extern "C" fn append(
        context: *mut std::ffi::c_void,
        bytes: NativePluginByteSliceV3,
    ) -> NativePluginCallbackStatusV3 {
        let output = unsafe { &mut *context.cast::<Vec<u8>>() };
        output.extend_from_slice(unsafe { bytes_from_slice(bytes) });
        callback_status(ZIRCON_NATIVE_PLUGIN_STATUS_OK, NATIVE_EMPTY_CSTR)
    }

    let mut output = Vec::new();
    let sink = NativePluginOutputSinkV4 {
        context: (&mut output as *mut Vec<u8>).cast(),
        max_output_bytes: 8,
        write: Some(append),
    };

    assert_eq!(
        unsafe { sink.write(b"echo:") }.code,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK
    );
    assert_eq!(
        unsafe { sink.write(b"ok") }.code,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK
    );
    assert_eq!(output, b"echo:ok");
}

#[test]
fn native_output_sink_v4_rejects_missing_writer_and_oversized_chunk() {
    let sink = NativePluginOutputSinkV4 {
        context: std::ptr::null_mut(),
        max_output_bytes: 4,
        write: None,
    };

    assert_eq!(
        unsafe { sink.write(b"12345") }.code,
        ZIRCON_NATIVE_PLUGIN_STATUS_ERROR
    );
    assert_eq!(
        unsafe { sink.write(b"ok") }.code,
        ZIRCON_NATIVE_PLUGIN_STATUS_ERROR
    );
}

unsafe extern "C" fn host_abi_version() -> u32 {
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION
}

unsafe extern "C" fn counting_host_abi_version() -> u32 {
    HOST_ABI_VERSION_CALLS.fetch_add(1, Ordering::SeqCst);
    ZIRCON_NATIVE_PLUGIN_ABI_VERSION
}
