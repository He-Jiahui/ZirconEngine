use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::{
    ShaderAbiBinding, ShaderNamedResourceBinding, ShaderResourceAccess,
    ShaderResourceBindingRequest, ShaderResourceDescriptor, ShaderResourceKind,
    validate_named_resource_bindings,
};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const RESOURCES_PER_BUILD: usize = 256;

#[test]
fn optimization_batch_20260826eq_runtime186_capacity_preserves_resource_abi_projection() {
    let declared = (0..RESOURCES_PER_BUILD)
        .map(|index| descriptor(format!("resource-{index}")))
        .collect::<Vec<_>>();
    let requested = declared
        .iter()
        .map(|resource| {
            (
                resource.name.clone(),
                ShaderResourceBindingRequest {
                    name: resource.name.clone(),
                    kind: resource.kind,
                    access: ShaderResourceAccess::Read,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();

    let bindings = validate_named_resource_bindings(&mut diagnostics, &requested, &declared, 2, 4);

    assert!(diagnostics.is_empty());
    assert_eq!(bindings.len(), RESOURCES_PER_BUILD);
    assert!(bindings.capacity() >= RESOURCES_PER_BUILD);
    assert_eq!(
        bindings[0].abi,
        ShaderAbiBinding {
            group: 2,
            binding: 4
        }
    );
    assert_eq!(bindings[255].abi.binding, 259);
}

#[test]
fn optimization_batch_20260826eq_runtime186_binding_output_uses_shared_input_upper_bound() {
    let source = include_str!("../compute_dispatch.rs");
    let validator_start = source.find("fn validate_named_resource_bindings").unwrap();
    let validator_end = source[validator_start..]
        .find("fn normalize_workgroup_size")
        .map(|offset| validator_start + offset)
        .unwrap();
    let validator_source = &source[validator_start..validator_end];

    assert!(validator_source.contains("Vec::with_capacity("));
    assert!(validator_source.contains("declared_resources.len().min(requested_bindings.len())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826eq_runtime186_shader_resource_binding_capacity_bench() {
    let binding = ShaderNamedResourceBinding {
        name: String::new(),
        kind: ShaderResourceKind::StorageBuffer,
        access: ShaderResourceAccess::Read,
        abi: ShaderAbiBinding {
            group: 0,
            binding: 1,
        },
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&binding, false));
            optimized_samples.push(measure(&binding, true));
        } else {
            optimized_samples.push(measure(&binding, true));
            legacy_samples.push(measure(&binding, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME186_SHADER_RESOURCE_BINDING_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} resources_per_build={RESOURCES_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn descriptor(name: String) -> ShaderResourceDescriptor {
    ShaderResourceDescriptor {
        name,
        kind: ShaderResourceKind::StorageBuffer,
        access: Some(ShaderResourceAccess::Read),
    }
}

fn measure(binding: &ShaderNamedResourceBinding, reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut output = if reserve {
            Vec::with_capacity(RESOURCES_PER_BUILD)
        } else {
            Vec::new()
        };
        for _ in 0..RESOURCES_PER_BUILD {
            output.push(black_box(binding.clone()));
        }
        checksum ^= black_box(output.len() ^ output.capacity());
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
