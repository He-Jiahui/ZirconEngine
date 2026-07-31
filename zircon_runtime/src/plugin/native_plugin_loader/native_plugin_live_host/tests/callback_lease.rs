use super::runtime_behavior::{callback_test_behavior, successful_runtime_command};
use super::*;
use crate::plugin::native_plugin_loader::loaded_native_plugin::{
    NativePluginCallbackLeaseError, NativePluginLifecycleTransitionError,
};

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn native_callback_stable_owner_source_has_no_per_call_state_mutex() {
    let source = include_str!("../../loaded_native_plugin.rs");

    assert!(!source.contains("callback_state: Mutex"));
    assert!(source.contains("callback_activity: AtomicUsize"));
    assert!(source.contains("diagnostic_shards:"));
    assert!(source.contains(".then(Instant::now)"));
}

#[test]
fn native_callback_owner_uses_atomic_transition_and_reports_zero_state_mutex_acquires() {
    let plugin = native_live_host_test_plugin_with_behavior(
        "physics",
        callback_test_behavior(successful_runtime_command),
    );
    let lease = plugin
        .callback_owner_lease()
        .expect("stable callback lease should be available");

    assert!(matches!(
        plugin.begin_lifecycle_transition(),
        Err(NativePluginLifecycleTransitionError::ActiveCallbacks { count: 1 })
    ));
    drop(lease);
    plugin
        .begin_lifecycle_transition()
        .expect("transition should begin after the last lease exits");
    assert!(matches!(
        plugin.callback_owner_lease(),
        Err(NativePluginCallbackLeaseError::LifecycleTransitionActive)
    ));

    let transitioning = plugin.callback_diagnostics();
    assert_eq!(transitioning.active_callbacks, 0);
    assert!(transitioning.lifecycle_transition_active);
    assert_eq!(transitioning.callback_state_mutex_acquisitions, 0);
    plugin.cancel_lifecycle_transition();
    drop(
        plugin
            .callback_owner_lease()
            .expect("callbacks should resume after transition cancellation"),
    );
}

#[test]
fn native_callback_diagnostics_off_skips_callback_measurement_updates() {
    let plugin = native_live_host_test_plugin_with_behavior(
        "physics",
        callback_test_behavior(successful_runtime_command),
    );
    plugin.set_callback_diagnostics_enabled(false);

    for _ in 0..128 {
        assert_eq!(
            plugin.invoke_runtime_command("probe", b"").status_code,
            ZIRCON_NATIVE_PLUGIN_STATUS_OK
        );
    }

    let disabled = plugin.callback_diagnostics();
    assert!(!disabled.diagnostics_enabled);
    assert_eq!(disabled.completed_callbacks, 0);
    assert_eq!(disabled.total_callback_duration_ns, 0);
    assert_eq!(disabled.max_callback_duration_ns, 0);
    assert_eq!(disabled.callback_state_mutex_acquisitions, 0);

    plugin.set_callback_diagnostics_enabled(true);
    assert_eq!(
        plugin.invoke_runtime_command("probe", b"").status_code,
        ZIRCON_NATIVE_PLUGIN_STATUS_OK
    );
    let enabled = plugin.callback_diagnostics();
    assert!(enabled.diagnostics_enabled);
    assert_eq!(enabled.completed_callbacks, 1);
    assert_eq!(enabled.diagnostic_shard_count, 64);
}

#[test]
fn native_callback_atomic_transition_survives_64_thread_lease_races() {
    let plugin = Arc::new(native_live_host_test_plugin_with_behavior(
        "physics",
        callback_test_behavior(successful_runtime_command),
    ));
    plugin.set_callback_diagnostics_enabled(false);
    let start = Arc::new(std::sync::Barrier::new(65));
    let successful_leases = Arc::new(AtomicUsize::new(0));
    let workers = (0..64)
        .map(|_| {
            let plugin = Arc::clone(&plugin);
            let start = Arc::clone(&start);
            let successful_leases = Arc::clone(&successful_leases);
            std::thread::spawn(move || {
                start.wait();
                for _ in 0..1_000 {
                    if let Ok(lease) = plugin.callback_owner_lease() {
                        successful_leases.fetch_add(1, Ordering::Relaxed);
                        std::hint::black_box(&lease);
                    }
                }
            })
        })
        .collect::<Vec<_>>();
    start.wait();

    let mut successful_transitions = 0;
    for _ in 0..10_000 {
        match plugin.begin_lifecycle_transition() {
            Ok(()) => {
                successful_transitions += 1;
                assert!(matches!(
                    plugin.callback_owner_lease(),
                    Err(NativePluginCallbackLeaseError::LifecycleTransitionActive)
                ));
                plugin.cancel_lifecycle_transition();
            }
            Err(NativePluginLifecycleTransitionError::ActiveCallbacks { count }) => {
                assert!(count > 0);
            }
            Err(NativePluginLifecycleTransitionError::AlreadyTransitioning) => {
                panic!("one transition caller cannot observe a second active transition")
            }
        }
    }
    for worker in workers {
        worker
            .join()
            .expect("callback lease worker should not panic");
    }

    assert!(successful_leases.load(Ordering::Relaxed) > 0);
    if successful_transitions == 0 {
        plugin
            .begin_lifecycle_transition()
            .expect("transition must succeed once all callback workers finish");
        plugin.cancel_lifecycle_transition();
    }
    let diagnostics = plugin.callback_diagnostics();
    assert_eq!(diagnostics.active_callbacks, 0);
    assert_eq!(diagnostics.callback_state_mutex_acquisitions, 0);
}

#[test]
#[ignore = "manual 1/2/16/64-thread x 1M same-plugin callback lease benchmark"]
fn native_callback_atomic_lease_1_2_16_64_thread_benchmark() {
    const TOTAL_LEASES: usize = 1_000_000;
    for thread_count in [1_usize, 2, 16, 64] {
        let plugin = Arc::new(native_live_host_test_plugin_with_behavior(
            "physics",
            callback_test_behavior(successful_runtime_command),
        ));
        plugin.set_callback_diagnostics_enabled(false);
        let start = Arc::new(std::sync::Barrier::new(thread_count + 1));
        let base = TOTAL_LEASES / thread_count;
        let remainder = TOTAL_LEASES % thread_count;
        let workers = (0..thread_count)
            .map(|worker_index| {
                let plugin = Arc::clone(&plugin);
                let start = Arc::clone(&start);
                let iterations = base + usize::from(worker_index < remainder);
                std::thread::spawn(move || {
                    start.wait();
                    for _ in 0..iterations {
                        let lease = plugin
                            .callback_owner_lease()
                            .expect("benchmark callback lease should be admitted");
                        std::hint::black_box(&lease);
                    }
                })
            })
            .collect::<Vec<_>>();
        let started = std::time::Instant::now();
        start.wait();
        for worker in workers {
            worker.join().expect("benchmark worker should not panic");
        }
        let elapsed = started.elapsed();
        let diagnostics = plugin.callback_diagnostics();
        assert_eq!(diagnostics.active_callbacks, 0);
        assert_eq!(diagnostics.callback_state_mutex_acquisitions, 0);
        eprintln!(
            "native callback atomic lease: threads={thread_count} total={TOTAL_LEASES} \
             elapsed_ns={} leases_per_second={:.2} state_mutex_acquires={}",
            elapsed.as_nanos(),
            TOTAL_LEASES as f64 / elapsed.as_secs_f64(),
            diagnostics.callback_state_mutex_acquisitions,
        );
    }
}
