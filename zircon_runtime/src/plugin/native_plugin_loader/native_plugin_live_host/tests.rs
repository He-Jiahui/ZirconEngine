use super::*;

use crate::core::framework::bridge::{BridgeError, BridgeOwnerTransitionMode, PluginInterface};
use crate::core::framework::project::{ExportPackagingStrategy, ProjectPluginSelection};
use crate::plugin::native::{
    NativeBridgeCall, NativeBridgeMethodBinding, NativeBridgeMethodFn,
    NativePluginBehaviorValidationReport, NativePluginDescriptor, NativePluginEntryReport,
    NativePluginLoadReport, ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
};
use crate::plugin::{
    PluginDependencyManifest, PluginInterfaceManifest, PluginInterfaceMethodManifest,
    PluginPackageManifest, RuntimeExtensionRegistry, RuntimePluginBridgeLifecycleEvent,
    RuntimePluginBridgeLifecycleState, RuntimePluginCatalog, RuntimePluginRegistrationReport,
};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use zircon_runtime_interface::{
    ZrByteBufferRef, ZrByteSlice, ZrStatus, ZrStatusCode,
    ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1,
};

use super::super::behavior_calls::{NativePluginBehavior, NativePluginCommandTable};

struct NativeLiveHostTestAllocator;

thread_local! {
    static NATIVE_LIVE_HOST_COUNT_ALLOCATIONS: Cell<bool> = const { Cell::new(false) };
    static NATIVE_LIVE_HOST_ALLOCATION_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[global_allocator]
static NATIVE_LIVE_HOST_TEST_ALLOCATOR: NativeLiveHostTestAllocator = NativeLiveHostTestAllocator;

unsafe impl GlobalAlloc for NativeLiveHostTestAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_native_live_host_test_allocation();
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_native_live_host_test_allocation();
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record_native_live_host_test_allocation();
        unsafe { System.realloc(pointer, layout, new_size) }
    }
}

fn record_native_live_host_test_allocation() {
    let _ = NATIVE_LIVE_HOST_COUNT_ALLOCATIONS.try_with(|enabled| {
        if enabled.get() {
            let _ = NATIVE_LIVE_HOST_ALLOCATION_COUNT.try_with(|count| {
                count.set(count.get().saturating_add(1));
            });
        }
    });
}

fn count_native_live_host_test_allocations<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    NATIVE_LIVE_HOST_ALLOCATION_COUNT.with(|count| count.set(0));
    NATIVE_LIVE_HOST_COUNT_ALLOCATIONS.with(|enabled| {
        assert!(!enabled.replace(true), "allocation counting must not nest");
    });
    let value = operation();
    NATIVE_LIVE_HOST_COUNT_ALLOCATIONS.with(|enabled| enabled.set(false));
    let allocations = NATIVE_LIVE_HOST_ALLOCATION_COUNT.with(Cell::get);
    (value, allocations)
}

#[path = "tests/bridge_bindings.rs"]
mod bridge_bindings;
#[path = "tests/callback_lease.rs"]
mod callback_lease;
#[path = "tests/hot_reload_failures.rs"]
mod hot_reload_failures;
#[path = "tests/hot_reload_publication.rs"]
mod hot_reload_publication;
#[path = "tests/hot_reload_state.rs"]
mod hot_reload_state;
#[path = "tests/hot_update_application.rs"]
mod hot_update_application;
#[path = "tests/registration_replay.rs"]
mod registration_replay;
#[path = "tests/runtime_behavior.rs"]
mod runtime_behavior;

