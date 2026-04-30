use crate::core::math::Vec3;
use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::{
    HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap, HybridGiScenePrepareFrame,
};

use super::super::scene_prepare_surface_cache_samples::{
    rgba_sample_is_present, rgba_sample_rgb, scene_prepare_surface_cache_owner_rgb_and_quality,
};

pub(super) fn scene_prepare_voxel_cell_support(
    probe_position: Vec3,
    probe_radius: f32,
    cell_center: Vec3,
    cell_half_extent: f32,
    occupancy_count: u32,
) -> f32 {
    let reach = (probe_radius.max(0.05) + cell_half_extent * 2.5).max(0.05);
    let falloff = (1.0 - probe_position.distance(cell_center) / reach).max(0.0);
    if falloff <= f32::EPSILON {
        return 0.0;
    }

    let occupancy_support = (occupancy_count.min(8) as f32 / 8.0).max(0.125);
    falloff * (0.18 + occupancy_support * 0.82)
}

pub(super) fn scene_prepare_voxel_cell_base_rgb(
    scene_prepare: &HybridGiScenePrepareFrame,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
    cell: &HybridGiPrepareVoxelCell,
    clipmap: &HybridGiPrepareVoxelClipmap,
    cell_center: Vec3,
) -> [f32; 3] {
    let spatial_rgb =
        scene_prepare_voxel_cell_spatial_rgb(clipmap, cell_center, cell.occupancy_count);
    let authority_rgb = [
        cell.radiance_rgb[0] as f32 / 255.0,
        cell.radiance_rgb[1] as f32 / 255.0,
        cell.radiance_rgb[2] as f32 / 255.0,
    ];
    if !cell.radiance_present {
        if let Some(resource_rgb) =
            scene_prepare_voxel_cell_resource_rgb(scene_prepare_resources, cell)
        {
            return resource_rgb;
        }

        if let Some(owner_rgb) =
            scene_prepare_voxel_owner_card_capture_rgb(scene_prepare, scene_prepare_resources, cell)
        {
            return owner_rgb;
        }

        return spatial_rgb;
    }

    if authority_rgb[0] + authority_rgb[1] + authority_rgb[2] <= f32::EPSILON {
        return [0.0, 0.0, 0.0];
    }

    let occupancy_bias = cell.occupancy_count.min(8) as f32 / 8.0;
    let authority_mix = (0.82 + occupancy_bias * 0.12).clamp(0.82, 0.94);
    let spatial_mix = 1.0 - authority_mix;
    [
        (authority_rgb[0] * authority_mix + spatial_rgb[0] * spatial_mix).clamp(0.0, 1.0),
        (authority_rgb[1] * authority_mix + spatial_rgb[1] * spatial_mix).clamp(0.0, 1.0),
        (authority_rgb[2] * authority_mix + spatial_rgb[2] * spatial_mix).clamp(0.0, 1.0),
    ]
}

pub(super) fn scene_prepare_voxel_cell_resource_rgb(
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
    cell: &HybridGiPrepareVoxelCell,
) -> Option<[f32; 3]> {
    let scene_prepare_resources = scene_prepare_resources?;
    scene_prepare_resources
        .voxel_clipmap_cell_dominant_rgba_sample(cell.clipmap_id, cell.cell_index)
        .filter(|rgba| rgba_sample_is_present(*rgba))
        .map(rgba_sample_rgb)
        .or_else(|| {
            scene_prepare_resources
                .voxel_clipmap_cell_rgba_sample(cell.clipmap_id, cell.cell_index)
                .filter(|rgba| rgba_sample_is_present(*rgba))
                .map(rgba_sample_rgb)
        })
}

fn scene_prepare_voxel_owner_card_capture_rgb(
    scene_prepare: &HybridGiScenePrepareFrame,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
    cell: &HybridGiPrepareVoxelCell,
) -> Option<[f32; 3]> {
    scene_prepare_voxel_owner_card_capture_rgb_and_quality(
        scene_prepare,
        scene_prepare_resources,
        cell,
    )
    .map(|(rgb, _)| rgb)
}

