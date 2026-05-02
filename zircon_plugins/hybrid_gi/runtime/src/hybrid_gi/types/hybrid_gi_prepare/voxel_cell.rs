use zircon_runtime::core::math::Vec3;

use super::HybridGiPrepareVoxelClipmap;

pub const HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION: usize = 4;
pub const HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT: usize = HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION
    * HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION
    * HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HybridGiPrepareVoxelCell {
    pub clipmap_id: u32,
    pub cell_index: u32,
    pub occupancy_count: u32,
    pub dominant_card_id: u32,
    pub radiance_present: bool,
    pub radiance_rgb: [u8; 3],
}

pub fn hybrid_gi_voxel_clipmap_bounds_cell_ranges(
    clipmap: &HybridGiPrepareVoxelClipmap,
    bounds_center: Vec3,
    bounds_radius: f32,
) -> Option<[(usize, usize); 3]> {
    if clipmap.half_extent <= 0.0 || bounds_radius <= 0.0 {
        return None;
    }

    let clipmap_min = clipmap.center - Vec3::splat(clipmap.half_extent);
    let clipmap_extent = clipmap.half_extent * 2.0;
    let bounds_min = bounds_center - Vec3::splat(bounds_radius);
    let bounds_max = bounds_center + Vec3::splat(bounds_radius);

    Some([
        mesh_axis_range(clipmap_min.x, clipmap_extent, bounds_min.x, bounds_max.x)?,
        mesh_axis_range(clipmap_min.y, clipmap_extent, bounds_min.y, bounds_max.y)?,
        mesh_axis_range(clipmap_min.z, clipmap_extent, bounds_min.z, bounds_max.z)?,
    ])
}

pub fn hybrid_gi_voxel_clipmap_cell_bit_index(x: usize, y: usize, z: usize) -> usize {
    x + y * HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION
        + z * HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION * HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION
}

pub fn hybrid_gi_voxel_clipmap_cell_center(
    clipmap: &HybridGiPrepareVoxelClipmap,
    x: usize,
    y: usize,
    z: usize,
) -> Vec3 {
    let clipmap_min = clipmap.center - Vec3::splat(clipmap.half_extent);
    let cell_extent = (clipmap.half_extent * 2.0) / HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION as f32;
    clipmap_min
        + Vec3::new(
            (x as f32 + 0.5) * cell_extent,
            (y as f32 + 0.5) * cell_extent,
            (z as f32 + 0.5) * cell_extent,
        )
}

fn mesh_axis_range(
    clipmap_min: f32,
    clipmap_extent: f32,
    bounds_min: f32,
    bounds_max: f32,
) -> Option<(usize, usize)> {
    if clipmap_extent <= 0.0
        || bounds_max < clipmap_min
        || bounds_min > clipmap_min + clipmap_extent
    {
        return None;
    }

    let cell_extent = clipmap_extent / HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION as f32;
    let clamped_bounds_min = bounds_min.clamp(clipmap_min, clipmap_min + clipmap_extent);
    let clamped_bounds_max = bounds_max.clamp(clipmap_min, clipmap_min + clipmap_extent);
    let start = (((clamped_bounds_min - clipmap_min) / cell_extent).floor() as isize)
        .clamp(0, HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION as isize - 1) as usize;
    let end = ((((clamped_bounds_max - clipmap_min) / cell_extent).ceil() as isize) - 1)
        .clamp(0, HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION as isize - 1) as usize;

    Some((start.min(end), start.max(end)))
}
