use super::*;

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Barrier, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const STABLE_CALLS_PER_THREAD: usize = 1_000_000;
const LATENCY_SAMPLES_PER_THREAD: usize = 2_048;
const SERIALIZED_WAIT_SAMPLES_PER_THREAD: usize = 4_096;
const SIXTEEN_THREAD_MIN_NON_COLLAPSE_RATIO: f64 = 0.5;
const LOCK_FREE_TO_SERIALIZED_CONTROL_MIN_RATIO: f64 = 1.1;

#[test]
fn bridge_import_and_weak_cache_stable_paths_do_not_contain_mutexes() {
    let import = include_str!("../../../plugin/bridge/import.rs");
    let weak = include_str!("../../../plugin/bridge/weak.rs");
    let table = include_str!("../../../plugin/bridge/table.rs");
    let diagnostics = include_str!("../../../core/framework/bridge/diagnostics.rs");

    assert_eq!(
        exclusive_wait_site_count([import, weak, table, diagnostics]),
        0
    );
    assert!(import.contains("ArcSwapOption"));
    assert!(weak.contains("ProviderSnapshot"));
}

#[test]
fn stable_bridge_calls_scale_across_one_and_sixteen_threads() {
    let (bridge, _, _) = finalized_import(23);
    let single_thread = sample_stable_bridge_calls(&bridge, 1);
    let sixteen_threads = sample_stable_bridge_calls(&bridge, 16);
    let serialized_control = sample_serialized_bridge_calls(&bridge, 16);
    let serialized_wait_sample = sample_serialized_bridge_waits(&bridge, 16);

    assert_eq!(single_thread.total, single_thread.expected_total(23));
    assert_eq!(sixteen_threads.total, sixteen_threads.expected_total(23));
    assert!(
        sixteen_threads.throughput()
            >= single_thread.throughput() * SIXTEEN_THREAD_MIN_NON_COLLAPSE_RATIO,
        "sixteen-thread aggregate throughput regressed below the lock-free budget: {:.0}/s < {:.0}/s * {:.2}",
        sixteen_threads.throughput(),
        single_thread.throughput(),
        SIXTEEN_THREAD_MIN_NON_COLLAPSE_RATIO,
    );
    assert_eq!(
        serialized_control.total,
        serialized_control.expected_total(23)
    );
    assert_eq!(
        serialized_wait_sample.total,
        serialized_wait_sample.expected_total(23)
    );
    assert_eq!(
        serialized_wait_sample.mutex_acquisitions,
        serialized_wait_sample.call_count
    );
    assert!(
        sixteen_threads.throughput()
            >= serialized_control.throughput() * LOCK_FREE_TO_SERIALIZED_CONTROL_MIN_RATIO,
        "lock-free bridge path did not beat the same-machine serialized control: {:.0}/s < {:.0}/s * {:.2}",
        sixteen_threads.throughput(),
        serialized_control.throughput(),
        LOCK_FREE_TO_SERIALIZED_CONTROL_MIN_RATIO,
    );

    let exclusive_wait_sites = exclusive_wait_site_count([
        include_str!("../../../plugin/bridge/import.rs"),
        include_str!("../../../plugin/bridge/weak.rs"),
        include_str!("../../../plugin/bridge/table.rs"),
        include_str!("../../../core/framework/bridge/diagnostics.rs"),
    ]);
    for sample in [&single_thread, &sixteen_threads] {
        println!(
            "bridge stable snapshot: threads={}, calls={}, throughput={:.0}/s, call_p95={:?}, call_p99={:?}, worker_p99={:?}, stable_path_exclusive_wait_sites={exclusive_wait_sites}",
            sample.thread_count,
            sample.call_count,
            sample.throughput(),
            sample.call_p95,
            sample.call_p99,
            sample.worker_p99,
        );
    }
    println!(
        "bridge serialized throughput control: threads=16, calls={}, throughput={:.0}/s",
        serialized_control.call_count,
        serialized_control.throughput(),
    );
    println!(
        "bridge serialized wait sample: threads=16, calls={}, mutex_acquisitions={}, mutex_wait_total={:?}",
        serialized_wait_sample.call_count,
        serialized_wait_sample.mutex_acquisitions,
        serialized_wait_sample.mutex_wait_total,
    );
}

