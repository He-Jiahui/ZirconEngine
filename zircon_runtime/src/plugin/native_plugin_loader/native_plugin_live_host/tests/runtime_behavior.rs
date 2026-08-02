use super::*;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::Duration;

use super::super::super::abi_declarations::{
    NativePluginByteSliceV2, NativePluginCallbackStatusV2, NativePluginOutputSinkV4,
    NativePluginOwnedByteBufferV2,
};
use super::super::super::behavior_calls::NativePluginCommandTable;

struct SlowCallbackProbe {
    entered: Mutex<Option<mpsc::Sender<()>>>,
    released: Mutex<bool>,
    release_signal: Condvar,
}

impl SlowCallbackProbe {
    fn new(entered: mpsc::Sender<()>) -> Self {
        Self {
            entered: Mutex::new(Some(entered)),
            released: Mutex::new(false),
            release_signal: Condvar::new(),
        }
    }

    fn wait_for_release(&self) {
        if let Some(entered) = self
            .entered
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            let _ = entered.send(());
        }
        let released = self
            .released
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _released = self
            .release_signal
            .wait_while(released, |released| !*released)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
    }

    fn release(&self) {
        *self
            .released
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
        self.release_signal.notify_all();
    }
}

fn callback_concurrency_fixture_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn reentrant_host_slot() -> &'static Mutex<Option<Arc<NativePluginLiveHost>>> {
    static HOST: OnceLock<Mutex<Option<Arc<NativePluginLiveHost>>>> = OnceLock::new();
    HOST.get_or_init(|| Mutex::new(None))
}

fn slow_callback_slot() -> &'static Mutex<Option<Arc<SlowCallbackProbe>>> {
    static PROBE: OnceLock<Mutex<Option<Arc<SlowCallbackProbe>>>> = OnceLock::new();
    PROBE.get_or_init(|| Mutex::new(None))
}

fn state_restore_count() -> &'static AtomicUsize {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    &COUNT
}

unsafe extern "C" fn reentrant_descriptor_command(
    _command_slot: u32,
    _payload: NativePluginByteSliceV2,
    _output: NativePluginOutputSinkV4,
) -> NativePluginCallbackStatusV2 {
    let host = reentrant_host_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .expect("reentrant host fixture should be installed")
        .clone();
    let (completed_tx, completed_rx) = mpsc::channel();
    std::thread::spawn(move || {
        let result = host.runtime_behavior_descriptor("physics");
        let _ = completed_tx.send(result);
    });
    let completed = completed_rx
        .recv_timeout(Duration::from_millis(250))
        .is_ok();
    NativePluginCallbackStatusV2 {
        code: if completed {
            ZIRCON_NATIVE_PLUGIN_STATUS_OK
        } else {
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR
        },
        diagnostics: std::ptr::null(),
    }
}

unsafe extern "C" fn slow_runtime_command(
    _command_slot: u32,
    _payload: NativePluginByteSliceV2,
    _output: NativePluginOutputSinkV4,
) -> NativePluginCallbackStatusV2 {
    let probe = slow_callback_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .expect("slow callback fixture should be installed")
        .clone();
    probe.wait_for_release();
    NativePluginCallbackStatusV2 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: std::ptr::null(),
    }
}

unsafe extern "C" fn successful_unload() -> NativePluginCallbackStatusV2 {
    NativePluginCallbackStatusV2 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: std::ptr::null(),
    }
}

unsafe extern "C" fn slow_unload() -> NativePluginCallbackStatusV2 {
    let probe = slow_callback_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
        .expect("slow unload fixture should be installed")
        .clone();
    probe.wait_for_release();
    successful_unload()
}