#[test]
fn native_live_host_reports_missing_editor_package_on_hot_reload() {
    let project_root = std::env::temp_dir().join(format!(
        "zircon-runtime-missing-native-live-host-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let error = NativePluginLiveHost::default()
        .hot_reload_editor_plugin(&project_root, "physics")
        .unwrap_err();
    assert!(error.contains("plugin physics hot reload did not load an editor native package"));
    assert!(error.contains("native plugin root does not exist"));
}

#[test]
fn native_live_host_reports_unloaded_plugin_by_module_kind() {
    let error = NativePluginLiveHost::default()
        .unload_runtime_plugin("physics")
        .unwrap_err();
    assert_eq!(
        error,
        "plugin physics is not loaded in the runtime live host; run Hot Reload after building its native dynamic package"
    );
}

#[test]
fn native_live_host_lifecycle_unload_reports_typed_unloaded_error() {
    let error = NativePluginLiveHost::default()
        .unload_plugin_result("physics", PluginModuleKind::Runtime)
        .expect_err("unloaded runtime plugin should produce typed lifecycle error");

    assert!(matches!(
        &error,
        NativePluginLiveHostLifecycleError::RuntimePluginNotLoaded {
            plugin_id,
            module_kind: PluginModuleKind::Runtime,
        } if plugin_id == "physics"
    ));
    assert_eq!(
        error.to_string(),
        "plugin physics is not loaded in the runtime live host; run Hot Reload after building its native dynamic package"
    );
}

#[test]
fn native_live_host_lifecycle_rejects_unmanaged_module_kind_with_typed_error() {
    let error = load_for_module_kind(
        &NativePluginLoader::default(),
        std::path::Path::new("plugins"),
        PluginModuleKind::Vm,
    )
    .expect_err("VM module handles should not be loaded by the native live host");

    assert!(matches!(
        &error,
        NativePluginLiveHostLifecycleError::UnsupportedLiveHostModuleKind {
            module_kind: PluginModuleKind::Vm,
        }
    ));
    assert_eq!(
        error.to_string(),
        "native plugin live host does not manage vm module handles"
    );
}

#[test]
fn native_live_host_runtime_behavior_calls_report_unloaded_plugin() {
    let host = NativePluginLiveHost::default();
    let expected = "plugin physics is not loaded in the runtime live host; run Hot Reload after building its native dynamic package";
    assert_eq!(
        host.runtime_behavior_descriptor("physics").unwrap_err(),
        expected
    );
    assert!(host
        .runtime_behavior_descriptors()
        .expect("empty runtime live host should list no descriptors")
        .is_empty());
    assert_eq!(
        host.invoke_runtime_plugin_command("physics", "simulate", b"")
            .unwrap_err(),
        expected
    );
    assert_eq!(
        host.save_runtime_plugin_state("physics").unwrap_err(),
        expected
    );
    assert_eq!(
        host.restore_runtime_plugin_state("physics", b"")
            .unwrap_err(),
        expected
    );
}

#[test]
fn native_live_host_runtime_command_interior_nul_is_resolved_without_a_c_string() {
    INTERIOR_NUL_INVOKE_COUNT.store(0, Ordering::SeqCst);
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_behavior(
                "physics",
                NativePluginBehavior {
                    is_stateless: true,
                    state_schema_version: 0,
                    command_manifest_schema: Some("zircon.native.command-manifest/4".to_string()),
                    event_manifest_schema: None,
                    registration_manifest_schema: None,
                    command_manifest: Some(
                        r#"
schema = "zircon.native.command-manifest/4"
[[commands]]
name = "bad\u0000name"
slot = 0
payload_schema = "bytes"
max_output_bytes = 0
"#
                        .to_string(),
                    ),
                    event_manifest: None,
                    registration_manifest: None,
                    command_table: Some(Arc::new(
                        NativePluginCommandTable::from_manifest_v4(
                            r#"
schema = "zircon.native.command-manifest/4"
[[commands]]
name = "bad\u0000name"
slot = 0
payload_schema = "bytes"
max_output_bytes = 0
"#,
                        )
                        .unwrap(),
                    )),
                    invoke_command: Some(interior_nul_probe_invoke_command),
                    save_state: None,
                    restore_state: None,
                    unload: None,
                },
            ),
        );
    }

    let report = host
        .invoke_runtime_plugin_command("physics", "bad\0name", b"")
        .expect("loaded plugin should return behavior reports for bad command names");

    assert_eq!(report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert!(report.diagnostics.is_empty());
    assert!(report.payload.is_none());
    assert_eq!(INTERIOR_NUL_INVOKE_COUNT.load(Ordering::SeqCst), 1);
}

#[test]
fn native_installed_bridge_scope_owns_library_without_counting_as_active_callback() {
    let host = NativePluginLiveHost::default();
    let library_owner = {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        let plugin = native_live_host_test_plugin_with_bridge_manifest("physics");
        let library_owner = Arc::downgrade(&plugin.library);
        loaded.insert(live_key(PluginModuleKind::Runtime, "physics"), plugin);
        library_owner
    };
    host.install_runtime_bridge_method_bindings(
        "physics",
        [NativeBridgeMethodBinding::new(
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            "sample_count",
            NativeBridgeMethodFn::from_rust(native_live_host_bridge_method),
        )],
    )
    .expect("declared bridge method should install");
    let lifecycle = native_live_host_bridge_lifecycle_state(false);
    let scope = host
        .runtime_bridge_call_scope_from_installed_bindings("physics", &lifecycle)
        .expect("installed bridge method should build a pinned scope");
    let pinned_diagnostics = host
        .plugin_callback_diagnostics("physics", PluginModuleKind::Runtime)
        .expect("installed owner-pin diagnostics should remain queryable");
    assert_eq!(pinned_diagnostics.active_callbacks, 0);
    assert_eq!(pinned_diagnostics.completed_callbacks, 0);

    host.unload_runtime_plugin("physics")
        .expect("a passive bridge generation owner must not reject unload");
    assert!(
        library_owner.upgrade().is_some(),
        "the retained bridge scope must keep its old dynamic library generation loaded"
    );

    let api = scope.api();
    let status = unsafe {
        (api.bridge.call.unwrap())(
            scope.handle(),
            0,
            7,
            std::ptr::null(),
            0,
            ZrByteBufferRef::empty(),
        )
    };
    assert_eq!(
        status.status_code(),
        ZrStatusCode::BridgeNotEnabled,
        "a retained scope from the unloaded generation must reject new callback admission"
    );

    drop(scope);
    assert!(
        library_owner.upgrade().is_none(),
        "dropping the last old-generation scope must release the dynamic library"
    );
}

static BLOCKING_BRIDGE_CALLBACK_ENTERED: AtomicBool = AtomicBool::new(false);
static BLOCKING_BRIDGE_CALLBACK_RELEASED: AtomicBool = AtomicBool::new(false);

#[test]
fn native_bridge_call_blocks_unload_only_while_callback_is_executing() {
    BLOCKING_BRIDGE_CALLBACK_ENTERED.store(false, Ordering::Release);
    BLOCKING_BRIDGE_CALLBACK_RELEASED.store(false, Ordering::Release);

    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_bridge_manifest("physics"),
        );
    }
    host.install_runtime_bridge_method_bindings(
        "physics",
        [NativeBridgeMethodBinding::new(
            <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            "sample_count",
            NativeBridgeMethodFn::from_rust(blocking_native_live_host_bridge_method),
        )],
    )
    .expect("blocking bridge method should install");
    let lifecycle = native_live_host_bridge_lifecycle_state(false);
    let scope = host
        .runtime_bridge_call_scope_from_installed_bindings("physics", &lifecycle)
        .expect("installed bridge method should build a call scope");
    assert_eq!(
        host.plugin_callback_diagnostics("physics", PluginModuleKind::Runtime)
            .expect("loaded plugin diagnostics")
            .active_callbacks,
        0,
        "a retained scope is passive before callback dispatch"
    );

    let call_scope = scope.clone();
    let callback = std::thread::spawn(move || {
        let api = call_scope.api();
        let status = unsafe {
            (api.bridge.call.unwrap())(
                call_scope.handle(),
                0,
                7,
                std::ptr::null(),
                0,
                ZrByteBufferRef::empty(),
            )
        };
        status.status_code()
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !BLOCKING_BRIDGE_CALLBACK_ENTERED.load(Ordering::Acquire) {
        assert!(
            std::time::Instant::now() < deadline,
            "bridge callback did not begin within the test deadline"
        );
        std::thread::yield_now();
    }

    assert_eq!(
        host.plugin_callback_diagnostics("physics", PluginModuleKind::Runtime)
            .expect("in-flight plugin diagnostics")
            .active_callbacks,
        1
    );
    let busy = host
        .unload_runtime_plugin("physics")
        .expect_err("an executing bridge callback must reject unload");
    assert!(busy.contains("active native callback"));

    BLOCKING_BRIDGE_CALLBACK_RELEASED.store(true, Ordering::Release);
    assert_eq!(
        callback.join().expect("bridge callback thread should join"),
        ZrStatusCode::CapabilityDenied
    );
    assert_eq!(
        host.plugin_callback_diagnostics("physics", PluginModuleKind::Runtime)
            .expect("completed plugin diagnostics")
            .active_callbacks,
        0
    );
    host.unload_runtime_plugin("physics")
        .expect("unload should succeed after the executing callback returns");

    drop(scope);
}

#[test]
fn native_runtime_reports_synthesize_callback_status_diagnostics() {
    let failed_call = NativePluginRuntimeBehaviorCall {
        plugin_id: "physics".to_string(),
        report: NativePluginBehaviorCallReport {
            status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            diagnostics: Vec::new(),
            payload: None,
        },
    };
    let dispatch = NativePluginRuntimeCommandDispatchReport {
        command_name: "simulate".to_string(),
        calls: vec![failed_call.clone()],
        diagnostics: Vec::new(),
    };
    assert!(!dispatch.is_clean());
    assert_eq!(dispatch.failed_call_count(), 1);
    assert_eq!(
        dispatch.combined_diagnostics(),
        vec!["runtime plugin physics simulate returned status 1".to_string()]
    );

    let restore = NativePluginRuntimeStateRestoreReport {
        calls: vec![failed_call],
        skipped_plugin_ids: Vec::new(),
        diagnostics: Vec::new(),
    };
    assert!(!restore.is_clean());
    assert_eq!(restore.failed_call_count(), 1);
    assert_eq!(
        restore.combined_diagnostics(),
        vec!["runtime plugin physics restore-state returned status 1".to_string()]
    );
}

#[test]
fn native_live_host_behavior_diagnostics_report_typed_status_error() {
    let error = diagnostics_from_behavior_report(
        "runtime unload before hot reload",
        NativePluginBehaviorCallReport {
            status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            diagnostics: vec!["native plugin behavior callback unload failed".to_string()],
            payload: None,
        },
    )
    .expect_err("failed behavior status should produce a typed diagnostic error");

    assert!(matches!(
        &error,
        NativePluginBehaviorDiagnosticError::FailedStatus {
            label,
            status_code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            diagnostics,
        } if label == "runtime unload before hot reload"
            && diagnostics == &vec![
                "runtime unload before hot reload: native plugin behavior callback unload failed"
                    .to_string()
            ]
    ));
    assert_eq!(
        error.to_string(),
        "runtime unload before hot reload: native plugin behavior callback unload failed"
    );
}

#[test]
fn native_live_host_loads_runtime_export_diagnostics_without_handles() {
    let export_root = std::env::temp_dir().join(format!(
        "zircon-runtime-missing-native-live-host-export-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let report = NativePluginLiveHost::default()
        .load_runtime_plugins_from_export_root(&export_root)
        .expect("missing manifest should be reported as diagnostics, not a host failure");
    assert_eq!(report.module_kind, PluginModuleKind::Runtime);
    assert!(report.loaded_plugin_ids.is_empty());
    assert!(report.runtime_plugin_registration_reports.is_empty());
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("failed to read native plugin load manifest")));
}

#[test]
fn native_live_host_loading_lock_reports_typed_error() {
    let loaded = Arc::new(ObservedLoadedNativePlugins::default());
    let poison_target = Arc::clone(&loaded);
    let _ = std::thread::spawn(move || {
        let _guard = poison_target
            .entries
            .lock()
            .expect("test should lock live host map");
        panic!("poison native live host loading lock");
    })
    .join();

    let error = lock_loaded_native_plugins(&loaded)
        .expect_err("poisoned live-host lock should report a typed loading error");
    assert!(matches!(
        error,
        NativePluginLiveHostLoadingError::LiveHostLockPoisoned
    ));
    assert_eq!(
        error.to_string(),
        "native plugin live host lock is poisoned"
    );
}

#[test]
fn native_live_host_load_report_applies_runtime_bridge_lifecycle_state() {
    let state = native_live_host_bridge_lifecycle_state(false);
    let bridge = state
        .bridge_table()
        .resolve_weak::<dyn NativeLiveHostBridge>();
    let disabled = state.apply_provider_lifecycle_event(
        RuntimePluginBridgeLifecycleEvent::disable_provider("physics"),
    );
    assert!(disabled.is_applied());
    assert_eq!(
        bridge.call(|provider| provider.sample_count()),
        Err(BridgeError::NotEnabled)
    );
    let mut report = NativePluginLiveHostLoadReport {
        module_kind: PluginModuleKind::Runtime,
        loaded_plugin_ids: vec!["physics".to_string()],
        runtime_plugin_registration_reports: Vec::new(),
        runtime_plugin_feature_registration_reports: Vec::new(),
        bridge_lifecycle_reports: Vec::new(),
        diagnostics: Vec::new(),
    };

    report.apply_runtime_bridge_lifecycle(&state);

    assert_eq!(report.bridge_lifecycle_reports.len(), 1);
    assert_eq!(
        report.bridge_lifecycle_reports[0].event.mode,
        BridgeOwnerTransitionMode::Activate
    );
    assert!(report.bridge_lifecycle_reports[0].is_applied());
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("native.live_host.bridge_lifecycle")));
    assert_eq!(bridge.call(|provider| provider.sample_count()), Ok(7));
}

