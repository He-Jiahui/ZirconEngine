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
    if asset
        .dimensions
        .into_iter()
        .any(|dimension| !(MIN_MESH_SDF_DIMENSION..=settings.max_dimension).contains(&dimension))
    {
        return Err(MeshSdfValidationError::InvalidDimensions);
    }
    let expected = asset
        .voxel_count()
        .ok_or(MeshSdfValidationError::VoxelCountOverflow)?;
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