unsafe extern "C" fn stateful_save_state(
    output: *mut NativePluginOwnedByteBufferV2,
) -> NativePluginCallbackStatusV2 {
    if output.is_null() {
        return NativePluginCallbackStatusV2 {
            code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
            diagnostics: std::ptr::null(),
        };
    }
    static STATE: &[u8] = b"state";
    unsafe {
        *output = NativePluginOwnedByteBufferV2 {
            data: STATE.as_ptr().cast_mut(),
            len: STATE.len(),
            capacity: STATE.len(),
            owner_token: 0,
            free: None,
        };
    }
    NativePluginCallbackStatusV2 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: std::ptr::null(),
    }
}

unsafe extern "C" fn stateful_restore_state(
    state: NativePluginByteSliceV2,
) -> NativePluginCallbackStatusV2 {
    let restored = !state.data.is_null()
        && unsafe { std::slice::from_raw_parts(state.data, state.len) } == b"state";
    if restored {
        state_restore_count().fetch_add(1, Ordering::SeqCst);
    }
    NativePluginCallbackStatusV2 {
        code: if restored {
            ZIRCON_NATIVE_PLUGIN_STATUS_OK
        } else {
            ZIRCON_NATIVE_PLUGIN_STATUS_ERROR
        },
        diagnostics: std::ptr::null(),
    }
}

pub(super) unsafe extern "C" fn successful_runtime_command(
    _command_slot: u32,
    _payload: NativePluginByteSliceV2,
    _output: NativePluginOutputSinkV4,
) -> NativePluginCallbackStatusV2 {
    NativePluginCallbackStatusV2 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_OK,
        diagnostics: std::ptr::null(),
    }
}

pub(super) fn callback_test_behavior(
    invoke_command: super::super::super::abi_declarations::NativePluginInvokeCommandFnV4,
) -> NativePluginBehavior {
    let command_manifest = r#"
        schema = "zircon.native.command-manifest/4"
        [[commands]]
        name = "probe"
        slot = 0
        payload_schema = "bytes"
        max_output_bytes = 0
    "#;
    NativePluginBehavior {
        is_stateless: true,
        state_schema_version: 0,
        command_manifest_schema: Some("zircon.native.command-manifest/4".to_string()),
        event_manifest_schema: None,
        registration_manifest_schema: None,
        command_manifest: Some(command_manifest.to_string()),
        event_manifest: None,
        registration_manifest: None,
        command_table: Some(Arc::new(
            NativePluginCommandTable::from_manifest_v4(command_manifest).unwrap(),
        )),
        invoke_command: Some(invoke_command),
        save_state: None,
        restore_state: None,
        unload: Some(successful_unload),
    }
}

fn stateful_callback_test_behavior(
    invoke_command: super::super::super::abi_declarations::NativePluginInvokeCommandFnV4,
) -> NativePluginBehavior {
    let mut behavior = callback_test_behavior(invoke_command);
    behavior.is_stateless = false;
    behavior.state_schema_version = 7;
    behavior.save_state = Some(stateful_save_state);
    behavior.restore_state = Some(stateful_restore_state);
    behavior
}

#[test]
fn native_callback_can_reenter_live_host_descriptor_without_deadlock() {
    let _fixture = callback_concurrency_fixture_lock();
    let host = Arc::new(NativePluginLiveHost::default());
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_behavior(
                "physics",
                callback_test_behavior(reentrant_descriptor_command),
            ),
        );
    }
    *reentrant_host_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(host.clone());

    let report = host
        .invoke_runtime_plugin_command("physics", "probe", b"")
        .expect("loaded plugin callback should return a report");

    *reentrant_host_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
    assert_eq!(report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
}

