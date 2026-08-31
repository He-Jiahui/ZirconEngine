use super::super::super::benchmark_harness::{
    BenchmarkMeasurement, BenchmarkRunMetadata, BenchmarkWorkerCompletionGate,
    BenchmarkWorkerStartGate,
};
use super::runtime_behavior::{
    callback_test_behavior, native_live_host_test_plugin_with_behavior, successful_runtime_command,
};
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
fn native_callback_snapshot_defers_lease_until_foreign_call() {
    let plugin = native_live_host_test_plugin_with_behavior(
        "physics",
        callback_test_behavior(successful_runtime_command),
    );
    let snapshot = plugin
        .runtime_behavior_snapshot()
        .expect("snapshot should retain the native library generation");

    assert_eq!(plugin.callback_diagnostics().active_callbacks, 0);
    plugin
        .begin_lifecycle_transition()
        .expect("a passive snapshot must not block a lifecycle transition");

    let report = snapshot.invoke_command("probe", b"");
    assert_eq!(report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("lifecycle transition is active")));

    plugin.cancel_lifecycle_transition();
}

#[test]
fn native_callback_snapshot_keeps_generation_alive_after_loaded_plugin_releases() {
    let plugin = native_live_host_test_plugin_with_behavior(
        "physics",
        callback_test_behavior(successful_runtime_command),
    );
    let generation = Arc::clone(&plugin.library);
    let snapshot = plugin
        .runtime_behavior_snapshot()
        .expect("snapshot should retain the native library generation");

    assert_eq!(Arc::strong_count(&generation), 3);
    drop(plugin);
    assert_eq!(Arc::strong_count(&generation), 2);

    let report = snapshot.invoke_command("probe", b"");
    assert_eq!(report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);

    drop(snapshot);
    assert_eq!(Arc::strong_count(&generation), 1);
}

#[test]
fn native_callback_snapshot_without_behavior_reports_missing_before_transition_admission() {
    let plugin = native_live_host_test_plugin("physics", PluginModuleKind::Runtime);
    let snapshot = plugin
        .runtime_behavior_snapshot()
        .expect("snapshot should retain the native library generation");

    plugin
        .begin_lifecycle_transition()
        .expect("a missing behavior snapshot must not block a lifecycle transition");

    let report = snapshot.invoke_command("probe", b"");
    assert_eq!(report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
    assert_eq!(
        report.diagnostics,
        vec!["native plugin runtime behavior is missing".to_owned()]
    );
    assert_eq!(plugin.callback_diagnostics().active_callbacks, 0);

    plugin.cancel_lifecycle_transition();
}

#[test]
fn native_editor_command_binding_admits_declared_command_and_retains_generation_owner() {
    let plugin = editor_plugin_with_behavior(callback_test_behavior(successful_runtime_command));
    let binding = plugin
        .bind_editor_command("probe")
        .expect("declared editor command should produce an executable binding");

    assert_eq!(binding.plugin_id(), "physics-editor");
    assert_eq!(binding.command_name(), "probe");
    assert_eq!(binding.payload_schema_id(), "bytes");
    assert_eq!(binding.max_output_bytes(), 0);
    drop(plugin);
    let report = binding.invoke(b"payload");
    assert_eq!(report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_OK);
}

#[test]
fn native_editor_command_binding_rejects_undeclared_command_at_admission() {
    let plugin = editor_plugin_with_behavior(callback_test_behavior(successful_runtime_command));
    let error = plugin
        .bind_editor_command("missing")
        .expect_err("undeclared editor command must not produce a binding");

    assert!(matches!(
        error,
        crate::plugin::native_plugin_loader::NativePluginEditorCommandBindingError::UndeclaredCommand {
            plugin_id,
            command_name,
        } if plugin_id == "physics-editor" && command_name == "missing"
    ));
}

#[test]
fn native_editor_command_binding_rejects_missing_callback_at_admission() {
    let mut behavior = callback_test_behavior(successful_runtime_command);
    behavior.invoke_command = None;
    let plugin = editor_plugin_with_behavior(behavior);
    let error = plugin
        .bind_editor_command("probe")
        .expect_err("editor command without callback must not produce a binding");

    assert!(matches!(
        error,
        crate::plugin::native_plugin_loader::NativePluginEditorCommandBindingError::MissingInvokeCommandCallback {
            plugin_id,
        } if plugin_id == "physics-editor"
    ));
}

#[test]
fn native_editor_command_binding_rejects_missing_editor_behavior_at_admission() {
    let plugin = native_live_host_test_plugin("physics-editor", PluginModuleKind::Editor);
    let error = plugin
        .bind_editor_command("probe")
        .expect_err("editor plugin without behavior must not produce a binding");

    assert!(matches!(
        error,
        crate::plugin::native_plugin_loader::NativePluginEditorCommandBindingError::MissingEditorBehavior {
            plugin_id,
        } if plugin_id == "physics-editor"
    ));
}

#[test]
fn native_editor_command_binding_fails_closed_during_lifecycle_transition() {
    let plugin = editor_plugin_with_behavior(callback_test_behavior(successful_runtime_command));
    let binding = plugin
        .bind_editor_command("probe")
        .expect("declared editor command should produce an executable binding");

    plugin
        .begin_lifecycle_transition()
        .expect("passive binding owner should not count as an active callback");
    let report = binding.invoke(b"payload");
    assert_eq!(report.status_code, ZIRCON_NATIVE_PLUGIN_STATUS_ERROR);
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("lifecycle transition is active")));
    plugin.cancel_lifecycle_transition();
}

