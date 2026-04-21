use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::{RenderHybridGiProbe, RenderHybridGiTraceRegion};
use crate::core::math::Vec3;

use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::{
    hybrid_gi_voxel_clipmap_cell_center, HybridGiPrepareCardCaptureRequest,
    HybridGiPrepareVoxelClipmap, ViewportRenderFrame, HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT,
    HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION,
};

use super::hybrid_gi_budget_weight::hybrid_gi_budget_weight;
use super::runtime_parent_chain::{
    blend_runtime_rgb_lineage_sources, gather_runtime_descendant_chain_rgb,
    gather_runtime_parent_chain_rgb, runtime_resolve_weight_support,
};

const ANCESTOR_TRACE_INHERITANCE_FALLOFF: f32 = 0.72;
const TRACE_INHERITANCE_WEIGHT_SCALE: f32 = 0.45;
const SCENE_PREPARE_VOXEL_FALLBACK_WEIGHT_SCALE: f32 = 0.6;

#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn hybrid_gi_hierarchy_rt_lighting(
    frame: &ViewportRenderFrame,
    source: &RenderHybridGiProbe,
) -> [f32; 4] {
    hybrid_gi_hierarchy_rt_lighting_with_scene_prepare_resources(frame, source, None)
}

pub(crate) fn hybrid_gi_hierarchy_rt_lighting_with_scene_prepare_resources(
    frame: &ViewportRenderFrame,
    source: &RenderHybridGiProbe,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> [f32; 4] {
    let Some(extract) = frame.extract.lighting.hybrid_global_illumination.as_ref() else {
        return [0.0; 4];
    };
    let prepare = frame.hybrid_gi_prepare.as_ref();
    let scheduled_trace_region_ids = prepare
        .map(|prepare| prepare.scheduled_trace_region_ids.as_slice())
        .unwrap_or(&[]);

    let probes_by_id = extract
        .probes
        .iter()
        .copied()
        .map(|probe| (probe.probe_id, probe))
        .collect::<BTreeMap<_, _>>();
    let trace_regions_by_id = extract
        .trace_regions
        .iter()
        .copied()
        .map(|region| (region.region_id, region))
        .collect::<BTreeMap<_, _>>();
    let resident_prepare_by_id = prepare
        .map(|prepare| {
            prepare
                .resident_probes
                .iter()
                .map(|probe| (probe.probe_id, probe))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let exact_runtime_rt_lighting =
        runtime_hierarchy_rt_lighting(frame, source, &resident_prepare_by_id)
            .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let inherited_runtime_rt_lighting =
        gather_runtime_parent_chain_rgb(frame, source.probe_id, |runtime, ancestor_probe_id| {
            if let Some(hierarchy_rt_lighting) = runtime.hierarchy_rt_lighting(ancestor_probe_id) {
                return Some((
                    [
                        hierarchy_rt_lighting[0],
                        hierarchy_rt_lighting[1],
                        hierarchy_rt_lighting[2],
                    ],
                    hierarchy_rt_lighting[3],
                ));
            }

            runtime
                .probe_rt_lighting_rgb
                .get(&ancestor_probe_id)
                .copied()
                .map(|rgb| {
                    (
                        [
                            rgb[0] as f32 / 255.0,
                            rgb[1] as f32 / 255.0,
                            rgb[2] as f32 / 255.0,
                        ],
                        runtime_resolve_weight_support(
                            runtime.hierarchy_resolve_weight(ancestor_probe_id),
                        ),
                    )
                })
        })
        .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let descendant_runtime_rt_lighting = gather_runtime_descendant_chain_rgb(
        frame,
        source.probe_id,
        |runtime, descendant_probe_id| {
            if let Some(hierarchy_rt_lighting) = runtime.hierarchy_rt_lighting(descendant_probe_id)
            {
                return Some((
                    [
                        hierarchy_rt_lighting[0],
                        hierarchy_rt_lighting[1],
                        hierarchy_rt_lighting[2],
                    ],
                    hierarchy_rt_lighting[3],
                ));
            }

            runtime
                .probe_rt_lighting_rgb
                .get(&descendant_probe_id)
                .copied()
                .map(|rgb| {
                    (
                        [
                            rgb[0] as f32 / 255.0,
                            rgb[1] as f32 / 255.0,
                            rgb[2] as f32 / 255.0,
                        ],
                        runtime_resolve_weight_support(
                            runtime.hierarchy_resolve_weight(descendant_probe_id),
                        ),
                    )
                })
        },
    )
    .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    if let Some(runtime_rt_lighting) = blend_runtime_rgb_lineage_sources(
        exact_runtime_rt_lighting,
        inherited_runtime_rt_lighting,
        descendant_runtime_rt_lighting,
    ) {
        return runtime_rt_lighting;
    }
    let scene_prepare_voxel_fallback =
        scene_prepare_voxel_fallback_rt_lighting(frame, source, scene_prepare_resources);

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    if scheduled_trace_region_ids.is_empty() {
        return scene_prepare_voxel_fallback.unwrap_or([0.0; 4]);
    }

    let mut current_probe_id = source.probe_id;
    let mut visited_probe_ids = BTreeSet::from([source.probe_id]);
    let mut ancestor_depth = 0usize;

    loop {
        let Some(parent_probe_id) = probes_by_id
            .get(&current_probe_id)
            .and_then(|probe| probe.parent_probe_id)
        else {
            break;
        };
        if !visited_probe_ids.insert(parent_probe_id) {
            break;
        }
        let Some(ancestor_probe) = probes_by_id.get(&parent_probe_id) else {
            break;
        };
        let resident_budget_weight = resident_prepare_by_id
            .get(&parent_probe_id)
            .map(|probe| hybrid_gi_budget_weight(probe.ray_budget))
            .unwrap_or(0.0);
        if resident_budget_weight <= f32::EPSILON {
            current_probe_id = parent_probe_id;
            continue;
        }

        ancestor_depth += 1;
        let hierarchy_weight =
            ANCESTOR_TRACE_INHERITANCE_FALLOFF.powi((ancestor_depth.saturating_sub(1)) as i32);
        for region_id in scheduled_trace_region_ids {
            let Some(region) = trace_regions_by_id.get(region_id) else {
                continue;
            };
            let region_rt_lighting = hybrid_gi_trace_region_rt_lighting(region);
            if region_rt_lighting[3] <= 0.0 {
                continue;
            }

            let support = hierarchy_weight
                * resident_budget_weight
                * region_rt_lighting[3]
                * hierarchy_trace_region_support(ancestor_probe, region);
            if support <= 0.0 {
                continue;
            }

            weighted_rgb[0] += region_rt_lighting[0] * support;
            weighted_rgb[1] += region_rt_lighting[1] * support;
            weighted_rgb[2] += region_rt_lighting[2] * support;
            total_support += support;
        }

        current_probe_id = parent_probe_id;
    }

    if total_support <= f32::EPSILON {
        return scene_prepare_voxel_fallback.unwrap_or([0.0; 4]);
    }

    let inherited_weight = (total_support * TRACE_INHERITANCE_WEIGHT_SCALE).clamp(0.0, 0.75);
    [
        weighted_rgb[0] / total_support,
        weighted_rgb[1] / total_support,
        weighted_rgb[2] / total_support,
        inherited_weight,
    ]
}

fn scene_prepare_voxel_fallback_rt_lighting(
    frame: &ViewportRenderFrame,
    source: &RenderHybridGiProbe,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> Option<[f32; 4]> {
    let scene_prepare = frame.hybrid_gi_scene_prepare.as_ref()?;
    let clipmaps_by_id = scene_prepare
        .voxel_clipmaps
        .iter()
        .map(|clipmap| (clipmap.clipmap_id, clipmap))
        .collect::<BTreeMap<_, _>>();
    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;

    for cell in &scene_prepare.voxel_cells {
        if cell.occupancy_count == 0 {
            continue;
        }
        let Some(clipmap) = clipmaps_by_id.get(&cell.clipmap_id).copied() else {
            continue;
        };
        let cell_index = cell.cell_index as usize;
        if cell_index >= HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT {
            continue;
        }

        let cell_x = cell_index % HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION;
        let cell_y = (cell_index / HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION)
            % HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION;
        let cell_z = cell_index
            / (HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION * HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION);
        let cell_center = hybrid_gi_voxel_clipmap_cell_center(clipmap, cell_x, cell_y, cell_z);
        let cell_half_extent =
            (clipmap.half_extent / HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION as f32).max(0.05);
        let support = scene_prepare_voxel_cell_support(
            source.position,
            source.radius,
            cell_center,
            cell_half_extent,
            cell.occupancy_count,
        );
        if support <= f32::EPSILON {
            continue;
        }

        let base_rgb = scene_prepare_voxel_cell_base_rgb(
            scene_prepare,
            scene_prepare_resources,
            cell,
            clipmap,
            cell_center,
        );
        weighted_rgb[0] += base_rgb[0] * support;
        weighted_rgb[1] += base_rgb[1] * support;
        weighted_rgb[2] += base_rgb[2] * support;
        total_support += support;
    }

    if total_support <= f32::EPSILON {
        return scene_prepare_voxel_clipmap_fallback_rt_lighting(
            source,
            &scene_prepare.voxel_clipmaps,
            scene_prepare_resources,
        );
    }

    Some([
        weighted_rgb[0] / total_support,
        weighted_rgb[1] / total_support,
        weighted_rgb[2] / total_support,
        (total_support * SCENE_PREPARE_VOXEL_FALLBACK_WEIGHT_SCALE).clamp(0.18, 0.7),
    ])
}

fn scene_prepare_voxel_clipmap_fallback_rt_lighting(
    source: &RenderHybridGiProbe,
    voxel_clipmaps: &[HybridGiPrepareVoxelClipmap],
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> Option<[f32; 4]> {
    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;

    for clipmap in voxel_clipmaps {
        let support = scene_prepare_voxel_clipmap_support(
            source.position,
            source.radius,
            clipmap.center,
            clipmap.half_extent,
        );
        if support <= f32::EPSILON {
            continue;
        }

        let base_rgb =
            scene_prepare_voxel_clipmap_base_rgb(source.position, clipmap, scene_prepare_resources);
        weighted_rgb[0] += base_rgb[0] * support;
        weighted_rgb[1] += base_rgb[1] * support;
        weighted_rgb[2] += base_rgb[2] * support;
        total_support += support;
    }

    if total_support <= f32::EPSILON {
        return None;
    }

    Some([
        weighted_rgb[0] / total_support,
        weighted_rgb[1] / total_support,
        weighted_rgb[2] / total_support,
        (total_support * 0.65).clamp(0.22, 0.65),
    ])
}

fn scene_prepare_voxel_cell_support(
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

fn scene_prepare_voxel_cell_base_rgb(
    scene_prepare: &crate::graphics::types::HybridGiScenePrepareFrame,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
    cell: &crate::graphics::types::HybridGiPrepareVoxelCell,
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

fn scene_prepare_voxel_cell_resource_rgb(
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
    cell: &crate::graphics::types::HybridGiPrepareVoxelCell,
) -> Option<[f32; 3]> {
    let scene_prepare_resources = scene_prepare_resources?;
    scene_prepare_resources
        .voxel_clipmap_cell_dominant_rgba_samples
        .iter()
        .find(|(clipmap_id, cell_index, rgba)| {
            *clipmap_id == cell.clipmap_id
                && *cell_index == cell.cell_index
                && rgba_sample_is_present(*rgba)
        })
        .map(|(_, _, rgba)| rgba_sample_rgb(*rgba))
        .or_else(|| {
            scene_prepare_resources
                .voxel_clipmap_cell_rgba_samples
                .iter()
                .find(|(clipmap_id, cell_index, rgba)| {
                    *clipmap_id == cell.clipmap_id
                        && *cell_index == cell.cell_index
                        && rgba_sample_is_present(*rgba)
                })
                .map(|(_, _, rgba)| rgba_sample_rgb(*rgba))
        })
}

fn scene_prepare_voxel_owner_card_capture_rgb(
    scene_prepare: &crate::graphics::types::HybridGiScenePrepareFrame,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
    cell: &crate::graphics::types::HybridGiPrepareVoxelCell,
) -> Option<[f32; 3]> {
    let owner_id = cell.dominant_card_id;
    if owner_id == 0 {
        return None;
    }

    scene_prepare
        .card_capture_requests
        .iter()
        .find(|request| request.card_id == owner_id)
        .map(|request| scene_prepare_card_capture_request_rgb(request, scene_prepare_resources))
}

fn scene_prepare_card_capture_request_rgb(
    request: &HybridGiPrepareCardCaptureRequest,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> [f32; 3] {
    if let Some(scene_prepare_resources) = scene_prepare_resources {
        if let Some((_, rgba)) = scene_prepare_resources
            .capture_slot_rgba_samples
            .iter()
            .find(|(slot_id, _)| *slot_id == request.capture_slot_id)
        {
            return [
                rgba[0] as f32 / 255.0,
                rgba[1] as f32 / 255.0,
                rgba[2] as f32 / 255.0,
            ];
        }

        if let Some((_, rgba)) = scene_prepare_resources
            .atlas_slot_rgba_samples
            .iter()
            .find(|(slot_id, _)| *slot_id == request.atlas_slot_id)
        {
            return [
                rgba[0] as f32 / 255.0,
                rgba[1] as f32 / 255.0,
                rgba[2] as f32 / 255.0,
            ];
        }
    }

    let bounds_center_x_q = quantized_signed(request.bounds_center.x);
    let bounds_center_z_q = quantized_signed(request.bounds_center.z);
    let bounds_radius_q = quantized_positive(request.bounds_radius, 64.0);

    [
        (96 + ((request.card_id * 17 + request.page_id * 5 + request.capture_slot_id * 3) % 96))
            as f32
            / 255.0,
        (72 + ((request.page_id * 13 + request.atlas_slot_id * 7 + bounds_radius_q) % 80)) as f32
            / 255.0,
        (40 + ((request.card_id * 11 + bounds_center_x_q + bounds_center_z_q) % 56)) as f32 / 255.0,
    ]
}

fn quantized_signed(value: f32) -> u32 {
    ((value * 64.0).round() as i32).wrapping_add(2048) as u32
}

fn quantized_positive(value: f32, scale: f32) -> u32 {
    (value.max(0.0) * scale).round() as u32
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

fn scene_prepare_voxel_clipmap_support(
    probe_position: Vec3,
    probe_radius: f32,
    clipmap_center: Vec3,
    clipmap_half_extent: f32,
) -> f32 {
    let reach = (probe_radius.max(0.05) + clipmap_half_extent.max(0.05) * 1.5).max(0.05);
    (1.0 - probe_position.distance(clipmap_center) / reach).max(0.0) * 0.9
}

fn scene_prepare_voxel_clipmap_base_rgb(
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
        .voxel_clipmap_rgba_samples
        .iter()
        .find(|(clipmap_id, rgba)| {
            *clipmap_id == clipmap.clipmap_id && rgba_sample_is_present(*rgba)
        })
        .map(|(_, rgba)| rgba_sample_rgb(*rgba))
}

fn rgba_sample_is_present(rgba: [u8; 4]) -> bool {
    rgba[3] > 0
}

fn rgba_sample_rgb(rgba: [u8; 4]) -> [f32; 3] {
    [
        rgba[0] as f32 / 255.0,
        rgba[1] as f32 / 255.0,
        rgba[2] as f32 / 255.0,
    ]
}

fn runtime_hierarchy_rt_lighting(
    frame: &ViewportRenderFrame,
    source: &RenderHybridGiProbe,
    resident_prepare_by_id: &BTreeMap<u32, &crate::graphics::types::HybridGiPrepareProbe>,
) -> Option<[f32; 4]> {
    let runtime = frame.hybrid_gi_resolve_runtime.as_ref()?;
    let direct_rt_lighting_rgb = runtime.probe_rt_lighting_rgb.get(&source.probe_id).copied();
    let hierarchy_rt_lighting = runtime.hierarchy_rt_lighting(source.probe_id);
    if direct_rt_lighting_rgb.is_none() && hierarchy_rt_lighting.is_none() {
        return None;
    }

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    if let Some(direct_rt_lighting_rgb) = direct_rt_lighting_rgb {
        let direct_support = resident_prepare_by_id
            .get(&source.probe_id)
            .map(|probe| (0.25 + hybrid_gi_budget_weight(probe.ray_budget) * 0.5).clamp(0.25, 0.75))
            .unwrap_or(0.3);
        weighted_rgb[0] += direct_rt_lighting_rgb[0] as f32 / 255.0 * direct_support;
        weighted_rgb[1] += direct_rt_lighting_rgb[1] as f32 / 255.0 * direct_support;
        weighted_rgb[2] += direct_rt_lighting_rgb[2] as f32 / 255.0 * direct_support;
        total_support += direct_support;
    }
    if let Some(hierarchy_rt_lighting) = hierarchy_rt_lighting {
        if hierarchy_rt_lighting[3] > f32::EPSILON {
            weighted_rgb[0] += hierarchy_rt_lighting[0] * hierarchy_rt_lighting[3];
            weighted_rgb[1] += hierarchy_rt_lighting[1] * hierarchy_rt_lighting[3];
            weighted_rgb[2] += hierarchy_rt_lighting[2] * hierarchy_rt_lighting[3];
            total_support += hierarchy_rt_lighting[3];
        }
    }

    if total_support <= f32::EPSILON {
        return None;
    }

    Some([
        weighted_rgb[0] / total_support,
        weighted_rgb[1] / total_support,
        weighted_rgb[2] / total_support,
        total_support.clamp(0.0, 0.75),
    ])
}

fn hierarchy_trace_region_support(
    probe: &RenderHybridGiProbe,
    region: &RenderHybridGiTraceRegion,
) -> f32 {
    let reach = (probe.radius.max(0.05) + region.bounds_radius.max(0.05)).max(0.05);
    let distance = probe.position.distance(region.bounds_center);
    let falloff = (1.0 - distance / reach).max(0.0);
    let coverage_weight = (0.35 + region.screen_coverage.clamp(0.0, 1.0) * 0.65).clamp(0.35, 1.0);
    falloff * falloff * coverage_weight
}

fn hybrid_gi_trace_region_rt_lighting(region: &RenderHybridGiTraceRegion) -> [f32; 4] {
    let rgb = [
        region.rt_lighting_rgb[0] as f32 / 255.0,
        region.rt_lighting_rgb[1] as f32 / 255.0,
        region.rt_lighting_rgb[2] as f32 / 255.0,
    ];
    let max_component = rgb[0].max(rgb[1]).max(rgb[2]);

    [rgb[0], rgb[1], rgb[2], max_component]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        hybrid_gi_hierarchy_rt_lighting,
        hybrid_gi_hierarchy_rt_lighting_with_scene_prepare_resources,
    };
    use crate::core::framework::render::{
        FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderHybridGiExtract,
        RenderHybridGiProbe, RenderOverlayExtract, RenderSceneGeometryExtract, RenderSceneSnapshot,
        RenderWorldSnapshotHandle, ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec3, Vec4};
    use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
    use crate::graphics::types::{
        HybridGiPrepareCardCaptureRequest, HybridGiPrepareFrame, HybridGiPrepareProbe,
        HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap, HybridGiResolveRuntime,
        HybridGiScenePrepareFrame, ViewportRenderFrame,
    };

    #[test]
    fn exact_runtime_rt_lighting_keeps_blending_with_descendant_continuation() {
        let warm = hierarchy_rt_lighting_with_descendant(
            HybridGiResolveRuntime::pack_rgb_and_weight([0.95, 0.3, 0.12], 0.62),
        );
        let cool = hierarchy_rt_lighting_with_descendant(
            HybridGiResolveRuntime::pack_rgb_and_weight([0.12, 0.3, 0.95], 0.62),
        );

        assert!(
            warm[0] > cool[0] + 0.2,
            "expected exact runtime RT lighting to keep blending with descendant continuation instead of shadowing it; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            cool[2] > warm[2] + 0.2,
            "expected descendant continuation to affect RT blue when the child runtime turns cool; warm={warm:?}, cool={cool:?}"
        );
    }

    #[test]
    fn scene_prepare_voxel_cell_resource_samples_override_spatial_fallback_when_runtime_authority_is_absent(
    ) {
        let scene_prepare = HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 4.0,
            }],
            voxel_cells: vec![
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 20,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 21,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 24,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 25,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
            ],
        };

        let warm = scene_prepare_rt_lighting_with_resources(
            scene_prepare.clone(),
            scene_prepare_resources_snapshot(
                vec![(7, [160, 120, 96, 255])],
                vec![
                    (7, 20, [240, 96, 48, 255]),
                    (7, 21, [240, 96, 48, 255]),
                    (7, 24, [240, 96, 48, 255]),
                    (7, 25, [240, 96, 48, 255]),
                ],
                Vec::new(),
            ),
        );
        let cool = scene_prepare_rt_lighting_with_resources(
            scene_prepare,
            scene_prepare_resources_snapshot(
                vec![(7, [96, 120, 160, 255])],
                vec![
                    (7, 20, [48, 96, 240, 255]),
                    (7, 21, [48, 96, 240, 255]),
                    (7, 24, [48, 96, 240, 255]),
                    (7, 25, [48, 96, 240, 255]),
                ],
                Vec::new(),
            ),
        );

        assert!(
            warm[0] > cool[0] + 0.2,
            "expected current-frame scene_prepare voxel-cell resource samples to override the spatial fallback when runtime radiance/owner authority is absent; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            cool[2] > warm[2] + 0.2,
            "expected current-frame scene_prepare voxel-cell resource samples to preserve blue authority when runtime radiance/owner authority is absent; warm={warm:?}, cool={cool:?}"
        );
    }

    #[test]
    fn scene_prepare_voxel_clipmap_resource_samples_override_spatial_fallback_when_runtime_cells_are_absent(
    ) {
        let scene_prepare = HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::new(0.0, 0.0, 0.2),
                half_extent: 4.0,
            }],
            voxel_cells: Vec::new(),
        };

        let warm = scene_prepare_rt_lighting_with_resources(
            scene_prepare.clone(),
            scene_prepare_resources_snapshot(vec![(7, [224, 96, 48, 255])], Vec::new(), Vec::new()),
        );
        let cool = scene_prepare_rt_lighting_with_resources(
            scene_prepare,
            scene_prepare_resources_snapshot(vec![(7, [48, 96, 224, 255])], Vec::new(), Vec::new()),
        );

        assert!(
            warm[0] > cool[0] + 0.2,
            "expected current-frame scene_prepare voxel-clipmap resource samples to override the coarse spatial fallback when runtime cells are absent; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            cool[2] > warm[2] + 0.2,
            "expected current-frame scene_prepare voxel-clipmap resource samples to preserve blue authority when runtime cells are absent; warm={warm:?}, cool={cool:?}"
        );
    }

    #[test]
    fn scene_prepare_present_black_voxel_cell_resource_samples_stay_authoritative_over_spatial_fallback(
    ) {
        let scene_prepare = HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 4.0,
            }],
            voxel_cells: vec![
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 20,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 21,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 24,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 25,
                    occupancy_count: 4,
                    dominant_card_id: 0,
                    radiance_present: false,
                    radiance_rgb: [0, 0, 0],
                },
            ],
        };

        let present_black = scene_prepare_rt_lighting_with_resources(
            scene_prepare.clone(),
            scene_prepare_resources_snapshot(
                Vec::new(),
                Vec::new(),
                repeated_cell_samples([0, 0, 0, 255]),
            ),
        );
        let absent = scene_prepare_rt_lighting_with_resources(
            scene_prepare,
            scene_prepare_resources_snapshot(
                Vec::new(),
                Vec::new(),
                repeated_cell_samples([0, 0, 0, 0]),
            ),
        );

        assert!(
            present_black[3] > 0.18,
            "expected explicit-black cell resources to remain a valid GI fallback source; present_black={present_black:?}"
        );
        assert!(
            rgb_energy(present_black) < 0.05,
            "expected explicit-black cell resources to stay black instead of collapsing back to spatial fallback; present_black={present_black:?}"
        );
        assert!(
            rgb_energy(absent) > rgb_energy(present_black) + 0.2,
            "expected absent cell resources to fall back to the spatial heuristic while explicit-black resources remain authoritative; present_black={present_black:?}, absent={absent:?}"
        );
    }

    #[test]
    fn scene_prepare_present_black_voxel_clipmap_resource_samples_stay_authoritative_over_spatial_fallback(
    ) {
        let scene_prepare = HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::new(0.0, 0.0, 0.2),
                half_extent: 4.0,
            }],
            voxel_cells: Vec::new(),
        };

        let present_black = scene_prepare_rt_lighting_with_resources(
            scene_prepare.clone(),
            scene_prepare_resources_snapshot(vec![(7, [0, 0, 0, 255])], Vec::new(), Vec::new()),
        );
        let absent = scene_prepare_rt_lighting_with_resources(
            scene_prepare,
            scene_prepare_resources_snapshot(vec![(7, [0, 0, 0, 0])], Vec::new(), Vec::new()),
        );

        assert!(
            present_black[3] > 0.18,
            "expected explicit-black clipmap resources to remain a valid GI fallback source; present_black={present_black:?}"
        );
        assert!(
            rgb_energy(present_black) < 0.05,
            "expected explicit-black clipmap resources to stay black instead of collapsing back to spatial fallback; present_black={present_black:?}"
        );
        assert!(
            rgb_energy(absent) > rgb_energy(present_black) + 0.2,
            "expected absent clipmap resources to fall back to the spatial heuristic while explicit-black clipmap resources remain authoritative; present_black={present_black:?}, absent={absent:?}"
        );
    }

    #[test]
    fn runtime_explicit_black_voxel_radiance_stays_authoritative_over_owner_card_and_spatial_fallback(
    ) {
        let card_capture_request = HybridGiPrepareCardCaptureRequest {
            card_id: 11,
            page_id: 22,
            atlas_slot_id: 3,
            capture_slot_id: 4,
            bounds_center: Vec3::new(20.0, 20.0, 20.0),
            bounds_radius: 0.25,
        };
        let scene_prepare = HybridGiScenePrepareFrame {
            card_capture_requests: vec![card_capture_request],
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 4.0,
            }],
            voxel_cells: runtime_owner_voxel_cells(true),
        };

        let present_black = scene_prepare_rt_lighting_with_resources(
            scene_prepare.clone(),
            scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
        );
        let absent = scene_prepare_rt_lighting_with_resources(
            HybridGiScenePrepareFrame {
                voxel_cells: runtime_owner_voxel_cells(false),
                ..scene_prepare
            },
            scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
        );

        assert!(
            present_black[3] > 0.18,
            "expected explicit-black runtime voxel radiance to remain a valid GI fallback source; present_black={present_black:?}"
        );
        assert!(
            rgb_energy(present_black) < 0.05,
            "expected explicit-black runtime voxel radiance to stay black instead of collapsing to owner-card or spatial fallback; present_black={present_black:?}"
        );
        assert!(
            rgb_energy(absent) > rgb_energy(present_black) + 0.2,
            "expected absent runtime voxel radiance to fall back to owner-card or spatial fallback while explicit-black runtime radiance remains authoritative; present_black={present_black:?}, absent={absent:?}"
        );
    }

    fn hierarchy_rt_lighting_with_descendant(descendant_runtime: [u8; 4]) -> [f32; 4] {
        let parent_probe = RenderHybridGiProbe {
            probe_id: 100,
            resident: true,
            ray_budget: 128,
            ..Default::default()
        };
        let child_probe = RenderHybridGiProbe {
            probe_id: 200,
            parent_probe_id: Some(parent_probe.probe_id),
            ray_budget: 88,
            ..Default::default()
        };

        let snapshot = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        let mut extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            probe_budget: 2,
            trace_budget: 2,
            card_budget: 2,
            voxel_budget: 1,
            probes: vec![parent_probe, child_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_hierarchy_rt_lighting_rgb_and_weight: BTreeMap::from([
                    (
                        parent_probe.probe_id,
                        HybridGiResolveRuntime::pack_rgb_and_weight([0.5, 0.5, 0.5], 0.12),
                    ),
                    (child_probe.probe_id, descendant_runtime),
                ]),
                ..Default::default()
            }));

        hybrid_gi_hierarchy_rt_lighting(&frame, &parent_probe)
    }

    fn scene_prepare_rt_lighting_with_resources(
        scene_prepare: HybridGiScenePrepareFrame,
        scene_prepare_resources: HybridGiScenePrepareResourcesSnapshot,
    ) -> [f32; 4] {
        let probe = RenderHybridGiProbe {
            probe_id: 200,
            resident: true,
            ray_budget: 128,
            radius: 1.8,
            ..Default::default()
        };
        let snapshot = RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot::default(),
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        };
        let mut extract =
            RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
        extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
            enabled: true,
            probe_budget: 1,
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 1,
            probes: vec![probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: probe.probe_id,
                    slot: 0,
                    ray_budget: probe.ray_budget,
                    irradiance_rgb: [112, 112, 112],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(scene_prepare));

        hybrid_gi_hierarchy_rt_lighting_with_scene_prepare_resources(
            &frame,
            &probe,
            Some(&scene_prepare_resources),
        )
    }

    fn scene_prepare_resources_snapshot(
        voxel_clipmap_rgba_samples: Vec<(u32, [u8; 4])>,
        voxel_clipmap_cell_rgba_samples: Vec<(u32, u32, [u8; 4])>,
        voxel_clipmap_cell_dominant_rgba_samples: Vec<(u32, u32, [u8; 4])>,
    ) -> HybridGiScenePrepareResourcesSnapshot {
        HybridGiScenePrepareResourcesSnapshot {
            card_capture_request_count: 0,
            voxel_clipmap_ids: vec![7],
            occupied_atlas_slots: Vec::new(),
            occupied_capture_slots: Vec::new(),
            atlas_slot_rgba_samples: Vec::new(),
            capture_slot_rgba_samples: Vec::new(),
            voxel_clipmap_rgba_samples,
            voxel_clipmap_occupancy_masks: Vec::new(),
            voxel_clipmap_cell_rgba_samples,
            voxel_clipmap_cell_occupancy_counts: Vec::new(),
            voxel_clipmap_cell_dominant_node_ids: Vec::new(),
            voxel_clipmap_cell_dominant_rgba_samples,
            atlas_slot_count: 0,
            capture_slot_count: 0,
            atlas_texture_extent: (0, 0),
            capture_texture_extent: (0, 0),
            capture_layer_count: 0,
        }
    }

    fn repeated_cell_samples(rgba: [u8; 4]) -> Vec<(u32, u32, [u8; 4])> {
        [20_u32, 21, 24, 25]
            .into_iter()
            .map(|cell_index| (7, cell_index, rgba))
            .collect()
    }

    fn runtime_owner_voxel_cells(radiance_present: bool) -> Vec<HybridGiPrepareVoxelCell> {
        [20_u32, 21, 24, 25]
            .into_iter()
            .map(|cell_index| HybridGiPrepareVoxelCell {
                clipmap_id: 7,
                cell_index,
                occupancy_count: 4,
                dominant_card_id: 11,
                radiance_present,
                radiance_rgb: [0, 0, 0],
            })
            .collect()
    }

    fn rgb_energy(sample: [f32; 4]) -> f32 {
        sample[0] + sample[1] + sample[2]
    }
}