#[test]
fn slow_callback_allows_queries_and_rejects_concurrent_unload_as_busy() {
    let _fixture = callback_concurrency_fixture_lock();
    let host = Arc::new(NativePluginLiveHost::default());
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_behavior(
                "physics",
                callback_test_behavior(slow_runtime_command),
            ),
        );
    }
    let (entered_tx, entered_rx) = mpsc::channel();
    let probe = Arc::new(SlowCallbackProbe::new(entered_tx));
    *slow_callback_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(probe.clone());

    let command_host = host.clone();
    let command = std::thread::spawn(move || {
        command_host.invoke_runtime_plugin_command("physics", "probe", b"")
    });
    entered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("slow callback should enter before concurrency probes");
    let active_callback_diagnostics = host
        .plugin_callback_diagnostics("physics", PluginModuleKind::Runtime)
        .expect("active callback diagnostics should remain queryable");
    assert_eq!(active_callback_diagnostics.active_callbacks, 1);
    assert!(!active_callback_diagnostics.lifecycle_transition_active);

    let (descriptor_tx, descriptor_rx) = mpsc::channel();
    let descriptor_host = host.clone();
    let descriptor = std::thread::spawn(move || {
        let _ = descriptor_tx.send(descriptor_host.runtime_behavior_descriptor("physics"));
    });
    let descriptor_while_active = descriptor_rx.recv_timeout(Duration::from_millis(250));

    let (unload_tx, unload_rx) = mpsc::channel();
    let unload_host = host.clone();
    let unload = std::thread::spawn(move || {
        let _ = unload_tx.send(unload_host.unload_runtime_plugin("physics"));
    });
    let unload_while_active = unload_rx.recv_timeout(Duration::from_millis(250));

    let (reload_tx, reload_rx) = mpsc::channel();
    let reload_host = host.clone();
    let reload = std::thread::spawn(move || {
        let mut report = NativePluginLoadReport::default();
        report.push_loaded(native_live_host_test_plugin_with_behavior(
            "physics",
            callback_test_behavior(successful_runtime_command),
        ));
        let result = reload_host.hot_reload_reported_plugin(
            report,
            std::path::Path::new("callback-owner-test"),
            "physics",
            PluginModuleKind::Runtime,
        );
        let _ = reload_tx.send(result);
    });
    let reload_while_active = reload_rx.recv_timeout(Duration::from_millis(250));

    let (load_tx, load_rx) = mpsc::channel();
    let load_host = host.clone();
    let load = std::thread::spawn(move || {
        let mut report = NativePluginLoadReport::default();
        report.push_loaded(native_live_host_test_plugin_with_behavior(
            "physics",
            callback_test_behavior(successful_runtime_command),
        ));
        let result = load_host.load_reported_plugins(report, PluginModuleKind::Runtime);
        let _ = load_tx.send(result);
    });
    let load_while_active = load_rx.recv_timeout(Duration::from_millis(250));

    probe.release();
    let command_report = command
        .join()
        .expect("command worker should not panic")
        .expect("command worker should return a report");
    descriptor
        .join()
        .expect("descriptor worker should not panic");
    unload.join().expect("unload worker should not panic");
    reload.join().expect("hot reload worker should not panic");
    load.join().expect("bulk load worker should not panic");
    *slow_callback_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;

    assert_eq!(command_report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
    assert!(descriptor_while_active
        .expect("descriptor query should finish while callback is active")
        .is_ok());
    let unload_error = unload_while_active
        .expect("unload should reject active callbacks without waiting")
        .expect_err("active callback snapshot should make the plugin busy");
    assert!(unload_error.contains("active native callback"));
    let reload_error = reload_while_active
        .expect("hot reload should reject active callbacks without waiting")
        .expect_err("active callback snapshot should make hot reload busy");
    assert!(reload_error.contains("active native callback"));
    let load_error = load_while_active
        .expect("bulk load should reject active callbacks without waiting")
        .expect_err("active callback snapshot should make bulk load busy");
    assert!(load_error.contains("active native callback"));
    let callback_diagnostics = host
        .plugin_callback_diagnostics("physics", PluginModuleKind::Runtime)
        .expect("busy unload should keep callback diagnostics available");
    assert_eq!(callback_diagnostics.active_callbacks, 0);
    assert!(callback_diagnostics.completed_callbacks >= 1);
    assert!(callback_diagnostics.max_callback_duration_ns > 0);
    assert!(
        callback_diagnostics.total_callback_duration_ns
            >= callback_diagnostics.max_callback_duration_ns
    );
    let live_host_diagnostics = host.live_host_diagnostics();
    assert!(live_host_diagnostics.loaded_lock_acquisitions > 0);
    assert!(
        live_host_diagnostics.total_loaded_lock_wait_ns
            >= live_host_diagnostics.max_loaded_lock_wait_ns
    );
}

