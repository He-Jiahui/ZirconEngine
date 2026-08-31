use crate::asset::assets::{mesh_sdf_source_hash, MESH_SDF_FIXED_METADATA_BYTES};
use crate::asset::{
    MeshSdfAsset, MeshSdfCookError, MeshSdfCookSettings, MeshSdfEncoding, MeshVertex,
    MESH_SDF_SCHEMA_VERSION,
};
use crate::core::framework::render::RenderMeshBounds;
use crate::core::framework::tasks::ParallelSliceExecutor;
use crate::core::math::Vec3;

use super::acceleration::{Aabb, TriangleBvh};
use super::budget::{MAX_MESH_SDF_PRIMITIVE_WORK_UNITS, MAX_MESH_SDF_SOURCE_TRIANGLE_COUNT};
use super::MeshSdfCookBudget;

const MIN_MESH_SDF_DIMENSION: u32 = 4;
const MAX_MESH_SDF_DIMENSION: u32 = 256;
const MESH_SDF_OBJECT_BORDER_VOXELS: u32 = 1;
const SNORM16_MAX: f32 = i16::MAX as f32;

pub fn cook_mesh_sdf_from_mesh(
    vertices: &[MeshVertex],
    indices: &[u32],
    settings: MeshSdfCookSettings,
) -> Result<MeshSdfAsset, MeshSdfCookError> {
    cook_mesh_sdf_from_mesh_with_budget(
        vertices,
        indices,
        settings,
        &mut MeshSdfCookBudget::default(),
    )
}

pub fn cook_mesh_sdf_from_mesh_with_budget(
    vertices: &[MeshVertex],
    indices: &[u32],
    settings: MeshSdfCookSettings,
    budget: &mut MeshSdfCookBudget,
) -> Result<MeshSdfAsset, MeshSdfCookError> {
    cook_mesh_sdf_from_mesh_with_budget_and_voxel_builder(
        vertices,
        indices,
        settings,
        budget,
        |bvh, layout, distance_limit, voxel_count| {
            (0..voxel_count)
                .map(|linear_index| cook_voxel(bvh, layout, settings, distance_limit, linear_index))
                .collect()
        },
    )
}

/// Cooks a Mesh SDF through a caller-owned executor.
///
/// Importers use this only when their transaction owns an explicit execution
/// capability. The default cook entry points remain serial rather than
/// falling back to Rayon's process-global pool.
pub fn cook_mesh_sdf_from_mesh_with_executor(
    executor: &impl ParallelSliceExecutor,
    vertices: &[MeshVertex],
    indices: &[u32],
    settings: MeshSdfCookSettings,
) -> Result<MeshSdfAsset, MeshSdfCookError> {
    cook_mesh_sdf_from_mesh_with_budget_and_executor(
        executor,
        vertices,
        indices,
        settings,
        &mut MeshSdfCookBudget::default(),
    )
}

pub fn cook_mesh_sdf_from_mesh_with_budget_and_executor(
    executor: &impl ParallelSliceExecutor,
    vertices: &[MeshVertex],
    indices: &[u32],
    settings: MeshSdfCookSettings,
    budget: &mut MeshSdfCookBudget,
) -> Result<MeshSdfAsset, MeshSdfCookError> {
    cook_mesh_sdf_from_mesh_with_budget_and_voxel_builder(
        vertices,
        indices,
        settings,
        budget,
        |bvh, layout, distance_limit, voxel_count| {
            executor.parallel_map_indices(voxel_count, |linear_index| {
                cook_voxel(bvh, layout, settings, distance_limit, linear_index)
            })
        },
    )
}

