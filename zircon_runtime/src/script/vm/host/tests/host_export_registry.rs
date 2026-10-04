use std::hint::black_box;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Instant;

use crate::core::framework::script::{ScriptHostFunctionDescriptor, ScriptHostValueKind};

use super::*;

const RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_MODULES: usize = 256;
const RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_FUNCTIONS_PER_MODULE: usize = 8;
const RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_WARMUP_PAIRS: usize = 4;
const RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_SAMPLE_PAIRS: usize = 21;

#[test]
fn host_export_registry_accessors_recover_poisoned_module_lock() {
    let registry = HostExportRegistry::default();

    let poison_result = catch_unwind(AssertUnwindSafe(|| {
        let _guard = registry.state.lock().unwrap();
        panic!("poison host export registry");
    }));
    assert!(poison_result.is_err());

    let descriptor = ScriptHostModuleDescriptor::new("test.host", "1").with_function(
        ScriptHostFunctionDescriptor::new("ping", 0, 0, ScriptHostValueKind::Null),
    );
    let handle = registry
        .register_module(
            descriptor,
            [HostExportFunction::new("ping", |_| {
                Ok(ScriptHostValue::Null)
            })],
        )
        .unwrap();

    assert!(registry.module("test.host").is_some());
    assert_eq!(registry.modules()[0].handle, handle);
    assert_eq!(
        registry.call("test.host", "ping", Vec::new()).unwrap(),
        ScriptHostValue::Null
    );
    let call_table = registry.script_call_table();
    let call_site = call_table.resolve("test.host", "ping").unwrap();
    let arguments = Vec::new();
    let source = ScriptHostOwnedArgumentSource::new(&arguments);
    assert_eq!(
        call_site
            .call(
                crate::core::framework::script::ScriptHostArguments::new(&source),
                &CapabilitySet::default(),
            )
            .unwrap(),
        ScriptHostValue::Null
    );
}

#[test]
fn runtime24_lazy_host_export_call_table_defers_rebuild_until_read() {
    let registry = HostExportRegistry::default();
    for module_name in ["test.first", "test.second"] {
        registry
            .register_module(
                ScriptHostModuleDescriptor::new(module_name, "1").with_function(
                    ScriptHostFunctionDescriptor::new("ping", 0, 0, ScriptHostValueKind::Null),
                ),
                [HostExportFunction::new("ping", |_| {
                    Ok(ScriptHostValue::Null)
                })],
            )
            .unwrap();
    }

    {
        let state = registry.lock_state();
        assert_eq!(state.generation, 2);
        assert_eq!(state.call_table.generation(), 0);
        assert!(state.call_table.is_empty());
    }

    let table = registry.script_call_table();
    assert_eq!(table.generation(), 2);
    assert_eq!(table.len(), 2);
    assert!(table.resolve("test.first", "ping").is_some());
    assert!(table.resolve("test.second", "ping").is_some());

    let state = registry.lock_state();
    assert_eq!(state.call_table.generation(), 2);
    assert_eq!(state.call_table.len(), 2);
}

fn runtime24_register_benchmark_module(registry: &HostExportRegistry, module_index: usize) {
    let module_name = format!("benchmark.module.{module_index:04}");
    let mut descriptor = ScriptHostModuleDescriptor::new(module_name, "1");
    let mut callbacks =
        Vec::with_capacity(RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_FUNCTIONS_PER_MODULE);
    for function_index in 0..RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_FUNCTIONS_PER_MODULE {
        let function_name = format!("function.{function_index:02}");
        descriptor = descriptor.with_function(ScriptHostFunctionDescriptor::new(
            function_name.clone(),
            0,
            0,
            ScriptHostValueKind::Null,
        ));
        callbacks.push(HostExportFunction::new(function_name, |_| {
            Ok(ScriptHostValue::Null)
        }));
    }
    registry.register_module(descriptor, callbacks).unwrap();
}