#[test]
fn native_live_host_unload_runtime_plugin_applies_bridge_lifecycle_state() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin("physics", PluginModuleKind::Runtime),
        );
    }
    let state = native_live_host_bridge_lifecycle_state(false);
    let bridge = state
        .bridge_table()
        .resolve_weak::<dyn NativeLiveHostBridge>();
    assert_eq!(bridge.call(|provider| provider.sample_count()), Ok(7));

    let outcome = host
        .unload_runtime_plugin_with_bridge_lifecycle("physics", &state)
        .expect("optional dependents should allow bridge provider unload");

    let bridge_report = outcome
        .bridge_lifecycle_report
        .expect("unload outcome should retain bridge lifecycle report");
    assert_eq!(
        bridge_report.event.mode,
        BridgeOwnerTransitionMode::Deactivate
    );
    assert!(bridge_report.is_applied());
    assert_eq!(
        bridge.call(|provider| provider.sample_count()),
        Err(BridgeError::NotEnabled)
    );
    assert!(host
        .loaded_plugin_ids(PluginModuleKind::Runtime)
        .expect("loaded ids")
        .is_empty());
}

#[test]
fn native_live_host_unload_runtime_plugin_is_blocked_by_strong_bridge_dependents() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin("physics", PluginModuleKind::Runtime),
        );
    }
    let state = native_live_host_bridge_lifecycle_state(true);
    let bridge = state
        .bridge_table()
        .resolve_weak::<dyn NativeLiveHostBridge>();

    let error = host
        .unload_runtime_plugin_with_bridge_lifecycle("physics", &state)
        .unwrap_err();

    assert!(error.contains("bridge.provider_lifecycle_blocked"));
    assert_eq!(bridge.call(|provider| provider.sample_count()), Ok(7));
    assert_eq!(
        host.loaded_plugin_ids(PluginModuleKind::Runtime)
            .expect("loaded ids should still be readable"),
        vec!["physics".to_string()]
    );
}

