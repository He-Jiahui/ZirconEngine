use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const LABELS_PER_SAMPLE: usize = 8_192;

#[test]
fn registry_label_hotpath_contract_single_buffer_gltf_uris() {
    let root = AssetUri::parse("res://models/plugins07.glb").unwrap();

    assert_eq!(
        gltf_indexed_label_uri(&root, "Node", 42),
        gltf_label_uri(&root, "Node42")
    );
    assert_eq!(
        gltf_mesh_primitive_uri(&root, 7, 11),
        gltf_label_uri(&root, "Mesh7/Primitive11")
    );
    assert_eq!(
        gltf_indexed_label_reference(&root, "Material", 9).locator,
        gltf_label_uri(&root, "Material9")
    );
}

#[test]
#[ignore = "release performance gate"]
fn registry_label_hotpath_performance_release_single_buffer_gltf_uris() {
    let root = AssetUri::parse("res://models/plugins07-benchmark.glb").unwrap();
    for _ in 0..4 {
        black_box(measure_legacy(&root));
        black_box(measure_single_buffer(&root));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        let (legacy_ns, optimized_ns) = if pair_index % 2 == 0 {
            (measure_legacy(&root), measure_single_buffer(&root))
        } else {
            let optimized_ns = measure_single_buffer(&root);
            (measure_legacy(&root), optimized_ns)
        };
        legacy_samples.push(legacy_ns);
        optimized_samples.push(optimized_ns);
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "PERF_RESULT plugins07_single_buffer_gltf_label_uris sample_pairs={SAMPLE_PAIRS} labels_per_sample={LABELS_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=20 legacy_uri_input_allocations_per_sample={} optimized_uri_input_allocations_per_sample={LABELS_PER_SAMPLE} order=alternating_legacy_first_even legacy_first_pairs=11 optimized_first_pairs=10",
        csv(&legacy_samples),
        csv(&optimized_samples),
        LABELS_PER_SAMPLE * 2,
    );
    assert!(
        improvement_percent >= 20,
        "single-buffer glTF label URIs must improve P95 by at least 20%"
    );
}

fn measure_legacy(root: &AssetUri) -> u128 {
    let started = Instant::now();
    for index in 0..LABELS_PER_SAMPLE {
        let uri = if index % 2 == 0 {
            gltf_label_uri(black_box(root), &format!("Node{}", black_box(index)))
        } else {
            gltf_label_uri(
                black_box(root),
                &format!("Mesh{}/Primitive{}", black_box(index), black_box(index + 1)),
            )
        };
        black_box(uri);
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_single_buffer(root: &AssetUri) -> u128 {
    let started = Instant::now();
    for index in 0..LABELS_PER_SAMPLE {
        let uri = if index % 2 == 0 {
            gltf_indexed_label_uri(black_box(root), "Node", black_box(index))
        } else {
            gltf_mesh_primitive_uri(black_box(root), black_box(index), black_box(index + 1))
        };
        black_box(uri);
    }
    started.elapsed().as_nanos().max(1)
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