#[test]
fn bulk_reload_unload_callback_runs_outside_live_host_lock() {
    let _fixture = callback_concurrency_fixture_lock();
    let host = Arc::new(NativePluginLiveHost::default());
    let mut old_behavior = callback_test_behavior(successful_runtime_command);
    old_behavior.unload = Some(slow_unload);
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin_with_behavior("physics", old_behavior),
        );
    }
    let (entered_tx, entered_rx) = mpsc::channel();
    let probe = Arc::new(SlowCallbackProbe::new(entered_tx));
    *slow_callback_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(probe.clone());

    let load_host = host.clone();
    let load = std::thread::spawn(move || {
        let mut report = NativePluginLoadReport::default();
        report.push_loaded(native_live_host_test_plugin_with_behavior(
            "physics",
            callback_test_behavior(successful_runtime_command),
        ));
        load_host.load_reported_plugins(report, PluginModuleKind::Runtime)
    });
    entered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("bulk reload should enter the old plugin unload callback");

    let (descriptor_tx, descriptor_rx) = mpsc::channel();
    let descriptor_host = host.clone();
    let descriptor = std::thread::spawn(move || {
        let _ = descriptor_tx.send(descriptor_host.runtime_behavior_descriptor("physics"));
    });
    let descriptor_while_unloading = descriptor_rx.recv_timeout(Duration::from_millis(250));
    let callback_error = host
        .invoke_runtime_plugin_command("physics", "probe", b"")
        .expect_err("transitioning plugin should reject new callbacks");
    let transition_diagnostics = host
        .plugin_callback_diagnostics("physics", PluginModuleKind::Runtime)
        .expect("transitioning plugin diagnostics should remain queryable");

    probe.release();
    let load_report = load
        .join()
        .expect("bulk reload worker should not panic")
        .expect("bulk reload should finish after unload is released");
    descriptor
        .join()
        .expect("descriptor worker should not panic");
    *slow_callback_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;

    assert!(descriptor_while_unloading
        .expect("descriptor query should finish while unload callback is active")
        .is_ok());
    assert!(callback_error.contains("lifecycle transition is active"));
    assert!(transition_diagnostics.lifecycle_transition_active);
    assert_eq!(load_report.loaded_plugin_ids, vec!["physics".to_string()]);
    assert!(
        !host
            .plugin_callback_diagnostics("physics", PluginModuleKind::Runtime)
            .expect("replacement diagnostics should be available")
            .lifecycle_transition_active
    );
    assert_eq!(
        host.invoke_runtime_plugin_command("physics", "probe", b"")
            .expect("replacement plugin should accept callbacks")
            .status_code,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK
    );
}

#[test]
fn bulk_reload_reopens_the_old_generation_when_loaded_lock_reacquisition_fails() {
    let _fixture = callback_concurrency_fixture_lock();
    let host = Arc::new(NativePluginLiveHost::default());
    let mut old_behavior = callback_test_behavior(successful_runtime_command);
    old_behavior.unload = Some(slow_unload);
    let old_plugin = native_live_host_test_plugin_with_behavior("physics", old_behavior);
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            old_plugin.clone(),
        );
    }
    let (entered_tx, entered_rx) = mpsc::channel();
    let probe = Arc::new(SlowCallbackProbe::new(entered_tx));
    *slow_callback_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(probe.clone());

    let load_host = Arc::clone(&host);
    let reload = std::thread::spawn(move || {
        let mut report = NativePluginLoadReport::default();
        report.push_loaded(native_live_host_test_plugin_with_behavior(
            "physics",
            callback_test_behavior(successful_runtime_command),
        ));
        load_host.load_reported_plugins(report, PluginModuleKind::Runtime)
    });
    entered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("reload should enter the old plugin unload callback");

    let poison_host = Arc::clone(&host);
    let poison = std::thread::spawn(move || {
        let _guard = poison_host
            .loaded
            .entries
            .lock()
            .expect("test should lock the live host map");
        panic!("poison the replacement loaded-lock reacquisition");
    })
    .join();
    assert!(poison.is_err());

    probe.release();
    let error = reload
        .join()
        .expect("reload worker should not panic")
        .expect_err("a poisoned replacement lock should produce a typed loading error");
    *slow_callback_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;

    assert!(error.contains("native plugin live host lock is poisoned"));
    assert!(
        old_plugin.callback_owner_lease().is_ok(),
        "the failed replacement must reopen callback admission for its retained generation"
    );
}

