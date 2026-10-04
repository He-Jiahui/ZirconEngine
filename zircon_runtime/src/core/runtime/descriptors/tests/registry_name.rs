use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::RegistryName;
use crate::core::ServiceKind;

#[test]
fn registry_name_clones_share_value_storage() {
    let name =
        RegistryName::new("Runtime.Core.Manager.WindowManager").expect("valid registry name");
    let cloned = name.clone();

    assert!(Arc::ptr_eq(&name.value, &cloned.value));
    assert_eq!(cloned.as_str(), "Runtime.Core.Manager.WindowManager");
    assert_eq!(cloned.module_name(), "Runtime.Core");
    assert_eq!(cloned.service_kind(), ServiceKind::Manager);
    assert_eq!(cloned.service_name(), "WindowManager");
}

fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

fn legacy_clone_sample(names: &[String], clones_per_name: usize) -> (u128, usize) {
    let started = Instant::now();
    let checksum = {
        let mut clones = Vec::with_capacity(names.len() * clones_per_name);
        for name in names {
            for _ in 0..clones_per_name {
                clones.push(name.clone());
            }
        }
        let checksum = clones.iter().map(String::len).sum();
        black_box(&clones);
        checksum
    };
    (started.elapsed().as_nanos(), checksum)
}

fn shared_clone_sample(names: &[RegistryName], clones_per_name: usize) -> (u128, usize) {
    let started = Instant::now();
    let checksum = {
        let mut clones = Vec::with_capacity(names.len() * clones_per_name);
        for name in names {
            for _ in 0..clones_per_name {
                clones.push(name.clone());
            }
        }
        let checksum = clones.iter().map(|name| name.as_str().len()).sum();
        black_box(&clones);
        checksum
    };
    (started.elapsed().as_nanos(), checksum)
}

#[test]
#[ignore = "release-only registry-name clone performance evidence"]
fn registry_name_clone_release_benchmark_evidence() {
    const NAMES: usize = 65_536;
    const CLONES_PER_NAME: usize = 8;
    const SAMPLE_PAIRS: usize = 21;

    let legacy_names: Vec<String> = (0..NAMES)
        .map(|index| format!("Runtime.Feature{index}.Manager.Service{index}"))
        .collect();
    let shared_names: Vec<RegistryName> = legacy_names
        .iter()
        .cloned()
        .map(|name| RegistryName::new(name).expect("generated registry name is valid"))
        .collect();

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut shared_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let ((legacy_ns, legacy_checksum), (shared_ns, shared_checksum)) = if pair % 2 == 0 {
            (
                legacy_clone_sample(&legacy_names, CLONES_PER_NAME),
                shared_clone_sample(&shared_names, CLONES_PER_NAME),
            )
        } else {
            let shared = shared_clone_sample(&shared_names, CLONES_PER_NAME);
            let legacy = legacy_clone_sample(&legacy_names, CLONES_PER_NAME);
            (legacy, shared)
        };
        assert_eq!(legacy_checksum, shared_checksum);
        legacy_samples.push(legacy_ns);
        shared_samples.push(shared_ns);
    }

    let legacy_p50_ns = nearest_rank(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = nearest_rank(&mut legacy_samples, 95);
    let shared_p50_ns = nearest_rank(&mut shared_samples.clone(), 50);
    let shared_p95_ns = nearest_rank(&mut shared_samples, 95);
    println!(
        "RUNTIME01_REGISTRY_NAME_BENCH_V1 names={NAMES} clones_per_name={CLONES_PER_NAME} \
             sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
             shared_p50_ns={shared_p50_ns} legacy_p95_ns={legacy_p95_ns} \
             shared_p95_ns={shared_p95_ns} legacy_payload_allocations={} \
             shared_payload_allocations=0",
        NAMES * CLONES_PER_NAME
    );

    assert!(
        shared_p50_ns.saturating_mul(2) <= legacy_p50_ns,
        "shared clone P50 must be at least 50% faster: legacy={legacy_p50_ns}ns shared={shared_p50_ns}ns"
    );
    assert!(
        shared_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "shared clone P95 must be at least 50% faster: legacy={legacy_p95_ns}ns shared={shared_p95_ns}ns"
    );
}
