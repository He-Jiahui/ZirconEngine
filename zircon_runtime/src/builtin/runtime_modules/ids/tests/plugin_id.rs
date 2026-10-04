use std::borrow::Cow;
use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;
use std::time::Instant;

use super::{normalize_runtime_plugin_key, RuntimePluginId, RuntimePluginIdStorage};

#[test]
fn runtime_plugin_id_accepts_external_keys_without_core_variant() {
    let id = RuntimePluginId::new("third_party.weather_sim");

    assert_eq!(id.key(), "third_party.weather_sim");
    assert_eq!(id.label(), "third_party.weather_sim");
    assert_eq!(
        RuntimePluginId::parse_key("Third_Party.Weather_Sim"),
        Some(id)
    );
}

#[test]
fn runtime_plugin_id_rejects_empty_or_non_key_text() {
    for raw in ["", " ", ".starts_with_dot", "bad id", "bad/id"] {
        assert_eq!(RuntimePluginId::parse_key(raw), None);
    }
}

#[test]
fn canonical_plugin_keys_borrow_the_input_during_normalization() {
    assert!(matches!(
        normalize_runtime_plugin_key("  third_party.weather_sim  "),
        Some(Cow::Borrowed("third_party.weather_sim"))
    ));
    assert!(matches!(
        normalize_runtime_plugin_key("Third_Party.Weather_Sim"),
        Some(Cow::Owned(value)) if value == "third_party.weather_sim"
    ));
}

#[test]
fn dynamic_plugin_ids_do_not_use_process_global_leaks() {
    let source = include_str!("../plugin_id.rs");

    assert!(!source.contains(concat!("Box", "::leak")));
    assert!(!source.contains(concat!("HashSet<&'static", " str>")));
    assert!(!source.contains(concat!("static INTERNED", "_KEYS")));
}

#[test]
fn dynamic_plugin_id_storage_retires_with_the_last_generation_owner() {
    let id = RuntimePluginId::new("third_party.weather_sim");
    let weak = match &id.0 {
        RuntimePluginIdStorage::Dynamic(key) => Arc::downgrade(key),
        RuntimePluginIdStorage::Static(_) => panic!("external plugin ID must be dynamic"),
    };
    let generation = vec![id.clone(); 64];

    drop(id);
    assert!(weak.upgrade().is_some());
    drop(generation);
    assert!(weak.upgrade().is_none());
}

#[test]
fn builtin_plugin_ids_keep_static_zero_allocation_storage() {
    assert!(matches!(
        &RuntimePluginId::Sound.0,
        RuntimePluginIdStorage::Static("sound")
    ));
}

#[test]
fn dynamic_plugin_id_serde_hash_and_order_use_the_canonical_key() {
    let canonical = RuntimePluginId::new("third_party.weather_sim");
    let normalized = RuntimePluginId::new("Third_Party.Weather_Sim");

    assert_eq!(canonical, normalized);
    assert_eq!(
        serde_json::to_string(&canonical).expect("plugin ID should serialize"),
        "\"third_party.weather_sim\""
    );
    assert_eq!(
        serde_json::from_str::<RuntimePluginId>("\"Third_Party.Weather_Sim\"")
            .expect("plugin ID should deserialize"),
        canonical
    );

    let mut hashed = HashSet::new();
    hashed.insert(canonical.clone());
    hashed.insert(normalized);
    assert_eq!(hashed.len(), 1);

    let ordered = BTreeSet::from([
        RuntimePluginId::new("third_party.zeta"),
        RuntimePluginId::Sound,
        RuntimePluginId::new("third_party.alpha"),
    ]);
    assert_eq!(
        ordered.iter().map(RuntimePluginId::key).collect::<Vec<_>>(),
        ["sound", "third_party.alpha", "third_party.zeta"]
    );
}