#[test]
fn unload_reopens_the_retained_generation_when_loaded_lock_reacquisition_fails() {
    let _fixture = callback_concurrency_fixture_lock();
    let host = Arc::new(NativePluginLiveHost::default());
    let mut behavior = callback_test_behavior(successful_runtime_command);
    behavior.unload = Some(slow_unload);
    let plugin = native_live_host_test_plugin_with_behavior("physics", behavior);
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            plugin.clone(),
        );
    }
    let (entered_tx, entered_rx) = mpsc::channel();
    let probe = Arc::new(SlowCallbackProbe::new(entered_tx));
    *slow_callback_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(probe.clone());

    let unload_host = Arc::clone(&host);
    let unload = std::thread::spawn(move || unload_host.unload_runtime_plugin("physics"));
    entered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("unload should enter the old plugin callback");

    let poison_host = Arc::clone(&host);
    let poison = std::thread::spawn(move || {
        let _guard = poison_host
            .loaded
            .entries
            .lock()
            .expect("test should lock the live host map");
        panic!("poison the unload loaded-lock reacquisition");
    })
    .join();
    assert!(poison.is_err());

    probe.release();
    let error = unload
        .join()
        .expect("unload worker should not panic")
        .expect_err("a poisoned unload lock should produce a typed lifecycle error");
    *slow_callback_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;

    assert!(error.contains("native plugin live host lock is poisoned"));
    assert!(
        plugin.callback_owner_lease().is_ok(),
        "the failed unload must reopen callback admission for its retained generation"
    );
}

#[test]
fn hot_reload_reopens_the_retained_generation_when_loaded_lock_reacquisition_fails() {
    let _fixture = callback_concurrency_fixture_lock();
    state_restore_count().store(0, Ordering::SeqCst);
    let host = Arc::new(NativePluginLiveHost::default());
    let mut old_behavior = stateful_callback_test_behavior(successful_runtime_command);
    old_behavior.unload = Some(slow_unload);
    let old_plugin = native_live_host_test_plugin_with_behavior("physics", old_behavior);
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            old_plugin.clone(),
        );
    }
    let (entered_tx, entered_rx) = mpsc::channel();
    let probe = Arc::new(SlowCallbackProbe::new(entered_tx));
    *slow_callback_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(probe.clone());

    let reload_host = Arc::clone(&host);
    let reload = std::thread::spawn(move || {
        reload_host.hot_reload_reported_plugin(
            NativePluginLoadReport::from_loaded(vec![native_live_host_test_plugin_with_behavior(
                "physics",
                stateful_callback_test_behavior(successful_runtime_command),
            )]),
            std::path::Path::new("reload-root"),
            "physics",
            PluginModuleKind::Runtime,
        )
    });
    entered_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("hot reload should enter the old plugin unload callback");

    let poison_host = Arc::clone(&host);
    let poison = std::thread::spawn(move || {
        let _guard = poison_host
            .loaded
            .entries
            .lock()
            .expect("test should lock the live host map");
        panic!("poison the hot-reload loaded-lock reacquisition");
    })
    .join();
    assert!(poison.is_err());

    probe.release();
    let error = reload
        .join()
        .expect("hot-reload worker should not panic")
        .expect_err("a poisoned hot-reload lock should produce a typed lifecycle error");
    *slow_callback_slot()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;

    assert!(error.contains("native plugin live host lock is poisoned"));
    assert!(
        old_plugin.callback_owner_lease().is_ok(),
        "the failed hot reload must reopen callback admission for its retained generation"
    );
    assert_eq!(
        state_restore_count().load(Ordering::SeqCst),
        2,
        "a failed publication must restore the saved runtime state to both replacement and retained generations"
    );
}