fn cook_mesh_sdf_from_mesh_with_budget_and_voxel_builder(
    vertices: &[MeshVertex],
    indices: &[u32],
    settings: MeshSdfCookSettings,
    budget: &mut MeshSdfCookBudget,
    build_voxels: impl FnOnce(&TriangleBvh, VolumeLayout, f32, usize) -> Vec<i16>,
) -> Result<MeshSdfAsset, MeshSdfCookError> {
    validate_settings(settings)?;
    if indices.len() % 3 != 0 {
        return Err(MeshSdfCookError::InvalidTriangleIndexCount);
    }
    let source_triangle_count = u64::try_from(indices.len() / 3).unwrap_or(u64::MAX);
    if source_triangle_count > MAX_MESH_SDF_SOURCE_TRIANGLE_COUNT {
        return Err(MeshSdfCookError::SourceTriangleBudgetExceeded {
            actual: source_triangle_count,
            budget: MAX_MESH_SDF_SOURCE_TRIANGLE_COUNT,
        });
    }
    let bvh = TriangleBvh::build(vertices, indices)?;
    let layout = choose_volume_layout(bvh.source_bounds(), settings)?;
    let distance_limit = layout.voxel_size * settings.surface_band_voxels as f32;
    let voxel_count = layout
        .voxel_count()
        .ok_or(MeshSdfCookError::VoxelCountOverflow)?;
    let voxel_count_u64 =
        u64::try_from(voxel_count).map_err(|_| MeshSdfCookError::VoxelCountOverflow)?;
    let triangle_count = u64::try_from(bvh.triangle_count()).unwrap_or(u64::MAX);
    let work_units = voxel_count_u64.saturating_mul(triangle_count);
    if work_units > MAX_MESH_SDF_PRIMITIVE_WORK_UNITS {
        return Err(MeshSdfCookError::PrimitiveWorkBudgetExceeded {
            actual: work_units,
            budget: MAX_MESH_SDF_PRIMITIVE_WORK_UNITS,
        });
    }
    let payload_bytes = voxel_count_u64
        .saturating_mul(std::mem::size_of::<i16>() as u64)
        .saturating_add(MESH_SDF_FIXED_METADATA_BYTES);
    budget.reserve(voxel_count_u64, payload_bytes, work_units)?;
    let voxels = build_voxels(&bvh, layout, distance_limit, voxel_count);

    let asset = MeshSdfAsset {
        schema_version: MESH_SDF_SCHEMA_VERSION,
        source_hash: mesh_sdf_source_hash(vertices, indices, settings),
        local_bounds: RenderMeshBounds::from_min_max(
            layout.bounds.min.to_array(),
            layout.bounds.max.to_array(),
        ),
        dimensions: layout.dimensions,
        voxel_size: [layout.voxel_size; 3],
        distance_range: [-distance_limit, distance_limit],
        encoding: MeshSdfEncoding::SignedNormalized16,
        cook_settings: settings,
        voxels,
    };
    asset.validate_for_source(vertices, indices)?;
    Ok(asset)
}

fn cook_voxel(
    bvh: &TriangleBvh,
    layout: VolumeLayout,
    settings: MeshSdfCookSettings,
    distance_limit: f32,
    linear_index: usize,
) -> i16 {
    let point = layout.voxel_center(linear_index);
    let unsigned_distance = bvh.nearest_distance_squared(point).sqrt();
    let signed_distance = if settings.two_sided
        || bvh.positive_x_intersection_count(jittered_sign_origin(
            point,
            linear_index,
            layout.voxel_size,
        )) % 2
            == 0
    {
        unsigned_distance
    } else {
        -unsigned_distance
    };
    encode_snorm16_distance(signed_distance, distance_limit)
}

