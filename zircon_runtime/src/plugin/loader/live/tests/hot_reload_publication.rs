use super::*;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use super::super::super::abi_declarations::{
    NativePluginByteSliceV3, NativePluginCallbackStatusV3, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
};
use super::runtime_behavior::{
    callback_concurrency_fixture_lock, counted_replacement_unload, replacement_unload_count,
    slow_callback_slot, slow_unload, state_restore_count, stateful_callback_test_behavior,
    successful_runtime_command, SlowCallbackProbe,
};

const RETAINED_ROLLBACK_RESTORE_FAILURE_DIAGNOSTIC: &[u8] = b"retained rollback restore failed\0";
const REPLACEMENT_UNLOAD_FAILURE_DIAGNOSTIC: &[u8] = b"replacement unload cleanup failed\0";

fn retained_rollback_restore_count() -> &'static AtomicUsize {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    &COUNT
}

fn replacement_unload_failure_count() -> &'static AtomicUsize {
    static COUNT: AtomicUsize = AtomicUsize::new(0);
    &COUNT
}

unsafe extern "C" fn retained_rollback_restore_failure(
    _state: NativePluginByteSliceV3,
) -> NativePluginCallbackStatusV3 {
    retained_rollback_restore_count().fetch_add(1, Ordering::SeqCst);
    NativePluginCallbackStatusV3 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
        diagnostics: RETAINED_ROLLBACK_RESTORE_FAILURE_DIAGNOSTIC.as_ptr().cast(),
    }
}

unsafe extern "C" fn replacement_unload_failure() -> NativePluginCallbackStatusV3 {
    replacement_unload_failure_count().fetch_add(1, Ordering::SeqCst);
    NativePluginCallbackStatusV3 {
        code: ZIRCON_NATIVE_PLUGIN_STATUS_ERROR,
        diagnostics: REPLACEMENT_UNLOAD_FAILURE_DIAGNOSTIC.as_ptr().cast(),
    }
}

fn hot_reload_with_poisoned_loaded_lock(
    mut old_behavior: NativePluginBehavior,
    replacement_behavior: NativePluginBehavior,
) -> (String, LoadedNativePlugin) {
    let host = Arc::new(NativePluginLiveHost::default());
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
                replacement_behavior,
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

    (error, old_plugin)
}

#[test]
fn hot_reload_reopens_the_retained_generation_when_loaded_lock_reacquisition_fails() {
    let _fixture = callback_concurrency_fixture_lock();
    state_restore_count().store(0, Ordering::SeqCst);
    replacement_unload_count().store(0, Ordering::SeqCst);
    let mut replacement_behavior = stateful_callback_test_behavior(successful_runtime_command);
    replacement_behavior.unload = Some(counted_replacement_unload);
    let (error, old_plugin) = hot_reload_with_poisoned_loaded_lock(
        stateful_callback_test_behavior(successful_runtime_command),
        replacement_behavior,
    );

    assert!(error.contains("native plugin live host lock is poisoned"));
    assert!(
        old_plugin.callback_owner_lease().is_ok(),
        "the failed hot reload must reopen callback admission for its retained generation"
    );
    assert_eq!(
        replacement_unload_count().load(Ordering::SeqCst),
        1,
        "the unpublished replacement must run its unload cleanup before the old generation reopens"
    );
    assert_eq!(
        state_restore_count().load(Ordering::SeqCst),
        2,
        "a failed publication must restore the saved runtime state to both replacement and retained generations"
    );
}

#[test]
fn hot_reload_reports_replacement_cleanup_failure_after_publication_fails() {
    let _fixture = callback_concurrency_fixture_lock();
    replacement_unload_failure_count().store(0, Ordering::SeqCst);
    let mut replacement_behavior = stateful_callback_test_behavior(successful_runtime_command);
    replacement_behavior.unload = Some(replacement_unload_failure);
    let (error, old_plugin) = hot_reload_with_poisoned_loaded_lock(
        stateful_callback_test_behavior(successful_runtime_command),
        replacement_behavior,
    );

    assert!(error.contains("native plugin live host lock is poisoned"));
    assert!(error.contains("replacement unload cleanup failed"));
    assert!(
        old_plugin.callback_owner_lease().is_ok(),
        "the retained generation should reopen after its restore succeeds"
    );
    assert_eq!(replacement_unload_failure_count().load(Ordering::SeqCst), 1);
}

#[test]
fn hot_reload_keeps_retained_generation_transition_active_when_rollback_restore_fails() {
    let _fixture = callback_concurrency_fixture_lock();
    retained_rollback_restore_count().store(0, Ordering::SeqCst);
    let host = NativePluginLiveHost::default();
    let mut old_behavior = stateful_callback_test_behavior(successful_runtime_command);
    old_behavior.restore_state = Some(retained_rollback_restore_failure);
    let old_plugin = native_live_host_test_plugin_with_behavior("physics", old_behavior);
    {
        let mut loaded = lock_loaded_native_plugins(&host.loaded)
            .expect("test should lock the native live host");
        loaded.insert(
            live_key(PluginModuleKind::Runtime, "physics"),
            old_plugin.clone(),
        );
    }
    let mut replacement_behavior = stateful_callback_test_behavior(successful_runtime_command);
    replacement_behavior.restore_state = Some(hot_reload_restore_state_failure);

    let error = host
        .hot_reload_reported_plugin(
            NativePluginLoadReport::from_loaded(vec![native_live_host_test_plugin_with_behavior(
                "physics",
                replacement_behavior,
            )]),
            std::path::Path::new("reload-root"),
            "physics",
            PluginModuleKind::Runtime,
        )
        .expect_err("a failed rollback restore must reject the hot reload");

    assert!(error.contains("plugin physics hot reload failed while restoring runtime state"));
    assert!(error.contains("retained rollback restore failed"));
    assert_eq!(retained_rollback_restore_count().load(Ordering::SeqCst), 1);
    assert!(
        old_plugin.callback_owner_lease().is_err(),
        "the old generation must remain closed when its rollback restore failed"
    );
}

#[test]
fn hot_reload_keeps_retained_generation_transition_active_when_publication_rollback_restore_fails()
{
    let _fixture = callback_concurrency_fixture_lock();
    retained_rollback_restore_count().store(0, Ordering::SeqCst);
    let mut old_behavior = stateful_callback_test_behavior(successful_runtime_command);
    old_behavior.restore_state = Some(retained_rollback_restore_failure);
    let (error, old_plugin) = hot_reload_with_poisoned_loaded_lock(
        old_behavior,
        stateful_callback_test_behavior(successful_runtime_command),
    );

    assert!(error.contains("native plugin live host lock is poisoned"));
    assert!(error.contains("plugin physics hot reload failed while restoring runtime state"));
    assert!(error.contains("retained rollback restore failed"));
    assert_eq!(retained_rollback_restore_count().load(Ordering::SeqCst), 1);
    assert!(
        old_plugin.callback_owner_lease().is_err(),
        "the old generation must remain closed when publication rollback cannot restore it"
    );
}
