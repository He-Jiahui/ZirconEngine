use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::framework::render::PostProcessEffectSettings;

const RESOURCE_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_iu_runtime631_preserves_duplicate_output_error() {
    let stack = PostProcessStackDescriptor {
        initial_resources: Vec::new(),
        effects: vec![
            PostProcessEffectSettings::new(PostProcessEffectKind::Bloom)
                .with_produced_outputs(["shared-output"]),
            PostProcessEffectSettings::new(PostProcessEffectKind::Fxaa)
                .with_produced_outputs(["shared-output"]),
        ],
    };

    assert!(matches!(
        PostProcessPassGraph::validate_stack(&stack),
        Err(PostProcessGraphValidationError::DuplicateOutputResource { resource, .. })
            if resource == "shared-output"
    ));
}

#[test]
fn optimization_batch_iu_runtime631_hashes_resource_membership() {
    let source = include_str!("../../pass_graph.rs");
    let validate_body = source
        .split("pub fn validate_stack(")
        .nth(1)
        .expect("post-process stack validation remains present")
        .split("pub fn validate_stack_for_view_family")
        .next()
        .expect("post-process stack validation remains bounded");

    assert!(validate_body.contains("collect::<HashSet<_>>()"));
    assert!(validate_body.contains("HashSet::with_capacity("));
    assert!(validate_body.contains("node.produced_outputs.len()"));
    assert!(!validate_body.contains("let mut produced = BTreeSet::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_iu_runtime631_hash_resource_membership_benchmark() {
    let resources = (0..RESOURCE_COUNT)
        .map(|index| format!("post-process-resource-{index:08}"))
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_membership(&resources, false));
        black_box(measure_membership(&resources, true));
    }

    let mut ordered_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut hash_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            ordered_samples.push(measure_membership(&resources, false));
            hash_samples.push(measure_membership(&resources, true));
        } else {
            hash_samples.push(measure_membership(&resources, true));
            ordered_samples.push(measure_membership(&resources, false));
        }
    }

    let ordered_p95 = percentile(&ordered_samples, 95);
    let hash_p95 = percentile(&hash_samples, 95);
    let improvement_percent =
        ordered_p95.saturating_sub(hash_p95).saturating_mul(100) / ordered_p95.max(1);
    println!(
        "RUNTIME631_HASH_POST_PROCESS_RESOURCE_MEMBERSHIP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} resource_count={RESOURCE_COUNT} ordered_ns={} hash_ns={} ordered_p95_ns={ordered_p95} hash_p95_ns={hash_p95} improvement_percent={improvement_percent} threshold_percent=60",
        csv(&ordered_samples),
        csv(&hash_samples),
    );
    assert!(hash_p95 <= ordered_p95 * 40 / 100);
}

fn measure_membership(resources: &[String], hash: bool) -> u128 {
    let started = Instant::now();
    if hash {
        let mut seen = HashSet::with_capacity(resources.len());
        for resource in resources {
            black_box(seen.insert(black_box(resource.as_str())));
        }
        black_box(seen);
    } else {
        let mut seen = BTreeSet::new();
        for resource in resources {
            black_box(seen.insert(black_box(resource.as_str())));
        }
        black_box(seen);
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