#[test]
fn native_live_host_bridge_lifecycle_rejected_unload_reports_typed_error() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin("physics", PluginModuleKind::Runtime),
        );
    }
    let state = native_live_host_bridge_lifecycle_state(true);
    let error = host
        .unload_runtime_plugin_with_bridge_lifecycle_result("physics", &state)
        .expect_err("strong bridge dependents should reject unload before native unload");

    assert!(matches!(
        &error,
        NativePluginBridgeLifecycleError::BridgeLifecycleRejected { diagnostic }
            if diagnostic.contains("bridge.provider_lifecycle_blocked")
    ));
    assert_eq!(
        host.loaded_plugin_ids(PluginModuleKind::Runtime)
            .expect("loaded ids should still be readable"),
        vec!["physics".to_string()]
    );
}

fn native_live_host_bridge_lifecycle_state(
    include_required_dependent: bool,
) -> RuntimePluginBridgeLifecycleState {
    let mut extensions = RuntimeExtensionRegistry::default();
    let owner = extensions
        .intern_plugin_module("physics.runtime")
        .expect("test runtime module owner");
    extensions
        .export_interface::<dyn NativeLiveHostBridge>(owner, Arc::new(NativeLiveHostBridgeProvider))
        .expect("test bridge export");
    let mut registrations = vec![native_bridge_registration_with_extensions(
        native_live_host_bridge_manifest(),
        extensions,
    )];
    if include_required_dependent {
        registrations.push(
            RuntimePluginRegistrationReport::from_native_package_manifest(
                PluginPackageManifest::new("weather", "Weather").with_dependency(
                    PluginDependencyManifest::new("physics", true).with_interface(
                        <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
                    ),
                ),
            ),
        );
    }
    let catalog = RuntimePluginCatalog::from_registration_reports(registrations, []);
    RuntimePluginBridgeLifecycleState::from_catalog(catalog)
}