#[test]
fn concurrent_reload_and_disable_keep_provider_payload_within_the_generation_window() {
    let (bridge, table, slot) = finalized_import(1);
    let start = Arc::new(Barrier::new(9));
    let mut callers = Vec::new();
    for _ in 0..8 {
        let bridge = bridge.clone();
        let table = table.clone();
        let start = Arc::clone(&start);
        callers.push(thread::spawn(move || {
            start.wait();
            for _ in 0..20_000 {
                let generation_before = table.entry(slot).unwrap().generation();
                let result = bridge.call(|provider| provider.sample_temperature());
                let generation_after = table.entry(slot).unwrap().generation();
                match result {
                    Ok(value) => assert_provider_matches_generation_window(
                        value,
                        generation_before,
                        generation_after,
                    ),
                    Err(BridgeError::NotEnabled) => {}
                    Err(error) => panic!("stable binding became invalid during reload: {error:?}"),
                }
            }
        }));
    }

    start.wait();
    for value in 2..=1_001 {
        table.set_enabled(slot, false).expect("disable provider");
        table
            .replace_provider::<dyn WeatherQueryInterface>(
                slot,
                Arc::new(WeatherQueryProvider { temperature: value }),
            )
            .expect("replace provider");
        table.set_enabled(slot, true).expect("enable provider");
        let published_generation = table.entry(slot).unwrap().generation();
        assert_eq!(published_generation, (value as u32 - 1) * 2);
        assert_eq!(
            bridge.call(|provider| provider.sample_temperature()),
            Ok(value),
            "provider payload must match every fully published generation"
        );
    }

    for caller in callers {
        caller.join().expect("bridge caller");
    }
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Ok(1_001)
    );
}

#[test]
fn restoring_a_different_enabled_provider_advances_the_published_generation() {
    let (bridge, table, slot) = finalized_import(5);
    assert_eq!(bridge.call(|provider| provider.sample_temperature()), Ok(5));
    let original_generation = table.entry(slot).unwrap().generation();
    let owner = table.entry(slot).unwrap().owner();

    let mut replacement_registry = RuntimeExtensionRegistry::default();
    let replacement_owner = replacement_registry
        .intern_plugin_module("weather.runtime")
        .unwrap();
    replacement_registry
        .export_interface::<dyn WeatherQueryInterface>(
            replacement_owner,
            Arc::new(WeatherQueryProvider { temperature: 31 }),
        )
        .unwrap();

    table.restore_owner_exports_with_report(
        owner,
        replacement_registry.interface_exports_owned_by(replacement_owner),
    );

    assert_eq!(
        table.entry(slot).unwrap().generation(),
        original_generation + 2
    );
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Ok(31)
    );
}

#[test]
fn unbind_does_not_revoke_an_in_flight_provider_arc() {
    let mut registry = RuntimeExtensionRegistry::default();
    let consumer = registry.intern_plugin_module("climate.runtime").unwrap();
    let provider_owner = registry.intern_plugin_module("weather.runtime").unwrap();
    let bridge: BridgeImport<dyn WeatherQueryInterface> =
        registry.import_interface(consumer).unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            provider_owner,
            Arc::new(WeatherQueryProvider { temperature: 41 }),
        )
        .unwrap();
    registry.finalize_bridge_imports();

    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let caller = {
        let bridge = bridge.clone();
        let entered = Arc::clone(&entered);
        let release = Arc::clone(&release);
        thread::spawn(move || {
            bridge.call(|provider| {
                entered.wait();
                release.wait();
                provider.sample_temperature()
            })
        })
    };

    entered.wait();
    registry.unbind_interface_imports_owned_by(consumer);
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Err(BridgeError::Absent)
    );
    release.wait();
    assert_eq!(caller.join().unwrap(), Ok(41));
}

