use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::{active_morph_target_capacity, morph_payload_from_mesh_asset};
use crate::asset::{
    AssetUri, MESH_ATTRIBUTE_POSITION, MeshAsset, MeshAttributeValues, MeshIndices,
    MeshMorphTargetAsset,
};
use crate::core::framework::render::RenderMeshTopology;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 256;
const VERTICES_PER_MESH: usize = 64;
const TARGETS_PER_MESH: usize = 64;
const ACTIVE_TARGETS: usize = TARGETS_PER_MESH / 2 + 1;
const DELTA_ROWS_PER_ACTIVE_TARGET: usize = VERTICES_PER_MESH * 4;

#[test]
fn optimization_batch_20260826fn_runtime209_capacity_preserves_active_morph_payload() {
    let mesh = morph_mesh();
    let mut weights = vec![0.0; TARGETS_PER_MESH];
    for index in (0..TARGETS_PER_MESH).step_by(2) {
        weights[index] = 0.5;
    }
    let mut previous_weights = weights.clone();
    previous_weights[1] = 0.25;

    let payload = morph_payload_from_mesh_asset(&mesh, &weights, Some(&previous_weights))
        .expect("active and previous-only targets should produce a morph payload");

    assert_eq!(
        active_morph_target_capacity(TARGETS_PER_MESH, &weights, Some(&previous_weights)),
        ACTIVE_TARGETS
    );
    assert_eq!(payload.target_count as usize, ACTIVE_TARGETS);
    assert_eq!(
        payload.deltas.len(),
        ACTIVE_TARGETS * DELTA_ROWS_PER_ACTIVE_TARGET
    );
    assert!(payload.deltas.capacity() >= ACTIVE_TARGETS * DELTA_ROWS_PER_ACTIVE_TARGET);
    assert_eq!(payload.weights.len(), ACTIVE_TARGETS);
    assert!(payload.weights.capacity() >= ACTIVE_TARGETS);
    assert_eq!(payload.previous_weights.len(), ACTIVE_TARGETS);
    assert!(payload.previous_weights.capacity() >= ACTIVE_TARGETS);
    assert_eq!(payload.weights[1].value, 0.0);
    assert_eq!(payload.previous_weights[1].value, 0.25);
}

#[test]
fn optimization_batch_20260826fn_runtime209_morph_vectors_reserve_active_upper_bound() {
    let source = include_str!("../morph_payload_upload.rs");
    assert!(source.contains("fn active_morph_target_capacity("));
    assert!(source.contains("Vec::with_capacity(delta_capacity)"));
    assert_eq!(
        source
            .matches("Vec::with_capacity(active_target_capacity)")
            .count(),
        2
    );
    assert!(source.contains("MORPH_DELTA_ROWS_PER_VERTEX_TARGET"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fn_runtime209_active_morph_payload_capacity_bench() {
    const DELTAS_PER_BUILD: usize = ACTIVE_TARGETS * DELTA_ROWS_PER_ACTIVE_TARGET;
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, DELTAS_PER_BUILD));
            optimized_samples.push(measure(true, DELTAS_PER_BUILD));
        } else {
            optimized_samples.push(measure(true, DELTAS_PER_BUILD));
            legacy_samples.push(measure(false, DELTAS_PER_BUILD));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME209_ACTIVE_MORPH_PAYLOAD_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} vertices_per_mesh={VERTICES_PER_MESH} \
active_targets={ACTIVE_TARGETS} deltas_per_build={DELTAS_PER_BUILD} \
legacy_reservations_per_vector=0 optimized_reservations_per_vector=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn morph_mesh() -> MeshAsset {
    let mut mesh = MeshAsset::new(
        AssetUri::parse("res://meshes/morph-capacity.zmesh").expect("fixture URI"),
        RenderMeshTopology::TriangleList,
        BTreeMap::from([(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.0]; VERTICES_PER_MESH]),
        )]),
        Some(MeshIndices::U32(vec![0, 1, 2])),
    )
    .expect("fixture mesh");
    mesh.morph_targets = (0..TARGETS_PER_MESH)
        .map(|index| MeshMorphTargetAsset {
            name: Some(format!("Target {index}")),
            attributes: BTreeMap::from([(
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![[index as f32, 0.0, 0.0]; VERTICES_PER_MESH]),
            )]),
        })
        .collect();
    mesh
}

fn measure(reserve: bool, delta_count: usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut deltas = if reserve {
            Vec::with_capacity(delta_count)
        } else {
            Vec::new()
        };
        let mut weights = if reserve {
            Vec::with_capacity(ACTIVE_TARGETS)
        } else {
            Vec::new()
        };
        let mut previous = if reserve {
            Vec::with_capacity(ACTIVE_TARGETS)
        } else {
            Vec::new()
        };
        for delta in 0..delta_count {
            deltas.push(black_box([delta as f32; 4]));
        }
        for target in 0..ACTIVE_TARGETS {
            weights.push(black_box(target as f32));
            previous.push(black_box(target as f32));
        }
        checksum ^= black_box(
            deltas.len()
                ^ deltas.capacity()
                ^ weights.len()
                ^ weights.capacity()
                ^ previous.len()
                ^ previous.capacity(),
        );
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