fn runtime24_time_registration_batch(rebuild_after_every_module: bool) -> (u128, u64) {
    let started = Instant::now();
    let registry = HostExportRegistry::default();
    let mut eagerly_rebuilt_table = None;
    for module_index in 0..RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_MODULES {
        runtime24_register_benchmark_module(&registry, module_index);
        if rebuild_after_every_module {
            eagerly_rebuilt_table = Some(black_box(registry.script_call_table()));
        }
    }
    let table = eagerly_rebuilt_table.unwrap_or_else(|| black_box(registry.script_call_table()));
    let elapsed_ns = started.elapsed().as_nanos();
    let checksum = (table.len() as u64)
        .wrapping_mul(31)
        .wrapping_add(table.generation());
    (elapsed_ns, checksum)
}

fn runtime24_nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (percentile * sorted.len()).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

#[test]
#[ignore = "performance acceptance; run explicitly in release mode"]
fn runtime24_lazy_host_export_call_table_performance_acceptance() {
    for pair_index in 0..RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_WARMUP_PAIRS {
        let (legacy_checksum, optimized_checksum) = if pair_index % 2 == 0 {
            let (_, legacy_checksum) = runtime24_time_registration_batch(true);
            let (_, optimized_checksum) = runtime24_time_registration_batch(false);
            (legacy_checksum, optimized_checksum)
        } else {
            let (_, optimized_checksum) = runtime24_time_registration_batch(false);
            let (_, legacy_checksum) = runtime24_time_registration_batch(true);
            (legacy_checksum, optimized_checksum)
        };
        assert_eq!(legacy_checksum, optimized_checksum);
    }

    let mut legacy_samples_ns = [0_u128; RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_SAMPLE_PAIRS];
    let mut optimized_samples_ns = [0_u128; RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_SAMPLE_PAIRS];
    let mut legacy_checksum = 0;
    let mut optimized_checksum = 0;
    for sample_index in 0..RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            (legacy_samples_ns[sample_index], legacy_checksum) =
                runtime24_time_registration_batch(true);
            (optimized_samples_ns[sample_index], optimized_checksum) =
                runtime24_time_registration_batch(false);
        } else {
            (optimized_samples_ns[sample_index], optimized_checksum) =
                runtime24_time_registration_batch(false);
            (legacy_samples_ns[sample_index], legacy_checksum) =
                runtime24_time_registration_batch(true);
        }
        assert_eq!(legacy_checksum, optimized_checksum);
    }

    let legacy_p50_ns = runtime24_nearest_rank(&legacy_samples_ns, 50);
    let legacy_p95_ns = runtime24_nearest_rank(&legacy_samples_ns, 95);
    let optimized_p50_ns = runtime24_nearest_rank(&optimized_samples_ns, 50);
    let optimized_p95_ns = runtime24_nearest_rank(&optimized_samples_ns, 95);
    let legacy_copy_count = RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_FUNCTIONS_PER_MODULE
        * RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_MODULES
        * (RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_MODULES + 1)
        / 2;
    let optimized_copy_count = RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_FUNCTIONS_PER_MODULE
        * RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_MODULES;

    println!(
        "RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_PERF modules={} functions_per_module={} \
             legacy_copy_count={} optimized_copy_count={} legacy_samples_ns={:?} \
             optimized_samples_ns={:?} legacy_p50_ns={} optimized_p50_ns={} legacy_p95_ns={} \
             optimized_p95_ns={} checksum={}",
        RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_MODULES,
        RUNTIME24_LAZY_HOST_EXPORT_CALL_TABLE_FUNCTIONS_PER_MODULE,
        legacy_copy_count,
        optimized_copy_count,
        legacy_samples_ns,
        optimized_samples_ns,
        legacy_p50_ns,
        optimized_p50_ns,
        legacy_p95_ns,
        optimized_p95_ns,
        optimized_checksum,
    );

    assert_eq!(legacy_checksum, optimized_checksum);
    assert!(optimized_copy_count.saturating_mul(100) <= legacy_copy_count);
    assert!(optimized_p50_ns.saturating_mul(10) <= legacy_p50_ns);
    assert!(optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns);
}
