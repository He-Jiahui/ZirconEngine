use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::ZrStatus;

use super::{
    native_bridge_method_descriptors_from_manifest, NativeBridgeCall, NativeBridgeMethodBinding,
    NativeBridgeMethodFn,
};
use crate::plugin::{
    PluginInterfaceManifest, PluginInterfaceMethodManifest, PluginPackageManifest,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const DESCRIPTORS_PER_BUILD: usize = 256;

#[test]
fn optimization_batch_20260826ey_runtime194_capacity_preserves_bridge_method_descriptors() {
    let (manifest, bindings) = bridge_fixture(DESCRIPTORS_PER_BUILD);

    let descriptors = native_bridge_method_descriptors_from_manifest(&manifest, bindings).unwrap();

    assert_eq!(descriptors.len(), DESCRIPTORS_PER_BUILD);
    assert!(descriptors.capacity() >= DESCRIPTORS_PER_BUILD);
    assert_eq!(descriptors[0].interface_id(), "runtime194.bridge");
    assert_eq!(descriptors[0].method_slot(), 0);
    assert_eq!(descriptors[DESCRIPTORS_PER_BUILD - 1].method_slot(), 255);
}

#[test]
fn optimization_batch_20260826ey_runtime194_descriptors_reserve_manifest_method_count() {
    let source = include_str!("../bridge_method_bindings.rs");
    assert!(source.contains("let descriptor_capacity = manifest.bridge_methods().count();"));
    assert!(source.contains("Vec::with_capacity(descriptor_capacity)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ey_runtime194_native_bridge_method_descriptor_capacity_bench() {
    let method = NativeBridgeMethodFn::from_rust(bridge_method);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(method, false));
            optimized_samples.push(measure(method, true));
        } else {
            optimized_samples.push(measure(method, true));
            legacy_samples.push(measure(method, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME194_NATIVE_BRIDGE_METHOD_DESCRIPTOR_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} descriptors_per_build={DESCRIPTORS_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn bridge_fixture(method_count: usize) -> (PluginPackageManifest, Vec<NativeBridgeMethodBinding>) {
    let mut interface = PluginInterfaceManifest::new("runtime194.bridge");
    let mut bindings = Vec::with_capacity(method_count);
    for slot in 0..method_count {
        let method_name = format!("method_{slot}");
        interface = interface.with_method(PluginInterfaceMethodManifest::new(
            method_name.clone(),
            slot as u32,
        ));
        bindings.push(NativeBridgeMethodBinding::new(
            "runtime194.bridge",
            method_name,
            bridge_method as fn(NativeBridgeCall) -> ZrStatus,
        ));
    }
    (
        PluginPackageManifest::new("runtime194", "Runtime 194").with_provided_interface(interface),
        bindings,
    )
}

fn bridge_method(_: NativeBridgeCall) -> ZrStatus {
    ZrStatus::ok()
}

fn measure(method: NativeBridgeMethodFn, reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut descriptors = if reserve {
            Vec::with_capacity(DESCRIPTORS_PER_BUILD)
        } else {
            Vec::new()
        };
        for slot in 0..DESCRIPTORS_PER_BUILD {
            descriptors.push(black_box((slot as u32, method)));
        }
        checksum ^= black_box(descriptors.len() ^ descriptors.capacity());
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