#[test]
#[ignore = "PERF-MVP-436 long-running ownership benchmark"]
fn runtime_plugin_id_generation_churn_benchmark() {
    for id_count in [1, 1_000, 1_000_000] {
        let rss_before = current_rss_bytes();
        let started = Instant::now();
        let generation = (0..id_count)
            .map(|index| RuntimePluginId::new(format!("bench.plugin_{index}")))
            .collect::<Vec<_>>();
        let elapsed = started.elapsed();
        let string_bytes = generation.iter().map(|id| id.key().len()).sum::<usize>();
        let last_owner = dynamic_storage_weak(
            generation
                .last()
                .expect("benchmark generation should contain an ID"),
        );
        let rss_active = current_rss_bytes();

        assert_eq!(generation.len(), id_count);
        drop(generation);
        assert!(last_owner.upgrade().is_none());
        let rss_retired = current_rss_bytes();
        eprintln!(
            "PERF-MVP-436 ids={id_count} elapsed_ns={} interner_locks=0 probes=0 active_entries={id_count} active_string_bytes={string_bytes} rss_before={rss_before:?} rss_active={rss_active:?} rss_retired={rss_retired:?}",
            elapsed.as_nanos()
        );
    }

    for reload_count in [1, 1_000, 100_000] {
        let rss_before = current_rss_bytes();
        let started = Instant::now();
        for generation in 0..reload_count {
            let id = RuntimePluginId::new(format!("reload.plugin_{generation}"));
            let owner = dynamic_storage_weak(&id);
            drop(id);
            assert!(owner.upgrade().is_none());
        }
        let rss_retired = current_rss_bytes();
        eprintln!(
            "PERF-MVP-436 reloads={reload_count} elapsed_ns={} interner_locks=0 probes=0 retained_dynamic_entries=0 retained_dynamic_string_bytes=0 rss_before={rss_before:?} rss_retired={rss_retired:?}",
            started.elapsed().as_nanos()
        );
    }

    for thread_count in [1, 64] {
        let started = Instant::now();
        let workers = (0..thread_count)
            .map(|thread_index| {
                std::thread::spawn(move || {
                    (0..1_000)
                        .map(|index| {
                            RuntimePluginId::new(format!("thread_{thread_index}.plugin_{index}"))
                                .key()
                                .len()
                        })
                        .sum::<usize>()
                })
            })
            .collect::<Vec<_>>();
        let checksum = workers
            .into_iter()
            .map(|worker| worker.join().expect("benchmark worker should finish"))
            .sum::<usize>();
        assert!(checksum > 0);
        eprintln!(
            "PERF-MVP-436 threads={thread_count} ids_per_thread=1000 elapsed_ns={} interner_locks=0 probes=0",
            started.elapsed().as_nanos()
        );
    }
}

fn dynamic_storage_weak(id: &RuntimePluginId) -> std::sync::Weak<str> {
    match &id.0 {
        RuntimePluginIdStorage::Dynamic(key) => Arc::downgrade(key),
        RuntimePluginIdStorage::Static(_) => panic!("benchmark ID must be dynamic"),
    }
}

#[cfg(windows)]
fn current_rss_bytes() -> Option<usize> {
    use std::ffi::c_void;
    use std::mem::{size_of, MaybeUninit};

    #[repr(C)]
    struct ProcessMemoryCounters {
        cb: u32,
        page_fault_count: u32,
        peak_working_set_size: usize,
        working_set_size: usize,
        quota_peak_paged_pool_usage: usize,
        quota_paged_pool_usage: usize,
        quota_peak_non_paged_pool_usage: usize,
        quota_non_paged_pool_usage: usize,
        pagefile_usage: usize,
        peak_pagefile_usage: usize,
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> *mut c_void;
    }

    #[link(name = "psapi")]
    unsafe extern "system" {
        fn GetProcessMemoryInfo(
            process: *mut c_void,
            counters: *mut ProcessMemoryCounters,
            size: u32,
        ) -> i32;
    }

    let mut counters = MaybeUninit::<ProcessMemoryCounters>::zeroed();
    let counters_ptr = counters.as_mut_ptr();
    // SAFETY: the zeroed structure has the layout required by PROCESS_MEMORY_COUNTERS,
    // and both pointers remain valid for the duration of the OS call.
    unsafe {
        (*counters_ptr).cb = size_of::<ProcessMemoryCounters>() as u32;
        if GetProcessMemoryInfo(
            GetCurrentProcess(),
            counters_ptr,
            size_of::<ProcessMemoryCounters>() as u32,
        ) == 0
        {
            return None;
        }
        Some(counters.assume_init().working_set_size)
    }
}

#[cfg(not(windows))]
fn current_rss_bytes() -> Option<usize> {
    None
}
