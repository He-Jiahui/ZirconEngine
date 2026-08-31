use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 31;
const FORWARDS_PER_SAMPLE: usize = 200_000;

#[test]
fn optimization_batch_20260829af_runtime305_direct_forwarding_preserves_order_and_values() {
    let registrations = [3usize, 5, 8, 13, 21, 34, 55, 89];
    assert_eq!(optimized_forward(&registrations), 228);
    assert_eq!(
        optimized_forward(&registrations),
        legacy_forward(&registrations)
    );
    assert_eq!(optimized_forward(&[]), legacy_forward(&[]));
}

#[test]
fn optimization_batch_20260829af_runtime305_profile_wrappers_forward_without_outer_vecs() {
    let source = include_str!("../profile_modules.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let plugin_wrapper = implementation
        .split("fn runtime_modules_for_profile_descriptor_manifest_with_plugin_registration_reports")
        .nth(1)
        .expect("plugin registration wrapper")
        .split("pub(super) fn runtime_modules_for_runtime_profile_with_plugin_and_feature_registration_reports")
        .next()
        .expect("plugin wrapper body");
    let feature_wrapper = implementation
        .split("fn runtime_modules_for_profile_descriptor_manifest_with_plugin_and_feature_registration_reports")
        .nth(1)
        .expect("feature registration wrapper");

    assert!(!plugin_wrapper.contains("collect::<Vec<_>>"));
    assert!(!plugin_wrapper.contains("registrations.iter().copied()"));
    assert!(plugin_wrapper.contains("registrations,"));
    assert!(!feature_wrapper.contains("collect::<Vec<_>>"));
    assert!(!feature_wrapper.contains("feature_registrations.iter().copied()"));
    assert!(feature_wrapper.contains("registrations,"));
    assert!(feature_wrapper.contains("feature_registrations,"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829af_runtime305_direct_profile_registration_forwarding_bench() {
    let registrations = [3usize, 5, 8, 13, 21, 34, 55, 89];
    assert_eq!(
        optimized_forward(&registrations),
        legacy_forward(&registrations)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &registrations));
            optimized_samples.push(measure(true, &registrations));
        } else {
            optimized_samples.push(measure(true, &registrations));
            legacy_samples.push(measure(false, &registrations));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME305_DIRECT_PROFILE_REGISTRATION_FORWARDING_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
forwards_per_sample={FORWARDS_PER_SAMPLE} registrations_per_forward={} \
legacy_outer_vec_allocations_per_forward=1 optimized_outer_vec_allocations_per_forward=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        registrations.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn consume<'a>(registrations: impl IntoIterator<Item = &'a usize>) -> usize {
    registrations.into_iter().copied().sum()
}

fn legacy_forward(registrations: &[usize]) -> usize {
    let registrations = registrations.iter().collect::<Vec<_>>();
    consume(registrations.iter().copied())
}

fn optimized_forward(registrations: &[usize]) -> usize {
    consume(registrations.iter())
}

fn measure(optimized: bool, registrations: &[usize]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..FORWARDS_PER_SAMPLE {
        checksum = checksum.wrapping_add(if optimized {
            optimized_forward(black_box(registrations))
        } else {
            legacy_forward(black_box(registrations))
        });
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