fn editor_plugin_with_behavior(behavior: NativePluginBehavior) -> LoadedNativePlugin {
    let mut plugin = native_live_host_test_plugin_with_behavior("physics-editor", behavior);
    plugin.editor_entry_report = plugin.runtime_entry_report.take();
    plugin
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

const CALLBACK_BENCHMARK_TOTAL_LEASES: usize = 1_000_000;
const CALLBACK_BENCHMARK_WARMUP_LEASES: usize = 10_000;

#[test]
#[ignore = "manual 1-thread x 1M same-plugin callback lease benchmark"]
fn native_callback_atomic_lease_1_thread_benchmark() {
    run_callback_lease_benchmark(1);
}

#[test]
#[ignore = "manual 2-thread x 1M same-plugin callback lease benchmark"]
fn native_callback_atomic_lease_2_thread_benchmark() {
    run_callback_lease_benchmark(2);
}

#[test]
#[ignore = "manual 16-thread x 1M same-plugin callback lease benchmark"]
fn native_callback_atomic_lease_16_thread_benchmark() {
    run_callback_lease_benchmark(16);
}

#[test]
#[ignore = "manual 64-thread x 1M same-plugin callback lease benchmark"]
fn native_callback_atomic_lease_64_thread_benchmark() {
    run_callback_lease_benchmark(64);
}

fn run_callback_lease_benchmark(thread_count: usize) {
    let metadata = BenchmarkRunMetadata::from_environment(
        "native_callback_atomic_lease",
        format!("threads={thread_count},total_leases={CALLBACK_BENCHMARK_TOTAL_LEASES}"),
    )
    .expect("benchmark metadata must be bound to a managed optimized-profile run");
    let plugin = Arc::new(native_live_host_test_plugin_with_behavior(
        "physics",
        callback_test_behavior(successful_runtime_command),
    ));
    plugin.set_callback_diagnostics_enabled(false);

    run_callback_lease_batch(&plugin, thread_count, CALLBACK_BENCHMARK_WARMUP_LEASES);
    let workers = callback_lease_workers(
        Arc::clone(&plugin),
        thread_count,
        CALLBACK_BENCHMARK_TOTAL_LEASES,
    );
    workers.start.wait_until_ready();
    let started = std::time::Instant::now();
    workers.start.start();
    workers.wait_for_completion();
    let elapsed = started.elapsed();
    for worker in workers.threads {
        worker.join().expect("benchmark worker should not panic");
    }

    let diagnostics = plugin.callback_diagnostics();
    assert_eq!(diagnostics.active_callbacks, 0);
    assert_eq!(diagnostics.callback_state_mutex_acquisitions, 0);
    metadata.emit(BenchmarkMeasurement {
        warmup_operations: CALLBACK_BENCHMARK_WARMUP_LEASES as u64,
        measured_operations: CALLBACK_BENCHMARK_TOTAL_LEASES as u64,
        elapsed,
        counters: &[(
            "state_mutex_acquires",
            diagnostics.callback_state_mutex_acquisitions,
        )],
        latency_sample: None,
    });
}

struct CallbackLeaseWorkers {
    start: BenchmarkWorkerStartGate,
    completion: BenchmarkWorkerCompletionGate,
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl CallbackLeaseWorkers {
    fn wait_for_completion(&self) {
        self.completion.wait();
    }
}

fn run_callback_lease_batch(
    plugin: &Arc<LoadedNativePlugin>,
    thread_count: usize,
    total_leases: usize,
) {
    let workers = callback_lease_workers(Arc::clone(plugin), thread_count, total_leases);
    workers.start.wait_until_ready();
    workers.start.start();
    workers.wait_for_completion();
    for worker in workers.threads {
        worker
            .join()
            .expect("benchmark warm-up worker should not panic");
    }
}

fn callback_lease_workers(
    plugin: Arc<LoadedNativePlugin>,
    thread_count: usize,
    total_leases: usize,
) -> CallbackLeaseWorkers {
    let start = BenchmarkWorkerStartGate::new(thread_count);
    let completion = BenchmarkWorkerCompletionGate::new(thread_count);
    let base = total_leases / thread_count;
    let remainder = total_leases % thread_count;
    let threads = (0..thread_count)
        .map(|worker_index| {
            let plugin = Arc::clone(&plugin);
            let worker_start = start.worker_start();
            let worker_completion = completion.worker_completion();
            let iterations = base + usize::from(worker_index < remainder);
            std::thread::spawn(move || {
                let _worker_completion = worker_completion;
                worker_start.await_start();
                for _ in 0..iterations {
                    let lease = plugin
                        .callback_owner_lease()
                        .expect("benchmark callback lease should be admitted");
                    std::hint::black_box(lease);
                }
            })
        })
        .collect();
    CallbackLeaseWorkers {
        start,
        completion,
        threads,
    }
}