#[test]
fn native_runtime_broadcast_snapshot_preserves_sorted_plugin_order() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        for plugin_id in ["physics-c", "physics-a", "physics-b"] {
            loaded.insert(
                live_key(PluginModuleKind::Runtime, plugin_id),
                native_live_host_test_plugin_with_behavior(
                    plugin_id,
                    callback_test_behavior(successful_runtime_command),
                ),
            );
        }
    }

    let report = host
        .dispatch_runtime_plugin_command("probe", b"")
        .expect("runtime broadcast should succeed");

    assert_eq!(
        report
            .calls
            .iter()
            .map(|call| call.plugin_id.as_str())
            .collect::<Vec<_>>(),
        vec!["physics-a", "physics-b", "physics-c"]
    );
}

#[test]
fn aborted_broadcast_snapshot_does_not_record_unexecuted_callbacks() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        for plugin_id in ["plugin-a", "plugin-b"] {
            loaded.insert(
                live_key(PluginModuleKind::Runtime, plugin_id),
                native_live_host_test_plugin_with_behavior(
                    plugin_id,
                    callback_test_behavior(successful_runtime_command),
                ),
            );
        }
        loaded
            .get(&live_key(PluginModuleKind::Runtime, "plugin-b"))
            .expect("second plugin should be loaded")
            .begin_lifecycle_transition()
            .expect("second plugin transition should begin");
    }

    let error = host
        .dispatch_runtime_plugin_command("probe", b"")
        .expect_err("broadcast should reject a transitioning snapshot");

    assert!(error.contains("lifecycle transition is active"));
    let first_diagnostics = host
        .plugin_callback_diagnostics("plugin-a", PluginModuleKind::Runtime)
        .expect("first plugin diagnostics should remain available");
    assert_eq!(first_diagnostics.active_callbacks, 0);
    assert_eq!(first_diagnostics.completed_callbacks, 0);
    lock_loaded_native_plugins(&host.loaded)
        .expect("test should lock the native live host")
        .get(&live_key(PluginModuleKind::Runtime, "plugin-b"))
        .expect("second plugin should remain loaded")
        .cancel_lifecycle_transition();
}