fn native_live_host_bridge_manifest() -> PluginPackageManifest {
    native_live_host_bridge_manifest_with_method_slot(7)
}

fn native_live_host_bridge_manifest_with_method_slot(method_slot: u32) -> PluginPackageManifest {
    PluginPackageManifest::new("physics", "Physics")
        .with_runtime_crate("physics_runtime")
        .with_provided_interface(
            PluginInterfaceManifest::new(
                <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
            )
            .with_method(PluginInterfaceMethodManifest::new(
                "sample_count",
                method_slot,
            )),
        )
}

fn native_bridge_registration_with_extensions(
    manifest: PluginPackageManifest,
    extensions: RuntimeExtensionRegistry,
) -> RuntimePluginRegistrationReport {
    let project_selection = ProjectPluginSelection {
        id: manifest.id.clone(),
        enabled: true,
        required: false,
        target_modes: Vec::new(),
        packaging: ExportPackagingStrategy::SourceTemplate,
        runtime_crate: None,
        editor_crate: None,
        features: Vec::new(),
    };
    RuntimePluginRegistrationReport {
        package_manifest: manifest,
        project_selection,
        extensions,
        diagnostics: Vec::new(),
    }
}

trait NativeLiveHostBridge: Send + Sync {
    fn sample_count(&self) -> u32;
}

