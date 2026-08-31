use crate::asset::MeshVertex;
use crate::core::math::Vec3;

use super::{mesh_sdf_source_hash, MeshSdfAsset, MeshSdfValidationError, MESH_SDF_SCHEMA_VERSION};

const MIN_MESH_SDF_DIMENSION: u32 = 4;
const MAX_MESH_SDF_DIMENSION: u32 = 256;

pub(super) fn validate_mesh_sdf_asset(asset: &MeshSdfAsset) -> Result<(), MeshSdfValidationError> {
    if asset.schema_version != MESH_SDF_SCHEMA_VERSION {
        return Err(MeshSdfValidationError::UnsupportedSchema {
            expected: MESH_SDF_SCHEMA_VERSION,
            actual: asset.schema_version,
        });
    }
    let settings = asset.cook_settings;
    if settings.max_dimension < MIN_MESH_SDF_DIMENSION
        || settings.max_dimension > MAX_MESH_SDF_DIMENSION
        || settings.max_voxel_count == 0
        || settings.max_payload_bytes == 0
        || settings.surface_band_voxels == 0
    {
        return Err(MeshSdfValidationError::InvalidCookSettings);
    }
    let expected = bounded_voxel_count(asset.dimensions, settings.max_dimension)?;
    let actual = u64::try_from(asset.voxels.len())
        .map_err(|_| MeshSdfValidationError::VoxelCountOverflow)?;
    if actual != expected {
        return Err(MeshSdfValidationError::VoxelCountMismatch { expected, actual });
    }
    if expected > settings.max_voxel_count {
        return Err(MeshSdfValidationError::InvalidDimensions);
    }
    let bounds_min = Vec3::from_array(asset.local_bounds.min);
    let bounds_max = Vec3::from_array(asset.local_bounds.max);
    if !bounds_min.is_finite() || !bounds_max.is_finite() || bounds_max.cmple(bounds_min).any() {
        return Err(MeshSdfValidationError::InvalidLocalBounds);
    }
    let voxel_size = Vec3::from_array(asset.voxel_size);
    if !voxel_size.is_finite() || voxel_size.cmple(Vec3::ZERO).any() {
        return Err(MeshSdfValidationError::InvalidVoxelSize);
    }
    let [distance_min, distance_max] = asset.distance_range;
    if !distance_min.is_finite()
        || !distance_max.is_finite()
        || distance_min >= 0.0
        || distance_max <= 0.0
        || (distance_min.abs() - distance_max).abs() > f32::EPSILON * distance_max.max(1.0)
    {
        return Err(MeshSdfValidationError::InvalidDistanceRange);
    }
    if asset.source_hash.iter().all(|byte| *byte == 0) {
        return Err(MeshSdfValidationError::MissingSourceHash);
    }
    let payload_bytes = asset
        .encoded_size_bytes()
        .ok_or(MeshSdfValidationError::PayloadSizeOverflow)?;
    if payload_bytes > settings.max_payload_bytes {
        return Err(MeshSdfValidationError::PayloadBudgetExceeded {
            actual: payload_bytes,
            budget: settings.max_payload_bytes,
        });
    }
    Ok(())
}

fn bounded_voxel_count(
    dimensions: [u32; 3],
    max_dimension: u32,
) -> Result<u64, MeshSdfValidationError> {
    debug_assert!(max_dimension <= MAX_MESH_SDF_DIMENSION);
    let mut voxel_count = 1_u64;
    for dimension in dimensions {
        if !(MIN_MESH_SDF_DIMENSION..=max_dimension).contains(&dimension) {
            return Err(MeshSdfValidationError::InvalidDimensions);
        }
        voxel_count *= u64::from(dimension);
    }
    Ok(voxel_count)
}

pub(super) fn validate_mesh_sdf_asset_for_source(
    asset: &MeshSdfAsset,
    vertices: &[MeshVertex],
    indices: &[u32],
) -> Result<(), MeshSdfValidationError> {
    validate_mesh_sdf_asset(asset)?;
    if asset.source_hash != mesh_sdf_source_hash(vertices, indices, asset.cook_settings) {
        return Err(MeshSdfValidationError::SourceHashMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod optimization_batch_gv_runtime577_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::*;

    const SAMPLE_PAIRS: usize = 31;
    const ITERATIONS: usize = 2_000_000;

    #[test]
    fn optimization_batch_gv_runtime577_dimension_fold_preserves_bounds_and_count() {
        assert_eq!(bounded_voxel_count([4, 8, 16], 32), Ok(512));
        assert_eq!(
            bounded_voxel_count([3, 8, 16], 32),
            Err(MeshSdfValidationError::InvalidDimensions)
        );
        assert_eq!(
            bounded_voxel_count([4, 8, 33], 32),
            Err(MeshSdfValidationError::InvalidDimensions)
        );

        let production = include_str!("validate.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production implementation");
        assert!(production.contains("bounded_voxel_count(asset.dimensions"));
        assert!(!production.contains(".voxel_count()"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_gv_runtime577_mesh_sdf_dimension_fold_p95() {
        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy_samples.push(measure(false));
                optimized_samples.push(measure(true));
            } else {
                optimized_samples.push(measure(true));
                legacy_samples.push(measure(false));
            }
        }

        let legacy_p95_ns = p95(&mut legacy_samples);
        let optimized_p95_ns = p95(&mut optimized_samples);
        println!(
            "RUNTIME577_MESH_SDF_DIMENSION_FOLD_BENCH_V1 sample_pairs={SAMPLE_PAIRS} iterations={ITERATIONS} dimensions=128x64x32 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy_samples),
            csv(&optimized_samples),
        );
        assert!(
            optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(90),
            "expected single-pass dimension folding to lower p95 by at least 10%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
        );
    }

    fn measure(optimized: bool) -> u128 {
        let started = Instant::now();
        let mut checksum = 0_u64;
        for _ in 0..ITERATIONS {
            let dimensions = black_box([128_u32, 64, 32]);
            let count = if optimized {
                bounded_voxel_count(dimensions, 256).expect("valid dimensions")
            } else {
                legacy_voxel_count(dimensions, 256).expect("valid dimensions")
            };
            checksum ^= count;
        }
        black_box(checksum);
        started.elapsed().as_nanos()
    }

    fn legacy_voxel_count(dimensions: [u32; 3], max_dimension: u32) -> Option<u64> {
        if dimensions
            .into_iter()
            .any(|dimension| !(MIN_MESH_SDF_DIMENSION..=max_dimension).contains(&dimension))
        {
            return None;
        }
        dimensions
            .into_iter()
            .map(u64::from)
            .try_fold(1_u64, u64::checked_mul)
    }

    fn p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        samples[samples.len() * 95 / 100]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