#[test]
fn replaced_provider_lives_until_in_flight_call_and_cached_snapshot_finish() {
    let drops = Arc::new(AtomicUsize::new(0));
    let mut registry = RuntimeExtensionRegistry::default();
    let consumer = registry.intern_plugin_module("climate.runtime").unwrap();
    let provider_owner = registry.intern_plugin_module("weather.runtime").unwrap();
    let bridge: BridgeImport<dyn WeatherQueryInterface> =
        registry.import_interface(consumer).unwrap();
    let old_provider: Arc<dyn WeatherQueryInterface> = Arc::new(DropTrackedWeatherProvider {
        temperature: 7,
        drops: Arc::clone(&drops),
    });
    registry
        .export_interface::<dyn WeatherQueryInterface>(provider_owner, old_provider)
        .unwrap();
    registry.finalize_bridge_imports();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    drop(registry);

    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let caller = {
        let bridge = bridge.clone();
        let entered = Arc::clone(&entered);
        let release = Arc::clone(&release);
        thread::spawn(move || {
            bridge.call(|provider| {
                entered.wait();
                release.wait();
                provider.sample_temperature()
            })
        })
    };

    entered.wait();
    table
        .replace_provider::<dyn WeatherQueryInterface>(
            slot,
            Arc::new(WeatherQueryProvider { temperature: 19 }),
        )
        .unwrap();
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    release.wait();
    assert_eq!(caller.join().unwrap(), Ok(7));
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Ok(19)
    );
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn pinned_provider_lives_until_guard_drops_after_reload_and_deactivate() {
    let drops = Arc::new(AtomicUsize::new(0));
    let (bridge, table, slot) = finalized_drop_tracked_import(37, Arc::clone(&drops));
    let guard = crate::plugin::WeakBridge::<dyn WeatherQueryInterface>::owned(table.clone())
        .pin()
        .expect("initial provider pin");

    table
        .replace_provider::<dyn WeatherQueryInterface>(
            slot,
            Arc::new(WeatherQueryProvider { temperature: 43 }),
        )
        .expect("replace provider");
    table.set_enabled(slot, false).expect("deactivate provider");

    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Err(BridgeError::NotEnabled)
    );
    assert_eq!(guard.sample_temperature(), 37);
    assert_eq!(drops.load(Ordering::SeqCst), 0);

    drop(guard);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn cached_snapshot_does_not_retain_provider_after_reload_without_another_call() {
    let drops = Arc::new(AtomicUsize::new(0));
    let (bridge, table, slot) = finalized_drop_tracked_import(11, Arc::clone(&drops));
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Ok(11)
    );

    table
        .replace_provider::<dyn WeatherQueryInterface>(
            slot,
            Arc::new(WeatherQueryProvider { temperature: 17 }),
        )
        .unwrap();

    assert_eq!(drops.load(Ordering::SeqCst), 1);
}

#[test]
fn cached_snapshot_does_not_retain_provider_after_deactivate() {
    let drops = Arc::new(AtomicUsize::new(0));
    let (bridge, table, _) = finalized_drop_tracked_import(13, Arc::clone(&drops));
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Ok(13)
    );
    let owner = table.entries()[0].owner();

    table.deactivate_owner(owner);

    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Err(BridgeError::NotEnabled)
    );
}

#[test]
fn callback_panic_cannot_poison_lock_free_bridge_snapshots() {
    let (bridge, _, _) = finalized_import(29);
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _ = bridge.call::<()>(|_| panic!("callback failure"));
    }));
    assert!(panic.is_err());
    assert_eq!(
        bridge.call(|provider| provider.sample_temperature()),
        Ok(29)
    );
}

#[derive(Debug)]
struct StableBridgeCallSample {
    thread_count: usize,
    call_count: usize,
    elapsed: Duration,
    call_p95: Duration,
    call_p99: Duration,
    worker_p99: Duration,
    total: i64,
}

impl StableBridgeCallSample {
    fn throughput(&self) -> f64 {
        self.call_count as f64 / self.elapsed.as_secs_f64().max(f64::MIN_POSITIVE)
    }

    fn expected_total(&self, value: i32) -> i64 {
        self.call_count as i64 * i64::from(value)
    }
}

#[derive(Debug)]
struct SerializedBridgeCallSample {
    call_count: usize,
    elapsed: Duration,
    total: i64,
}

impl SerializedBridgeCallSample {
    fn throughput(&self) -> f64 {
        self.call_count as f64 / self.elapsed.as_secs_f64().max(f64::MIN_POSITIVE)
    }

    fn expected_total(&self, value: i32) -> i64 {
        self.call_count as i64 * i64::from(value)
    }
}

#[derive(Debug)]
struct SerializedMutexWaitSample {
    call_count: usize,
    mutex_acquisitions: usize,
    mutex_wait_total: Duration,
    total: i64,
}

impl SerializedMutexWaitSample {
    fn expected_total(&self, value: i32) -> i64 {
        self.call_count as i64 * i64::from(value)
    }
}