impl PluginInterface for dyn NativeLiveHostBridge {
    const INTERFACE_ID: &'static str = "native.live_host.bridge.v1";
}

#[derive(Debug)]
struct NativeLiveHostBridgeProvider;

impl NativeLiveHostBridge for NativeLiveHostBridgeProvider {
    fn sample_count(&self) -> u32 {
        7
    }
}

fn native_live_host_bridge_method(call: NativeBridgeCall) -> ZrStatus {
    let payload = match unsafe {
        call.payload
            .checked_slice(ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1)
    } {
        Ok(payload) => payload,
        Err(_) => return ZrStatus::new(ZrStatusCode::InvalidArgument, ZrByteSlice::empty()),
    };
    if call.interface_slot == 0 && matches!(call.method_slot, 7 | 9) && payload == b"ping" {
        ZrStatus::new(ZrStatusCode::CapabilityDenied, ZrByteSlice::empty())
    } else {
        ZrStatus::new(ZrStatusCode::InvalidArgument, ZrByteSlice::empty())
    }
}

fn blocking_native_live_host_bridge_method(_call: NativeBridgeCall) -> ZrStatus {
    BLOCKING_BRIDGE_CALLBACK_ENTERED.store(true, Ordering::Release);
    while !BLOCKING_BRIDGE_CALLBACK_RELEASED.load(Ordering::Acquire) {
        std::thread::yield_now();
    }
    ZrStatus::new(ZrStatusCode::CapabilityDenied, ZrByteSlice::empty())
}