#[test]
#[ignore = "manual 1/8/32 native callback broadcast microbenchmark"]
fn native_runtime_broadcast_1_8_32_plugin_benchmark() {
    for plugin_count in [1_usize, 8, 32] {
        let host = NativePluginLiveHost::default();
        {
            let mut loaded = lock_loaded_native_plugins(&host.loaded)
                .expect("benchmark should lock the native live host");
            for index in (0..plugin_count).rev() {
                let plugin_id = format!("plugin-{index:02}");
                loaded.insert(
                    live_key(PluginModuleKind::Runtime, &plugin_id),
                    native_live_host_test_plugin_with_behavior(
                        &plugin_id,
                        callback_test_behavior(successful_runtime_command),
                    ),
                );
            }
        }
        let started = std::time::Instant::now();
        for _ in 0..100 {
            let report = host
                .dispatch_runtime_plugin_command("benchmark", b"")
                .expect("benchmark broadcast should succeed");
            assert_eq!(report.calls.len(), plugin_count);
            assert!(report
                .calls
                .windows(2)
                .all(|calls| calls[0].plugin_id < calls[1].plugin_id));
        }
        let callback_diagnostics = (0..plugin_count)
            .map(|index| {
                host.plugin_callback_diagnostics(
                    format!("plugin-{index:02}"),
                    PluginModuleKind::Runtime,
                )
                .expect("benchmark plugin callback diagnostics")
            })
            .collect::<Vec<_>>();
        let completed_callbacks = callback_diagnostics
            .iter()
            .map(|diagnostics| diagnostics.completed_callbacks)
            .sum::<u64>();
        let total_callback_duration_ns = callback_diagnostics
            .iter()
            .map(|diagnostics| diagnostics.total_callback_duration_ns)
            .sum::<u64>();
        let max_callback_duration_ns = callback_diagnostics
            .iter()
            .map(|diagnostics| diagnostics.max_callback_duration_ns)
            .max()
            .unwrap_or_default();
        let live_host_diagnostics = host.live_host_diagnostics();
        assert_eq!(completed_callbacks, (plugin_count * 100) as u64);
        assert!(callback_diagnostics
            .iter()
            .all(|diagnostics| diagnostics.active_callbacks == 0));
        eprintln!(
            "native callback broadcast: plugins={plugin_count} iterations=100 elapsed_ns={} \
             completed_callbacks={completed_callbacks} total_callback_duration_ns={total_callback_duration_ns} \
             max_callback_duration_ns={max_callback_duration_ns} loaded_lock_acquisitions={} \
             total_loaded_lock_wait_ns={} max_loaded_lock_wait_ns={}",
            started.elapsed().as_nanos(),
            live_host_diagnostics.loaded_lock_acquisitions,
            live_host_diagnostics.total_loaded_lock_wait_ns,
            live_host_diagnostics.max_loaded_lock_wait_ns,
        );
    }
}

#[test]
fn native_live_host_runtime_descriptor_includes_validation_report() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin("physics", PluginModuleKind::Runtime),
        );
    }

    let descriptor = host
        .runtime_behavior_descriptor("physics")
        .expect("loaded test plugin should return a descriptor");

    let validation = descriptor
        .validation_report
        .expect("runtime descriptor should carry validation report");
    assert_eq!(validation.plugin_id, "physics");
    assert_eq!(validation.module_kind, PluginModuleKind::Runtime);
    assert!(!validation.diagnostics.is_empty());
    assert!(validation
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("runtime behavior is missing")));
}

#[test]
fn native_live_host_runtime_behavior_reports_typed_unloaded_error() {
    let error = NativePluginLiveHost::default()
        .runtime_behavior_descriptor_result("physics")
        .expect_err("unloaded runtime plugin should produce typed runtime behavior error");

    assert!(matches!(
        &error,
        NativePluginRuntimeBehaviorError::RuntimePluginNotLoaded { plugin_id }
            if plugin_id == "physics"
    ));
    assert_eq!(
        error.to_string(),
        "plugin physics is not loaded in the runtime live host; run Hot Reload after building its native dynamic package"
    );
}