fn sample_stable_bridge_calls(
    bridge: &BridgeImport<dyn WeatherQueryInterface>,
    thread_count: usize,
) -> StableBridgeCallSample {
    let ready = Arc::new(Barrier::new(thread_count + 1));
    let start = Arc::new(Barrier::new(thread_count + 1));
    let mut workers = Vec::with_capacity(thread_count);
    for _ in 0..thread_count {
        let bridge = bridge.clone();
        let ready = Arc::clone(&ready);
        let start = Arc::clone(&start);
        workers.push(thread::spawn(move || {
            ready.wait();
            start.wait();
            let worker_started = Instant::now();
            let mut total = 0_i64;
            let mut samples = Vec::with_capacity(LATENCY_SAMPLES_PER_THREAD);
            for call_index in 0..STABLE_CALLS_PER_THREAD {
                if call_index < LATENCY_SAMPLES_PER_THREAD {
                    let call_started = Instant::now();
                    let value = bridge
                        .call(|provider| provider.sample_temperature())
                        .expect("stable bridge call");
                    samples.push(call_started.elapsed());
                    total += i64::from(value);
                } else {
                    total += i64::from(
                        bridge
                            .call(|provider| provider.sample_temperature())
                            .expect("stable bridge call"),
                    );
                }
            }
            (total, worker_started.elapsed(), samples)
        }));
    }
    ready.wait();
    let started = Instant::now();
    start.wait();

    let mut worker_durations = Vec::with_capacity(thread_count);
    let mut call_latencies = Vec::with_capacity(thread_count * LATENCY_SAMPLES_PER_THREAD);
    let mut total = 0_i64;
    for worker in workers {
        let (worker_total, worker_duration, mut samples) = worker.join().expect("bridge caller");
        total += worker_total;
        worker_durations.push(worker_duration);
        call_latencies.append(&mut samples);
    }
    worker_durations.sort_unstable();
    call_latencies.sort_unstable();

    StableBridgeCallSample {
        thread_count,
        call_count: thread_count * STABLE_CALLS_PER_THREAD,
        elapsed: started.elapsed(),
        call_p95: percentile(&call_latencies, 95),
        call_p99: percentile(&call_latencies, 99),
        worker_p99: percentile(&worker_durations, 99),
        total,
    }
}

fn sample_serialized_bridge_calls(
    bridge: &BridgeImport<dyn WeatherQueryInterface>,
    thread_count: usize,
) -> SerializedBridgeCallSample {
    let serialized = Arc::new(Mutex::new(()));
    let ready = Arc::new(Barrier::new(thread_count + 1));
    let start = Arc::new(Barrier::new(thread_count + 1));
    let mut workers = Vec::with_capacity(thread_count);
    for _ in 0..thread_count {
        let bridge = bridge.clone();
        let serialized = Arc::clone(&serialized);
        let ready = Arc::clone(&ready);
        let start = Arc::clone(&start);
        workers.push(thread::spawn(move || {
            ready.wait();
            start.wait();
            let mut total = 0_i64;
            for _ in 0..STABLE_CALLS_PER_THREAD {
                let _guard = serialized
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                total += i64::from(
                    bridge
                        .call(|provider| provider.sample_temperature())
                        .expect("serialized bridge control call"),
                );
            }
            total
        }));
    }
    ready.wait();
    let started = Instant::now();
    start.wait();

    let mut total = 0_i64;
    for worker in workers {
        total += worker.join().expect("serialized bridge caller");
    }
    let call_count = thread_count * STABLE_CALLS_PER_THREAD;
    SerializedBridgeCallSample {
        call_count,
        elapsed: started.elapsed(),
        total,
    }
}

fn sample_serialized_bridge_waits(
    bridge: &BridgeImport<dyn WeatherQueryInterface>,
    thread_count: usize,
) -> SerializedMutexWaitSample {
    let serialized = Arc::new(Mutex::new(()));
    let ready = Arc::new(Barrier::new(thread_count + 1));
    let start = Arc::new(Barrier::new(thread_count + 1));
    let mut workers = Vec::with_capacity(thread_count);
    for _ in 0..thread_count {
        let bridge = bridge.clone();
        let serialized = Arc::clone(&serialized);
        let ready = Arc::clone(&ready);
        let start = Arc::clone(&start);
        workers.push(thread::spawn(move || {
            ready.wait();
            start.wait();
            let mut mutex_acquisitions = 0_usize;
            let mut mutex_wait_total = Duration::ZERO;
            let mut total = 0_i64;
            for _ in 0..SERIALIZED_WAIT_SAMPLES_PER_THREAD {
                let wait_started = Instant::now();
                let _guard = serialized
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                mutex_wait_total += wait_started.elapsed();
                mutex_acquisitions += 1;
                total += i64::from(
                    bridge
                        .call(|provider| provider.sample_temperature())
                        .expect("serialized bridge wait sample call"),
                );
            }
            (mutex_acquisitions, mutex_wait_total, total)
        }));
    }
    ready.wait();
    start.wait();

    let mut mutex_acquisitions = 0_usize;
    let mut mutex_wait_total = Duration::ZERO;
    let mut total = 0_i64;
    for worker in workers {
        let (worker_acquisitions, worker_wait, worker_total) =
            worker.join().expect("serialized bridge wait sampler");
        mutex_acquisitions += worker_acquisitions;
        mutex_wait_total += worker_wait;
        total += worker_total;
    }
    SerializedMutexWaitSample {
        call_count: thread_count * SERIALIZED_WAIT_SAMPLES_PER_THREAD,
        mutex_acquisitions,
        mutex_wait_total,
        total,
    }
}