fn native_live_host_test_plugin(
    plugin_id: &str,
    module_kind: PluginModuleKind,
) -> LoadedNativePlugin {
    let descriptor = NativePluginDescriptor {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        plugin_id: plugin_id.to_string(),
        package_manifest: None,
        runtime_entry_name: None,
        editor_entry_name: None,
        requested_capabilities: Vec::new(),
    };
    let entry_report = NativePluginEntryReport {
        plugin_id: plugin_id.to_string(),
        module_kind,
        package_manifest: None,
        diagnostics: Vec::new(),
        negotiated_capabilities: Vec::new(),
        missing_required_capabilities: Vec::new(),
        denied_capabilities: Vec::new(),
        bridge_method_bindings: Vec::new(),
        editor_contribution_batch: None,
        behavior: None,
        behavior_validation: NativePluginBehaviorValidationReport::from_behavior(
            plugin_id,
            module_kind,
            ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
            None,
        ),
    };
    let (runtime_entry_report, editor_entry_report) = match module_kind {
        PluginModuleKind::Runtime => (Some(entry_report), None),
        PluginModuleKind::Editor => (None, Some(entry_report)),
        PluginModuleKind::Native | PluginModuleKind::Vm => {
            panic!("native live-host test plugins require a runtime or editor entry")
        }
    };
    LoadedNativePlugin {
        plugin_id: plugin_id.to_string(),
        library_path: std::path::PathBuf::from(format!("{plugin_id}.test.dll")),
        descriptor: Some(descriptor),
        runtime_entry_report,
        editor_entry_report,
        library: LoadedNativePlugin::stable_library(this_process_library()),
    }
}

fn native_live_host_test_plugin_with_bridge_manifest(plugin_id: &str) -> LoadedNativePlugin {
    native_live_host_test_plugin_with_bridge_manifest_slot(plugin_id, 7)
}

fn native_live_host_test_plugin_with_discovered_bridge_table(
    plugin_id: &str,
) -> LoadedNativePlugin {
    let mut plugin = native_live_host_test_plugin_with_bridge_manifest(plugin_id);
    if let Some(report) = plugin.runtime_entry_report.as_mut() {
        report
            .bridge_method_bindings
            .push(NativeBridgeMethodBinding::new(
                <dyn NativeLiveHostBridge as PluginInterface>::INTERFACE_ID,
                "sample_count",
                NativeBridgeMethodFn::from_rust(native_live_host_bridge_method),
            ));
    }
    plugin
}

fn native_live_host_test_plugin_with_bridge_manifest_slot(
    plugin_id: &str,
    method_slot: u32,
) -> LoadedNativePlugin {
    let manifest = native_live_host_bridge_manifest_with_method_slot(method_slot);
    let mut plugin = native_live_host_test_plugin(plugin_id, PluginModuleKind::Runtime);
    if let Some(descriptor) = plugin.descriptor.as_mut() {
        descriptor.package_manifest = Some(manifest.clone());
    }
    if let Some(report) = plugin.runtime_entry_report.as_mut() {
        report.package_manifest = Some(manifest);
    }
    plugin
}

pub(super) fn native_live_host_test_plugin_with_behavior(
    plugin_id: &str,
    behavior: NativePluginBehavior,
) -> LoadedNativePlugin {
    let descriptor = NativePluginDescriptor {
        abi_version: ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        plugin_id: plugin_id.to_string(),
        package_manifest: None,
        runtime_entry_name: None,
        editor_entry_name: None,
        requested_capabilities: Vec::new(),
    };
    let behavior_validation = NativePluginBehaviorValidationReport::from_behavior(
        plugin_id,
        PluginModuleKind::Runtime,
        ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3,
        Some(&behavior),
    );
    LoadedNativePlugin {
        plugin_id: plugin_id.to_string(),
        library_path: std::path::PathBuf::from(format!("{plugin_id}.test.dll")),
        descriptor: Some(descriptor),
        runtime_entry_report: Some(NativePluginEntryReport {
            plugin_id: plugin_id.to_string(),
            module_kind: PluginModuleKind::Runtime,
            package_manifest: None,
            diagnostics: Vec::new(),
            negotiated_capabilities: Vec::new(),
            missing_required_capabilities: Vec::new(),
            denied_capabilities: Vec::new(),
            bridge_method_bindings: Vec::new(),
            editor_contribution_batch: None,
            behavior: Some(behavior),
            behavior_validation,
        }),
        editor_entry_report: None,
        library: LoadedNativePlugin::stable_library(this_process_library()),
    }
}

