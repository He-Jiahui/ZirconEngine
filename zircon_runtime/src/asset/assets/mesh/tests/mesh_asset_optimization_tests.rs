use std::hint::black_box;
use std::time::Instant;

use crate::core::math::Vec2;

use super::*;

const PERFORMANCE_VERTEX_COUNT: usize = 262_144;
const SAMPLE_PAIRS: usize = 17;

fn primitive(vertices: Vec<MeshVertex>) -> ModelPrimitiveAsset {
    ModelPrimitiveAsset {
        vertices,
        indices: vec![0, 1, 2],
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    }
}

fn vertex() -> MeshVertex {
    MeshVertex::new(Vec3::ZERO, Vec3::Z, Vec2::ZERO)
}

fn legacy_optional_uv1_values(vertices: &[MeshVertex]) -> Option<Vec<[f32; 2]>> {
    let uv1_values = vertices.iter().map(|vertex| vertex.uv1).collect::<Vec<_>>();
    uv1_values
        .iter()
        .any(|uv| *uv != DEFAULT_UV)
        .then_some(uv1_values)
}

fn elapsed_micros(run: impl FnOnce()) -> u128 {
    let started = Instant::now();
    run();
    started.elapsed().as_micros()
}

fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * 95).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
fn optimization_batch_20260826e_runtime93_optional_uv1_scans_before_materializing() {
    let source = include_str!("../mesh_asset.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(source.contains("fn optional_uv1_values("));
    assert!(source.contains(".any(|vertex| vertex.uv1 != DEFAULT_UV)"));
    assert!(source.contains("optional_uv1_values(&primitive.vertices)"));
    assert!(!source.contains("let uv1_values = primitive\n            .vertices"));
}

#[test]
fn optimization_batch_20260826e_runtime93_optional_uv1_preserves_presence_and_values() {
    let absent = MeshAsset::from_model_primitive(
        AssetUri::parse("res://mesh/no-uv1.zrmesh").unwrap(),
        &primitive(vec![vertex(), vertex(), vertex()]),
    );
    assert!(!absent.attributes.contains_key(MESH_ATTRIBUTE_UV1));

    let present_vertices = vec![vertex(), vertex().with_uv1(Vec2::new(0.25, 0.75)), vertex()];
    let present = MeshAsset::from_model_primitive(
        AssetUri::parse("res://mesh/with-uv1.zrmesh").unwrap(),
        &primitive(present_vertices.clone()),
    );
    let expected_uv1 = present_vertices
        .iter()
        .map(|vertex| vertex.uv1)
        .collect::<Vec<_>>();
    assert_eq!(
        present
            .attributes
            .get(MESH_ATTRIBUTE_UV1)
            .and_then(MeshAttributeValues::as_float32x2),
        Some(expected_uv1.as_slice())
    );
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826e_runtime93_optional_uv1_performance_evidence() {
    let vertices = vec![vertex(); PERFORMANCE_VERTEX_COUNT];
    assert!(legacy_optional_uv1_values(&vertices).is_none());
    assert!(optional_uv1_values(&vertices).is_none());

    for _ in 0..3 {
        black_box(legacy_optional_uv1_values(black_box(&vertices)));
        black_box(optional_uv1_values(black_box(&vertices)));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let measure_legacy = || {
            elapsed_micros(|| {
                black_box(legacy_optional_uv1_values(black_box(&vertices)));
            })
        };
        let measure_optimized = || {
            elapsed_micros(|| {
                black_box(optional_uv1_values(black_box(&vertices)));
            })
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95 = nearest_rank_p95(&mut legacy_samples);
    let optimized_p95 = nearest_rank_p95(&mut optimized_samples);
    println!(
        "RUNTIME93_OPTIONAL_UV1_LAZY_MATERIALIZATION_BENCH_V1 sample_pairs={} vertices={} legacy_temporary_uv1_allocations=1 optimized_temporary_uv1_allocations=0 legacy_copied_uv1_values={} optimized_copied_uv1_values=0 legacy_p95_us={} optimized_p95_us={} legacy_samples_us={:?} optimized_samples_us={:?}",
        SAMPLE_PAIRS,
        PERFORMANCE_VERTEX_COUNT,
        PERFORMANCE_VERTEX_COUNT,
        legacy_p95,
        optimized_p95,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(60),
        "borrowed UV1 presence scan p95 must be at least 40% below allocate-then-scan: legacy={legacy_p95}us optimized={optimized_p95}us"
    );
}