#[test]
fn native_live_host_runtime_broadcasts_and_snapshots_empty_when_no_plugins_loaded() {
    let host = NativePluginLiveHost::default();

    let dispatch = host
        .dispatch_runtime_plugin_command("play-mode.enter", b"{}")
        .expect("empty runtime live host should still dispatch as an empty report");
    assert_eq!(dispatch.command_name, "play-mode.enter");
    assert!(dispatch.calls.is_empty());
    assert!(dispatch.diagnostics.is_empty());
    assert!(dispatch.is_clean());
    assert_eq!(dispatch.failed_call_count(), 0);
    assert!(dispatch.combined_diagnostics().is_empty());

    let snapshot = host
        .save_runtime_plugin_states()
        .expect("empty runtime live host should still save an empty snapshot");
    assert!(snapshot.plugin_states.is_empty());
    assert!(snapshot.diagnostics.is_empty());
    assert!(snapshot.is_clean());
    assert!(snapshot.combined_diagnostics().is_empty());

    let restore = host
        .restore_runtime_plugin_states(&snapshot)
        .expect("empty runtime live host should still restore an empty snapshot");
    assert!(restore.calls.is_empty());
    assert!(restore.skipped_plugin_ids.is_empty());
    assert!(restore.diagnostics.is_empty());
    assert!(restore.is_clean());
    assert_eq!(restore.failed_call_count(), 0);
    assert!(restore.combined_diagnostics().is_empty());

    let play_snapshot = host
        .enter_runtime_play_mode()
        .expect("empty runtime live host should still enter play mode");
    assert_eq!(
        play_snapshot.enter_report.command_name,
        NATIVE_RUNTIME_PLAY_MODE_ENTER_COMMAND
    );
    assert!(play_snapshot.state_snapshot.plugin_states.is_empty());
    assert!(play_snapshot.is_clean());
    assert!(play_snapshot.combined_diagnostics().is_empty());
    let play_exit = host
        .exit_runtime_play_mode(&play_snapshot)
        .expect("empty runtime live host should still exit play mode");
    assert_eq!(
        play_exit.exit_report.command_name,
        NATIVE_RUNTIME_PLAY_MODE_EXIT_COMMAND
    );
    assert!(play_exit.restore_report.calls.is_empty());
    assert!(play_exit.is_clean());
    assert!(play_exit.combined_diagnostics().is_empty());
}

#[test]
fn native_live_host_runtime_snapshot_restore_reports_unloaded_plugins() {
    let host = NativePluginLiveHost::default();
    let snapshot = NativePluginRuntimeStateSnapshot {
        plugin_states: vec![NativePluginRuntimePluginState {
            plugin_id: "physics".to_string(),
            state_schema_version: Some(3),
            state: b"state".to_vec(),
        }],
        diagnostics: Vec::new(),
    };

    let restore = host
        .restore_runtime_plugin_states(&snapshot)
        .expect("unloaded plugins should be restore diagnostics, not host failures");
    assert!(restore.calls.is_empty());
    assert_eq!(restore.skipped_plugin_ids, vec!["physics".to_string()]);
    assert!(!restore.is_clean());
    assert_eq!(restore.failed_call_count(), 0);
    assert_eq!(
        restore.diagnostics,
        vec![
            "plugin physics is not loaded in the runtime live host; run Hot Reload after building its native dynamic package"
                .to_string()
        ]
    );
    assert_eq!(restore.combined_diagnostics(), restore.diagnostics);
}

#[test]
fn native_live_host_runtime_snapshot_restore_skips_schema_mismatch() {
    let host = NativePluginLiveHost::default();
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            native_live_host_test_plugin("physics", PluginModuleKind::Runtime),
        );
    }
    let snapshot = NativePluginRuntimeStateSnapshot {
        plugin_states: vec![NativePluginRuntimePluginState {
            plugin_id: "physics".to_string(),
            state_schema_version: Some(3),
            state: b"state".to_vec(),
        }],
        diagnostics: Vec::new(),
    };

    let restore = host
        .restore_runtime_plugin_states(&snapshot)
        .expect("schema mismatch should be a restore diagnostic, not host failure");

    assert!(restore.calls.is_empty());
    assert_eq!(restore.skipped_plugin_ids, vec!["physics".to_string()]);
    assert_eq!(restore.failed_call_count(), 0);
    assert_eq!(
        restore.diagnostics,
        vec![
            "runtime plugin physics restore-state skipped because snapshot state schema Some(3) does not match loaded state schema None"
                .to_string()
        ]
    );
    assert!(!restore.is_clean());
}

#[test]
fn native_live_host_runtime_snapshot_restore_borrows_state_payload_after_unlock() {
    let source = include_str!("../runtime_behavior.rs")
        .split_once("pub(super) fn restore_runtime_plugin_states_result")
        .expect("runtime state restore implementation should exist")
        .1
        .split_once("pub fn enter_runtime_play_mode")
        .expect("play mode implementation should follow restore")
        .0;

    assert!(source.contains("plugin_state.state.as_slice()"));
    assert!(!source.contains("plugin_state.state.clone()"));
}
