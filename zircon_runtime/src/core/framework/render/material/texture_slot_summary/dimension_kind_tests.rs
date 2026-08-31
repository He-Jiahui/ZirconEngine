use std::hint::black_box;
use std::time::Instant;

use super::*;

const KIND_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 21;
const KINDS: [&str; 16] = [
    "texture_2d_array",
    "Texture2DArray",
    "2D_ARRAY",
    "texture_cube_array",
    "TextureCubeArray",
    "CUBE_ARRAY",
    "texture_cube",
    "TextureCube",
    "CUBEMAP",
    "cube",
    "texture_3d",
    "Texture3D",
    "3D",
    "texture_1d",
    "1D",
    "texture_2d",
];

#[test]
fn optimization_batch_20260826cn_runtime131_dimension_kind_preserves_alias_contract() {
    let cases = [
        (
            " texture_2d_array ",
            RenderMaterialTextureDimension::D2Array,
        ),
        ("TEXTURE2DARRAY", RenderMaterialTextureDimension::D2Array),
        (
            "texture_cube_array",
            RenderMaterialTextureDimension::CubeArray,
        ),
        ("CubeMap", RenderMaterialTextureDimension::Cube),
        (" TEXTURE3D ", RenderMaterialTextureDimension::D3),
        ("Texture1D", RenderMaterialTextureDimension::D1),
        ("texture_2d", RenderMaterialTextureDimension::D2),
        ("unknown", RenderMaterialTextureDimension::D2),
    ];

    for (kind, expected) in cases {
        assert_eq!(
            RenderMaterialTextureDimension::from_shader_kind(kind),
            expected
        );
    }
}

#[test]
fn optimization_batch_20260826cn_runtime131_dimension_kind_avoids_lowercase_allocation() {
    let source = include_str!("../texture_slot_summary.rs")
        .split_once("#[cfg(test)]")
        .unwrap()
        .0;

    assert!(source.contains("eq_ignore_ascii_case"));
    assert!(!source.contains("to_ascii_lowercase"));
}

fn legacy_from_shader_kind(kind: &str) -> RenderMaterialTextureDimension {
    match kind.trim().to_ascii_lowercase().as_str() {
        "texture_2d_array" | "texture2darray" | "2d_array" => {
            RenderMaterialTextureDimension::D2Array
        }
        "texture_cube_array" | "texturecubearray" | "cube_array" => {
            RenderMaterialTextureDimension::CubeArray
        }
        "texture_cube" | "texturecube" | "cubemap" | "cube" => RenderMaterialTextureDimension::Cube,
        "texture_3d" | "texture3d" | "3d" => RenderMaterialTextureDimension::D3,
        "texture_1d" | "texture1d" | "1d" => RenderMaterialTextureDimension::D1,
        _ => RenderMaterialTextureDimension::D2,
    }
}

fn classify_batch(classify: fn(&str) -> RenderMaterialTextureDimension) -> usize {
    let mut checksum = 0usize;
    for index in 0..KIND_COUNT {
        let dimension = classify(black_box(KINDS[index % KINDS.len()]));
        checksum = checksum.wrapping_add(dimension as usize);
    }
    checksum
}

fn elapsed_ns(run: impl FnOnce() -> usize) -> u128 {
    let started = Instant::now();
    black_box(run());
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826cn_runtime131_dimension_kind_performance_evidence() {
    for _ in 0..3 {
        assert_eq!(
            black_box(classify_batch(legacy_from_shader_kind)),
            classify_batch(RenderMaterialTextureDimension::from_shader_kind)
        );
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(elapsed_ns(|| classify_batch(legacy_from_shader_kind)));
            optimized_samples.push(elapsed_ns(|| {
                classify_batch(RenderMaterialTextureDimension::from_shader_kind)
            }));
        } else {
            optimized_samples.push(elapsed_ns(|| {
                classify_batch(RenderMaterialTextureDimension::from_shader_kind)
            }));
            legacy_samples.push(elapsed_ns(|| classify_batch(legacy_from_shader_kind)));
        }
    }

    let legacy_p50_ns = nearest_rank(&mut legacy_samples.clone(), 50);
    let legacy_p95_ns = nearest_rank(&mut legacy_samples, 95);
    let optimized_p50_ns = nearest_rank(&mut optimized_samples.clone(), 50);
    let optimized_p95_ns = nearest_rank(&mut optimized_samples, 95);
    println!(
        "RUNTIME131_TEXTURE_DIMENSION_ZERO_ALLOCATION_MATCH_BENCH_V1 sample_pairs={} kind_count={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        SAMPLE_PAIRS,
        KIND_COUNT,
        legacy_p50_ns,
        legacy_p95_ns,
        optimized_p50_ns,
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "zero-allocation texture dimension matching p95 must be at least 30% below lowercase allocation: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