fn exclusive_wait_site_count<'a>(sources: impl IntoIterator<Item = &'a str>) -> usize {
    const EXCLUSIVE_WAIT_TOKENS: [&str; 7] = [
        "Mutex",
        "RwLock",
        "Condvar",
        "parking_lot",
        "spin_loop",
        "yield_now",
        ".park(",
    ];
    sources
        .into_iter()
        .map(|source| {
            EXCLUSIVE_WAIT_TOKENS
                .iter()
                .map(|token| source.matches(token).count())
                .sum::<usize>()
        })
        .sum()
}

fn assert_provider_matches_generation_window(
    provider_value: i32,
    generation_before: u32,
    generation_after: u32,
) {
    let lower_generation = generation_before.min(generation_after);
    let upper_generation = generation_before.max(generation_after);
    let lower_value = lower_generation / 2 + 1;
    let upper_value = upper_generation / 2 + 1;
    assert!(
        (lower_value..=upper_value).contains(&(provider_value as u32)),
        "provider payload {provider_value} was outside observed generation window {generation_before}..={generation_after}"
    );
    if generation_before == generation_after {
        assert_eq!(generation_before % 2, 0);
        assert_eq!(provider_value as u32, generation_before / 2 + 1);
    }
}

fn finalized_import(
    temperature: i32,
) -> (
    BridgeImport<dyn WeatherQueryInterface>,
    crate::plugin::FrozenBridgeTable,
    InterfaceSlot,
) {
    let mut registry = RuntimeExtensionRegistry::default();
    let consumer = registry.intern_plugin_module("climate.runtime").unwrap();
    let provider_owner = registry.intern_plugin_module("weather.runtime").unwrap();
    let bridge = registry
        .import_interface::<dyn WeatherQueryInterface>(consumer)
        .unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            provider_owner,
            Arc::new(WeatherQueryProvider { temperature }),
        )
        .unwrap();
    registry.finalize_bridge_imports();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    (bridge, table, slot)
}

fn finalized_drop_tracked_import(
    temperature: i32,
    drops: Arc<AtomicUsize>,
) -> (
    BridgeImport<dyn WeatherQueryInterface>,
    crate::plugin::FrozenBridgeTable,
    InterfaceSlot,
) {
    let mut registry = RuntimeExtensionRegistry::default();
    let consumer = registry.intern_plugin_module("climate.runtime").unwrap();
    let provider_owner = registry.intern_plugin_module("weather.runtime").unwrap();
    let bridge = registry
        .import_interface::<dyn WeatherQueryInterface>(consumer)
        .unwrap();
    registry
        .export_interface::<dyn WeatherQueryInterface>(
            provider_owner,
            Arc::new(DropTrackedWeatherProvider { temperature, drops }),
        )
        .unwrap();
    registry.finalize_bridge_imports();
    let table = registry.frozen_bridge_table();
    let slot = table
        .resolve_slot(<dyn WeatherQueryInterface as PluginInterface>::INTERFACE_ID)
        .unwrap();
    drop(registry);
    (bridge, table, slot)
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    let index = ((samples.len() - 1) * percentile).div_ceil(100);
    samples[index]
}

struct DropTrackedWeatherProvider {
    temperature: i32,
    drops: Arc<AtomicUsize>,
}

impl WeatherQueryInterface for DropTrackedWeatherProvider {
    fn sample_temperature(&self) -> i32 {
        self.temperature
    }
}

impl Drop for DropTrackedWeatherProvider {
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}