static INTERIOR_NUL_INVOKE_COUNT: AtomicUsize = AtomicUsize::new(0);

static HOT_RELOAD_STATE_BYTES: &[u8] = b"state:physics";

pub(super) fn restored_payloads() -> &'static Mutex<Vec<Vec<u8>>> {
    static PAYLOADS: OnceLock<Mutex<Vec<Vec<u8>>>> = OnceLock::new();
    PAYLOADS.get_or_init(|| Mutex::new(Vec::new()))
}

pub(super) fn hot_reload_payload_fixture_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(super) unsafe extern "C" fn hot_reload_save_state(
    output: *mut super::super::abi_declarations::NativePluginOwnedByteBufferV2,
) -> super::super::abi_declarations::NativePluginCallbackStatusV2 {
    if !output.is_null() {
        *output = super::super::abi_declarations::NativePluginOwnedByteBufferV2 {
            data: HOT_RELOAD_STATE_BYTES.as_ptr() as *mut u8,
            len: HOT_RELOAD_STATE_BYTES.len(),
            capacity: HOT_RELOAD_STATE_BYTES.len(),
            owner_token: 0,
            free: None,
        };
    }
    super::super::abi_declarations::NativePluginCallbackStatusV2 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: std::ptr::null(),
    }
}

pub(super) unsafe extern "C" fn hot_reload_save_state_failure(
    _output: *mut super::super::abi_declarations::NativePluginOwnedByteBufferV2,
) -> super::super::abi_declarations::NativePluginCallbackStatusV2 {
    super::super::abi_declarations::NativePluginCallbackStatusV2 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
        diagnostics: c"save failed during hot reload".as_ptr(),
    }
}

pub(super) unsafe extern "C" fn hot_reload_restore_state(
    state: super::super::abi_declarations::NativePluginByteSliceV2,
) -> super::super::abi_declarations::NativePluginCallbackStatusV2 {
    let payload = if state.data.is_null() || state.len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(state.data, state.len).to_vec()
    };
    restored_payloads()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push(payload);
    super::super::abi_declarations::NativePluginCallbackStatusV2 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: std::ptr::null(),
    }
}

pub(super) unsafe extern "C" fn hot_reload_restore_state_failure(
    state: super::super::abi_declarations::NativePluginByteSliceV2,
) -> super::super::abi_declarations::NativePluginCallbackStatusV2 {
    let payload = if state.data.is_null() || state.len == 0 {
        Vec::new()
    } else {
        std::slice::from_raw_parts(state.data, state.len).to_vec()
    };
    restored_payloads()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push(payload);
    super::super::abi_declarations::NativePluginCallbackStatusV2 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
        diagnostics: c"restore failed during hot reload".as_ptr(),
    }
}

unsafe extern "C" fn interior_nul_probe_invoke_command(
    _command_slot: u32,
    _payload: super::super::abi_declarations::NativePluginByteSliceV2,
    _output: super::super::abi_declarations::NativePluginOutputSinkV4,
) -> super::super::abi_declarations::NativePluginCallbackStatusV2 {
    INTERIOR_NUL_INVOKE_COUNT.fetch_add(1, Ordering::SeqCst);
    super::super::abi_declarations::NativePluginCallbackStatusV2 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: std::ptr::null(),
    }
}

fn this_process_library() -> libloading::Library {
    #[cfg(unix)]
    {
        libloading::os::unix::Library::this().into()
    }
    #[cfg(windows)]
    {
        libloading::os::windows::Library::this()
            .expect("current process library handle should be available")
            .into()
    }
}