pub(super) fn scene_prepare_voxel_owner_card_capture_rgb_and_quality(
    scene_prepare: &HybridGiScenePrepareFrame,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
    cell: &HybridGiPrepareVoxelCell,
) -> Option<([f32; 3], f32)> {
    scene_prepare_surface_cache_owner_rgb_and_quality(
        scene_prepare,
        scene_prepare_resources,
        cell.dominant_card_id,
    )
}

fn scene_prepare_voxel_cell_spatial_rgb(
    clipmap: &HybridGiPrepareVoxelClipmap,
    cell_center: Vec3,
    occupancy_count: u32,
) -> [f32; 3] {
    let normalized = if clipmap.half_extent > f32::EPSILON {
        (cell_center - clipmap.center) / clipmap.half_extent
    } else {
        Vec3::ZERO
    };
    let warm_bias = (-normalized.x).max(0.0) * 0.55 + (-normalized.z).max(0.0) * 0.45;
    let cool_bias = normalized.x.max(0.0) * 0.55 + normalized.z.max(0.0) * 0.45;
    let vertical_bias = (1.0 - normalized.y.abs()).clamp(0.0, 1.0);
    let occupancy_bias = occupancy_count.min(8) as f32 / 8.0;

    [
        (0.14 + warm_bias * 0.62 + occupancy_bias * 0.14).clamp(0.0, 1.0),
        (0.12 + vertical_bias * 0.28 + occupancy_bias * 0.1).clamp(0.0, 1.0),
        (0.14 + cool_bias * 0.62 + occupancy_bias * 0.14).clamp(0.0, 1.0),
    ]
}

pub(super) fn scene_prepare_voxel_clipmap_support(
    probe_position: Vec3,
    probe_radius: f32,
    clipmap_center: Vec3,
    clipmap_half_extent: f32,
) -> f32 {
    let reach = (probe_radius.max(0.05) + clipmap_half_extent.max(0.05) * 1.5).max(0.05);
    (1.0 - probe_position.distance(clipmap_center) / reach).max(0.0) * 0.9
}

pub(super) fn scene_prepare_voxel_clipmap_base_rgb(
    probe_position: Vec3,
    clipmap: &HybridGiPrepareVoxelClipmap,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> [f32; 3] {
    if let Some(resource_rgb) =
        scene_prepare_voxel_clipmap_resource_rgb(scene_prepare_resources, clipmap)
    {
        return resource_rgb;
    }

    let normalized = if clipmap.half_extent > f32::EPSILON {
        (clipmap.center - probe_position) / clipmap.half_extent
    } else {
        Vec3::ZERO
    };
    let scale_bias = (clipmap.half_extent / 8.0).clamp(0.0, 1.0);
    let lateral_bias = (1.0 - normalized.x.abs()).clamp(0.0, 1.0);
    let vertical_bias = (1.0 - normalized.y.abs()).clamp(0.0, 1.0);
    let depth_bias = (1.0 - normalized.z.abs()).clamp(0.0, 1.0);

    [
        (0.46 + scale_bias * 0.22 + lateral_bias * 0.12).clamp(0.0, 1.0),
        (0.42 + scale_bias * 0.2 + vertical_bias * 0.1).clamp(0.0, 1.0),
        (0.36 + scale_bias * 0.18 + depth_bias * 0.08).clamp(0.0, 1.0),
    ]
}

fn scene_prepare_voxel_clipmap_resource_rgb(
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
    clipmap: &HybridGiPrepareVoxelClipmap,
) -> Option<[f32; 3]> {
    scene_prepare_resources?
        .voxel_clipmap_rgba_sample(clipmap.clipmap_id)
        .filter(|rgba| rgba_sample_is_present(*rgba))
        .map(rgba_sample_rgb)
}