pub fn cook_mesh_sdf_or_fallback(
    vertices: &[MeshVertex],
    indices: &[u32],
    settings: MeshSdfCookSettings,
    budget: &mut MeshSdfCookBudget,
) -> Result<Option<MeshSdfAsset>, MeshSdfCookError> {
    match cook_mesh_sdf_from_mesh_with_budget(vertices, indices, settings, budget) {
        Ok(asset) => Ok(Some(asset)),
        Err(error) if error.is_budget_exceeded() => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn cook_mesh_sdf_or_fallback_single(
    vertices: &[MeshVertex],
    indices: &[u32],
    settings: MeshSdfCookSettings,
) -> Result<Option<MeshSdfAsset>, MeshSdfCookError> {
    cook_mesh_sdf_or_fallback(
        vertices,
        indices,
        settings,
        &mut MeshSdfCookBudget::default(),
    )
}

#[derive(Clone, Copy, Debug)]
struct VolumeLayout {
    bounds: Aabb,
    dimensions: [u32; 3],
    voxel_size: f32,
}

impl VolumeLayout {
    fn voxel_count(self) -> Option<usize> {
        self.dimensions
            .into_iter()
            .map(|dimension| usize::try_from(dimension).ok())
            .try_fold(1_usize, |total, dimension| total.checked_mul(dimension?))
    }

    fn voxel_center(self, linear_index: usize) -> Vec3 {
        let width = self.dimensions[0] as usize;
        let height = self.dimensions[1] as usize;
        let x = linear_index % width;
        let yz = linear_index / width;
        let y = yz % height;
        let z = yz / height;
        self.bounds.min
            + Vec3::new(x as f32 + 0.5, y as f32 + 0.5, z as f32 + 0.5) * self.voxel_size
    }
}

fn validate_settings(settings: MeshSdfCookSettings) -> Result<(), MeshSdfCookError> {
    if !(MIN_MESH_SDF_DIMENSION..=MAX_MESH_SDF_DIMENSION).contains(&settings.max_dimension)
        || settings.max_voxel_count < u64::from(MIN_MESH_SDF_DIMENSION).pow(3)
        || settings.surface_band_voxels == 0
        || settings.max_payload_bytes <= MESH_SDF_FIXED_METADATA_BYTES
    {
        return Err(MeshSdfCookError::InvalidSettings);
    }
    Ok(())
}

fn choose_volume_layout(
    source_bounds: Aabb,
    settings: MeshSdfCookSettings,
) -> Result<VolumeLayout, MeshSdfCookError> {
    let payload_voxel_budget = settings
        .max_payload_bytes
        .saturating_sub(MESH_SDF_FIXED_METADATA_BYTES)
        / std::mem::size_of::<i16>() as u64;
    let voxel_budget = settings.max_voxel_count.min(payload_voxel_budget);
    if voxel_budget < u64::from(MIN_MESH_SDF_DIMENSION).pow(3) {
        return Err(MeshSdfCookError::PayloadBudgetTooSmall);
    }

    let mut lower = MIN_MESH_SDF_DIMENSION;
    let mut upper = settings.max_dimension;
    let mut selected = None;
    while lower <= upper {
        let candidate_dimension = lower + (upper - lower) / 2;
        let candidate = volume_layout_for_max_dimension(source_bounds, candidate_dimension)?;
        let candidate_voxels = candidate
            .voxel_count()
            .and_then(|count| u64::try_from(count).ok())
            .ok_or(MeshSdfCookError::VoxelCountOverflow)?;
        if candidate_voxels <= voxel_budget {
            selected = Some(candidate);
            lower = candidate_dimension.saturating_add(1);
        } else if candidate_dimension == MIN_MESH_SDF_DIMENSION {
            break;
        } else {
            upper = candidate_dimension - 1;
        }
    }
    selected.ok_or(MeshSdfCookError::PayloadBudgetTooSmall)
}

fn volume_layout_for_max_dimension(
    source_bounds: Aabb,
    max_dimension: u32,
) -> Result<VolumeLayout, MeshSdfCookError> {
    let source_extent = source_bounds.max - source_bounds.min;
    let max_extent = source_extent.max_element();
    if !max_extent.is_finite() || max_extent <= f32::EPSILON {
        return Err(MeshSdfCookError::DegenerateGeometry);
    }
    let interior_dimension = max_dimension.saturating_sub(MESH_SDF_OBJECT_BORDER_VOXELS * 2);
    if interior_dimension == 0 {
        return Err(MeshSdfCookError::InvalidSettings);
    }
    let voxel_size = max_extent / interior_dimension as f32;
    let dimensions = [source_extent.x, source_extent.y, source_extent.z].map(|extent| {
        ((extent / voxel_size).ceil() as u32)
            .saturating_add(MESH_SDF_OBJECT_BORDER_VOXELS * 2)
            .clamp(MIN_MESH_SDF_DIMENSION, max_dimension)
    });
    let volume_extent = Vec3::new(
        dimensions[0] as f32,
        dimensions[1] as f32,
        dimensions[2] as f32,
    ) * voxel_size;
    let center = (source_bounds.min + source_bounds.max) * 0.5;
    Ok(VolumeLayout {
        bounds: Aabb {
            min: center - volume_extent * 0.5,
            max: center + volume_extent * 0.5,
        },
        dimensions,
        voxel_size,
    })
}

fn jittered_sign_origin(point: Vec3, linear_index: usize, voxel_size: f32) -> Vec3 {
    let hash = (linear_index as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    let y = ((hash >> 16) & 0xff) as f32 / 255.0 - 0.5;
    let z = ((hash >> 32) & 0xff) as f32 / 255.0 - 0.5;
    point + Vec3::new(0.0, y, z) * (voxel_size * 1.0e-4)
}

fn encode_snorm16_distance(distance: f32, distance_limit: f32) -> i16 {
    let normalized = (distance / distance_limit).clamp(-1.0, 1.0);
    (normalized * SNORM16_MAX).round() as i16
}
