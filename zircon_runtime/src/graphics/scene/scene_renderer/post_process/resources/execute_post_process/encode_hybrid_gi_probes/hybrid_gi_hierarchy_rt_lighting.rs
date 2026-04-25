use std::collections::{BTreeMap, BTreeSet};

use crate::core::framework::render::{RenderHybridGiProbe, RenderHybridGiTraceRegion};
use crate::core::math::Vec3;

use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::{
    hybrid_gi_voxel_clipmap_cell_center, HybridGiPrepareVoxelClipmap, HybridGiResolveRuntime,
    ViewportRenderFrame, HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT,
    HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION,
};

use super::hybrid_gi_budget_weight::hybrid_gi_budget_weight;
use super::runtime_parent_chain::{
    blend_runtime_rgb_lineage_sources, gather_runtime_descendant_chain_rgb,
    gather_runtime_descendant_chain_rgb_without_depth_falloff, gather_runtime_parent_chain_rgb,
    gather_runtime_parent_chain_rgb_without_depth_falloff,
    runtime_parent_topology_is_authoritative, runtime_probe_lineage_has_scene_truth,
    runtime_resolve_weight_support, runtime_rt_lighting_lineage_has_scene_truth,
    scheduled_live_trace_region_ids,
};
use super::scene_prepare_surface_cache_samples::{
    rgba_sample_is_present, rgba_sample_rgb, scene_prepare_surface_cache_fallback_rgb_and_support,
    scene_prepare_surface_cache_fallback_rgb_support_and_quality,
    scene_prepare_surface_cache_owner_rgb_and_quality,
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
    let hybrid_gi_extract = frame.extract.lighting.hybrid_global_illumination.as_ref();
    let has_scene_prepare = frame.hybrid_gi_scene_prepare.is_some();
    let runtime_probe_scene_truth = runtime_probe_lineage_has_scene_truth(frame, source.probe_id);
    let stripped_runtime_probe_scene_truth = !has_scene_prepare && runtime_probe_scene_truth;
    let scene_driven_frame = has_scene_prepare
        || stripped_runtime_probe_scene_truth
        || runtime_rt_lighting_lineage_has_scene_truth(frame, source.probe_id);
    let prepare = frame.hybrid_gi_prepare.as_ref();
    let scheduled_trace_region_ids = scheduled_live_trace_region_ids(frame);

    let probes_by_id = hybrid_gi_extract
        .map(|extract| {
            extract
                .probes
                .iter()
                .copied()
                .map(|probe| (probe.probe_id, probe))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let trace_regions_by_id = hybrid_gi_extract
        .map(|extract| {
            extract
                .trace_regions
                .iter()
                .copied()
                .map(|region| (region.region_id, region))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let resident_prepare_by_id = prepare
        .map(|prepare| {
            prepare
                .resident_probes
                .iter()
                .map(|probe| (probe.probe_id, probe))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let exact_runtime_includes_scene_truth = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .map(|runtime| runtime.hierarchy_rt_lighting_includes_scene_truth(source.probe_id))
        .unwrap_or(false);
    let exact_runtime_rt_lighting = if exact_runtime_includes_scene_truth {
        frame
            .hybrid_gi_resolve_runtime
            .as_ref()
            .and_then(|runtime| {
                runtime_rt_lighting_packed_or_legacy_source(runtime, source.probe_id)
            })
    } else {
        runtime_hierarchy_rt_lighting(frame, source, &resident_prepare_by_id)
    }
    .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let exact_scene_truth_runtime_rt_lighting =
        exact_runtime_rt_lighting.filter(|_| exact_runtime_includes_scene_truth);
    let exact_continuation_runtime_rt_lighting =
        exact_runtime_rt_lighting.filter(|_| !exact_runtime_includes_scene_truth);
    let exact_scene_truth_runtime_rt_lighting_present =
        exact_scene_truth_runtime_rt_lighting.is_some();
    let inherited_scene_truth_runtime_rt_lighting =
        (!exact_scene_truth_runtime_rt_lighting_present)
            .then(|| {
                gather_runtime_parent_chain_rgb_without_depth_falloff(
                    frame,
                    source.probe_id,
                    |runtime, ancestor_probe_id| {
                        if !runtime.hierarchy_rt_lighting_includes_scene_truth(ancestor_probe_id) {
                            return None;
                        }

                        runtime_rt_lighting_lineage_source(runtime, ancestor_probe_id)
                    },
                )
            })
            .flatten()
            .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let inherited_continuation_runtime_rt_lighting =
        (!exact_scene_truth_runtime_rt_lighting_present)
            .then(|| {
                gather_runtime_parent_chain_rgb(
                    frame,
                    source.probe_id,
                    |runtime, ancestor_probe_id| {
                        if runtime.hierarchy_rt_lighting_includes_scene_truth(ancestor_probe_id) {
                            return None;
                        }

                        runtime_rt_lighting_lineage_source(runtime, ancestor_probe_id)
                    },
                )
            })
            .flatten()
            .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let descendant_scene_truth_runtime_rt_lighting =
        (!exact_scene_truth_runtime_rt_lighting_present)
            .then(|| {
                gather_runtime_descendant_chain_rgb_without_depth_falloff(
                    frame,
                    source.probe_id,
                    |runtime, descendant_probe_id| {
                        if !runtime.hierarchy_rt_lighting_includes_scene_truth(descendant_probe_id)
                        {
                            return None;
                        }

                        runtime_rt_lighting_lineage_source(runtime, descendant_probe_id)
                    },
                )
            })
            .flatten()
            .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let descendant_continuation_runtime_rt_lighting =
        (!exact_scene_truth_runtime_rt_lighting_present)
            .then(|| {
                gather_runtime_descendant_chain_rgb(
                    frame,
                    source.probe_id,
                    |runtime, descendant_probe_id| {
                        if runtime.hierarchy_rt_lighting_includes_scene_truth(descendant_probe_id) {
                            return None;
                        }

                        runtime_rt_lighting_lineage_source(runtime, descendant_probe_id)
                    },
                )
            })
            .flatten()
            .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let scene_truth_runtime_rt_lighting = blend_runtime_rgb_lineage_sources(
        exact_scene_truth_runtime_rt_lighting,
        inherited_scene_truth_runtime_rt_lighting,
        descendant_scene_truth_runtime_rt_lighting,
    );
    let continuation_runtime_rt_lighting = blend_runtime_rgb_lineage_sources(
        exact_continuation_runtime_rt_lighting,
        inherited_continuation_runtime_rt_lighting,
        descendant_continuation_runtime_rt_lighting,
    );
    let selected_runtime_rt_lighting_is_scene_truth =
        scene_driven_frame && scene_truth_runtime_rt_lighting.is_some();
    let selected_runtime_rt_lighting = if selected_runtime_rt_lighting_is_scene_truth {
        scene_truth_runtime_rt_lighting
    } else if runtime_probe_scene_truth && scene_truth_runtime_rt_lighting.is_none() {
        None
    } else {
        blend_runtime_rgb_lineage_sources(
            scene_truth_runtime_rt_lighting,
            continuation_runtime_rt_lighting,
            None,
        )
    };
    if let Some(runtime_rt_lighting) = selected_runtime_rt_lighting {
        if scene_driven_frame && !selected_runtime_rt_lighting_is_scene_truth {
            if let Some(scene_prepare_rt_lighting) =
                scene_prepare_voxel_fallback_rt_lighting(frame, source, scene_prepare_resources)
                    .filter(|scene_prepare_rt_lighting| scene_prepare_rt_lighting[3] > f32::EPSILON)
            {
                let total_support = runtime_rt_lighting[3] + scene_prepare_rt_lighting[3];
                if total_support > f32::EPSILON {
                    return [
                        (runtime_rt_lighting[0] * runtime_rt_lighting[3]
                            + scene_prepare_rt_lighting[0] * scene_prepare_rt_lighting[3])
                            / total_support,
                        (runtime_rt_lighting[1] * runtime_rt_lighting[3]
                            + scene_prepare_rt_lighting[1] * scene_prepare_rt_lighting[3])
                            / total_support,
                        (runtime_rt_lighting[2] * runtime_rt_lighting[3]
                            + scene_prepare_rt_lighting[2] * scene_prepare_rt_lighting[3])
                            / total_support,
                        total_support.clamp(0.0, 0.75),
                    ];
                }
            }
        }
        return runtime_rt_lighting;
    }
    let scene_prepare_voxel_fallback =
        scene_prepare_voxel_fallback_rt_lighting(frame, source, scene_prepare_resources);
    if scene_driven_frame {
        return scene_prepare_voxel_fallback.unwrap_or([0.0; 4]);
    }

    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;
    if scheduled_trace_region_ids.is_empty() {
        return scene_prepare_voxel_fallback.unwrap_or([0.0; 4]);
    }
    if runtime_parent_topology_is_authoritative(frame) {
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
        for region in scheduled_trace_region_ids
            .iter()
            .filter_map(|region_id| trace_regions_by_id.get(region_id))
        {
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

pub(super) fn current_rt_lighting_surface_cache_proxy_rgb_and_support(
    frame: &ViewportRenderFrame,
    source: &RenderHybridGiProbe,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> Option<([f32; 3], f32)> {
    current_rt_lighting_surface_cache_proxy_rgb_support_and_quality(
        frame,
        source,
        scene_prepare_resources,
    )
    .map(|(rgb, support, _)| (rgb, support))
}

pub(super) fn current_rt_lighting_surface_cache_proxy_rgb_support_and_quality(
    frame: &ViewportRenderFrame,
    source: &RenderHybridGiProbe,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> Option<([f32; 3], f32, f32)> {
    let Some(scene_prepare) = frame.hybrid_gi_scene_prepare.as_ref() else {
        return None;
    };

    let prepare = frame.hybrid_gi_prepare.as_ref();
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
    let exact_runtime_includes_scene_truth = frame
        .hybrid_gi_resolve_runtime
        .as_ref()
        .map(|runtime| runtime.hierarchy_rt_lighting_includes_scene_truth(source.probe_id))
        .unwrap_or(false);
    let exact_scene_truth_runtime_rt_lighting =
        exact_runtime_rt_lighting.filter(|_| exact_runtime_includes_scene_truth);
    let exact_scene_truth_runtime_rt_lighting_present =
        exact_scene_truth_runtime_rt_lighting.is_some();
    let inherited_scene_truth_runtime_rt_lighting =
        (!exact_scene_truth_runtime_rt_lighting_present)
            .then(|| {
                gather_runtime_parent_chain_rgb_without_depth_falloff(
                    frame,
                    source.probe_id,
                    |runtime, ancestor_probe_id| {
                        if !runtime.hierarchy_rt_lighting_includes_scene_truth(ancestor_probe_id) {
                            return None;
                        }

                        runtime_rt_lighting_lineage_source(runtime, ancestor_probe_id)
                    },
                )
            })
            .flatten()
            .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let descendant_scene_truth_runtime_rt_lighting =
        (!exact_scene_truth_runtime_rt_lighting_present)
            .then(|| {
                gather_runtime_descendant_chain_rgb_without_depth_falloff(
                    frame,
                    source.probe_id,
                    |runtime, descendant_probe_id| {
                        if !runtime.hierarchy_rt_lighting_includes_scene_truth(descendant_probe_id)
                        {
                            return None;
                        }

                        runtime_rt_lighting_lineage_source(runtime, descendant_probe_id)
                    },
                )
            })
            .flatten()
            .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let scene_truth_runtime_rt_lighting = blend_runtime_rgb_lineage_sources(
        exact_scene_truth_runtime_rt_lighting,
        inherited_scene_truth_runtime_rt_lighting,
        descendant_scene_truth_runtime_rt_lighting,
    );
    if scene_truth_runtime_rt_lighting.is_some() {
        return None;
    }
    let exact_continuation_runtime_rt_lighting =
        exact_runtime_rt_lighting.filter(|_| !exact_runtime_includes_scene_truth);
    let inherited_continuation_runtime_rt_lighting =
        (!exact_scene_truth_runtime_rt_lighting_present)
            .then(|| {
                gather_runtime_parent_chain_rgb(
                    frame,
                    source.probe_id,
                    |runtime, ancestor_probe_id| {
                        if runtime.hierarchy_rt_lighting_includes_scene_truth(ancestor_probe_id) {
                            return None;
                        }

                        runtime_rt_lighting_lineage_source(runtime, ancestor_probe_id)
                    },
                )
            })
            .flatten()
            .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let descendant_continuation_runtime_rt_lighting =
        (!exact_scene_truth_runtime_rt_lighting_present)
            .then(|| {
                gather_runtime_descendant_chain_rgb(
                    frame,
                    source.probe_id,
                    |runtime, descendant_probe_id| {
                        if runtime.hierarchy_rt_lighting_includes_scene_truth(descendant_probe_id) {
                            return None;
                        }

                        runtime_rt_lighting_lineage_source(runtime, descendant_probe_id)
                    },
                )
            })
            .flatten()
            .filter(|runtime_rt_lighting| runtime_rt_lighting[3] > f32::EPSILON);
    let continuation_runtime_rt_lighting = blend_runtime_rgb_lineage_sources(
        exact_continuation_runtime_rt_lighting,
        inherited_continuation_runtime_rt_lighting,
        descendant_continuation_runtime_rt_lighting,
    );
    if continuation_runtime_rt_lighting.is_none() {
        return None;
    }

    current_scene_prepare_rt_lighting_surface_cache_proxy_rgb_support_and_quality(
        scene_prepare,
        source,
        scene_prepare_resources,
    )
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
            scene_prepare,
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

fn current_scene_prepare_rt_lighting_surface_cache_proxy_rgb_support_and_quality(
    scene_prepare: &crate::graphics::types::HybridGiScenePrepareFrame,
    source: &RenderHybridGiProbe,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> Option<([f32; 3], f32, f32)> {
    let clipmaps_by_id = scene_prepare
        .voxel_clipmaps
        .iter()
        .map(|clipmap| (clipmap.clipmap_id, clipmap))
        .collect::<BTreeMap<_, _>>();
    let mut total_cell_support = 0.0_f32;
    let mut surface_cache_weighted_rgb = [0.0_f32; 3];
    let mut surface_cache_support = 0.0_f32;
    let mut weighted_confidence_quality = 0.0_f32;

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

        total_cell_support += support;
        if cell.radiance_present {
            continue;
        }
        if scene_prepare_voxel_cell_resource_rgb(scene_prepare_resources, cell).is_some() {
            continue;
        }
        let Some((owner_rgb, confidence_quality)) =
            scene_prepare_voxel_owner_card_capture_rgb_and_quality(
                scene_prepare,
                scene_prepare_resources,
                cell,
            )
        else {
            continue;
        };
        surface_cache_weighted_rgb[0] += owner_rgb[0] * support;
        surface_cache_weighted_rgb[1] += owner_rgb[1] * support;
        surface_cache_weighted_rgb[2] += owner_rgb[2] * support;
        surface_cache_support += support;
        weighted_confidence_quality += confidence_quality * support;
    }
    if total_cell_support > f32::EPSILON {
        if surface_cache_support <= f32::EPSILON {
            return None;
        }

        return Some((
            [
                surface_cache_weighted_rgb[0] / surface_cache_support,
                surface_cache_weighted_rgb[1] / surface_cache_support,
                surface_cache_weighted_rgb[2] / surface_cache_support,
            ],
            surface_cache_support,
            (weighted_confidence_quality / surface_cache_support).clamp(0.0, 1.0),
        ));
    }

    let total_clipmap_support =
        scene_prepare
            .voxel_clipmaps
            .iter()
            .fold(0.0_f32, |total_support, clipmap| {
                total_support
                    + scene_prepare_voxel_clipmap_support(
                        source.position,
                        source.radius,
                        clipmap.center,
                        clipmap.half_extent,
                    )
            });
    if total_clipmap_support > f32::EPSILON {
        return None;
    }

    scene_prepare_surface_cache_fallback_rgb_support_and_quality(
        scene_prepare,
        source.position,
        source.radius,
        scene_prepare_resources,
    )
    .filter(|(_, support, _)| *support > f32::EPSILON)
}

fn scene_prepare_voxel_clipmap_fallback_rt_lighting(
    scene_prepare: &crate::graphics::types::HybridGiScenePrepareFrame,
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
        return scene_prepare_surface_cache_fallback_rgb_and_support(
            scene_prepare,
            source.position,
            source.radius,
            scene_prepare_resources,
        )
        .map(|(rgb, support)| [rgb[0], rgb[1], rgb[2], (support * 0.58).clamp(0.18, 0.62)]);
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
    scene_prepare_voxel_owner_card_capture_rgb_and_quality(
        scene_prepare,
        scene_prepare_resources,
        cell,
    )
    .map(|(rgb, _)| rgb)
}

fn scene_prepare_voxel_owner_card_capture_rgb_and_quality(
    scene_prepare: &crate::graphics::types::HybridGiScenePrepareFrame,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
    cell: &crate::graphics::types::HybridGiPrepareVoxelCell,
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

fn runtime_rt_lighting_lineage_source(
    runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> Option<([f32; 3], f32)> {
    runtime_rt_lighting_packed_or_legacy_source(runtime, probe_id)
        .map(|source| ([source[0], source[1], source[2]], source[3]))
}

fn runtime_rt_lighting_packed_or_legacy_source(
    runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> Option<[f32; 4]> {
    if let Some(hierarchy_rt_lighting) = runtime
        .hierarchy_rt_lighting(probe_id)
        .filter(|hierarchy_rt_lighting| hierarchy_rt_lighting[3] > f32::EPSILON)
    {
        return Some(hierarchy_rt_lighting);
    }

    runtime
        .probe_rt_lighting_rgb
        .get(&probe_id)
        .copied()
        .and_then(|rgb| {
            let support =
                runtime_resolve_weight_support(runtime.hierarchy_resolve_weight(probe_id));
            (support > f32::EPSILON).then_some([
                rgb[0] as f32 / 255.0,
                rgb[1] as f32 / 255.0,
                rgb[2] as f32 / 255.0,
                support,
            ])
        })
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
    use std::collections::{BTreeMap, BTreeSet};

    use super::super::super::super::super::constants::MAX_HYBRID_GI_TRACE_REGIONS;
    use super::{
        hybrid_gi_hierarchy_rt_lighting,
        hybrid_gi_hierarchy_rt_lighting_with_scene_prepare_resources,
    };
    use crate::core::framework::render::{
        FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract, RenderHybridGiExtract,
        RenderHybridGiProbe, RenderHybridGiTraceRegion, RenderOverlayExtract,
        RenderSceneGeometryExtract, RenderSceneSnapshot, RenderWorldSnapshotHandle,
        ViewportCameraSnapshot,
    };
    use crate::core::math::{UVec2, Vec3, Vec4};
    use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
    use crate::graphics::types::{
        HybridGiPrepareCardCaptureRequest, HybridGiPrepareFrame, HybridGiPrepareProbe,
        HybridGiPrepareSurfaceCachePageContent, HybridGiPrepareVoxelCell,
        HybridGiPrepareVoxelClipmap, HybridGiResolveRuntime, HybridGiScenePrepareFrame,
        ViewportRenderFrame,
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
    fn exact_runtime_rt_lighting_blends_current_surface_cache_truth_when_trace_schedule_is_empty() {
        let direct_runtime_rgb = [120, 120, 120];
        let warm = scene_prepare_rt_lighting_with_exact_runtime(
            HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.6,
                    atlas_sample_rgba: [224, 112, 64, 255],
                    capture_sample_rgba: [240, 96, 48, 255],
                }],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            },
            direct_runtime_rgb,
        );
        let cool = scene_prepare_rt_lighting_with_exact_runtime(
            HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.6,
                    atlas_sample_rgba: [64, 112, 224, 255],
                    capture_sample_rgba: [48, 96, 240, 255],
                }],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            },
            direct_runtime_rgb,
        );

        assert!(
            warm[0] > cool[0] + 0.12,
            "expected stale exact runtime RT lighting to keep blending with current warm surface-cache truth when there is no current trace schedule, instead of flattening both frames back to the same runtime-only color; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            cool[2] > warm[2] + 0.12,
            "expected stale exact runtime RT lighting to keep blending with current cool surface-cache truth when there is no current trace schedule, instead of flattening both frames back to the same runtime-only color; warm={warm:?}, cool={cool:?}"
        );
    }

    #[test]
    fn exact_runtime_rt_lighting_skips_scene_prepare_reblend_when_runtime_source_is_already_scene_driven(
    ) {
        let exact_runtime = HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58);
        let scene_prepare = HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 11,
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.6,
                atlas_sample_rgba: [64, 112, 224, 255],
                capture_sample_rgba: [48, 96, 240, 255],
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        };

        let scene_driven = scene_prepare_rt_lighting_with_exact_runtime_and_scene_driven_flag(
            scene_prepare.clone(),
            exact_runtime,
            true,
        );
        let reblended = scene_prepare_rt_lighting_with_exact_runtime_and_scene_driven_flag(
            scene_prepare,
            exact_runtime,
            false,
        );

        assert!(
            scene_driven[0] > reblended[0] + 0.08,
            "expected renderer-side hierarchy RT lighting to trust a runtime source that already includes current scene truth instead of blending the same surface-cache signal a second time; scene_driven={scene_driven:?}, reblended={reblended:?}"
        );
        assert!(
            reblended[2] > scene_driven[2] + 0.08,
            "expected unflagged runtime RT lighting to keep drifting toward the cool scene-prepare page while the scene-driven runtime source stays closer to its authored warm color; scene_driven={scene_driven:?}, reblended={reblended:?}"
        );
    }

    #[test]
    fn scene_driven_exact_runtime_rt_lighting_ignores_descendant_lineage_tint() {
        let exact_runtime = HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58);
        let warm = scene_prepare_rt_lighting_with_scene_driven_exact_and_descendant(
            exact_runtime,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.9, 0.32, 0.18], 0.66),
        );
        let cool = scene_prepare_rt_lighting_with_scene_driven_exact_and_descendant(
            exact_runtime,
            HybridGiResolveRuntime::pack_rgb_and_weight([0.18, 0.32, 0.9], 0.66),
        );

        assert!(
            (warm[0] - cool[0]).abs() < 0.03,
            "expected scene-driven exact runtime RT lighting to stay anchored to current exact scene truth instead of drifting with descendant lineage tint; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            (warm[2] - cool[2]).abs() < 0.03,
            "expected scene-driven exact runtime RT lighting to keep blue output stable while descendant lineage tint changes; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            warm[0] > warm[2] + 0.35,
            "expected scene-driven exact runtime RT lighting to keep its authored warm bias after descendant lineage tint changes; warm={warm:?}"
        );
    }

    #[test]
    fn scene_driven_lineage_runtime_rt_lighting_ignores_scene_prepare_surface_cache_tint() {
        let inherited_warm =
            scene_prepare_rt_lighting_with_scene_driven_lineage_and_surface_cache_page(
                true,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
                [240, 96, 48, 255],
            );
        let inherited_cool =
            scene_prepare_rt_lighting_with_scene_driven_lineage_and_surface_cache_page(
                true,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
                [48, 96, 240, 255],
            );
        let descendant_warm =
            scene_prepare_rt_lighting_with_scene_driven_lineage_and_surface_cache_page(
                false,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
                [240, 96, 48, 255],
            );
        let descendant_cool =
            scene_prepare_rt_lighting_with_scene_driven_lineage_and_surface_cache_page(
                false,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.84, 0.36, 0.18], 0.58),
                [48, 96, 240, 255],
            );

        assert!(
            (inherited_warm[0] - inherited_cool[0]).abs() < 0.03
                && (inherited_warm[2] - inherited_cool[2]).abs() < 0.03,
            "expected inherited scene-driven runtime RT lighting to stay anchored to lineage scene truth instead of drifting with current scene_prepare surface-cache tint; inherited_warm={inherited_warm:?}, inherited_cool={inherited_cool:?}"
        );
        assert!(
            inherited_warm[0] > inherited_warm[2] + 0.35,
            "expected inherited scene-driven runtime RT lighting to keep its authored warm bias after scene_prepare page tint changes; inherited_warm={inherited_warm:?}"
        );
        assert!(
            (descendant_warm[0] - descendant_cool[0]).abs() < 0.03
                && (descendant_warm[2] - descendant_cool[2]).abs() < 0.03,
            "expected descendant scene-driven runtime RT lighting to stay anchored to lineage scene truth instead of drifting with current scene_prepare surface-cache tint; descendant_warm={descendant_warm:?}, descendant_cool={descendant_cool:?}"
        );
        assert!(
            descendant_warm[0] > descendant_warm[2] + 0.35,
            "expected descendant scene-driven runtime RT lighting to keep its authored warm bias after scene_prepare page tint changes; descendant_warm={descendant_warm:?}"
        );
    }

    #[test]
    fn scene_driven_inherited_legacy_probe_rt_lighting_uses_legacy_when_packed_hierarchy_rt_is_zero(
    ) {
        let inherited_rt_lighting =
            inherited_legacy_probe_rt_lighting_with_zero_packed_hierarchy_rt();

        assert!(
            inherited_rt_lighting[0] > 0.45,
            "expected inherited legacy RT lighting to remain visible when the packed hierarchy RT scene-truth source has zero support; inherited_rt_lighting={inherited_rt_lighting:?}"
        );
        assert!(
            inherited_rt_lighting[3] > 0.1,
            "expected inherited legacy RT lighting resolve weight to provide support when packed hierarchy RT is zero-weight; inherited_rt_lighting={inherited_rt_lighting:?}"
        );
    }

    #[test]
    fn legacy_trace_region_inheritance_counts_duplicate_scheduled_live_payload_once() {
        let single = inherited_trace_region_rt_lighting_with_scheduled_region_ids(vec![40]);
        let duplicate = inherited_trace_region_rt_lighting_with_scheduled_region_ids(vec![40, 40]);

        assert!(
            (single[3] - duplicate[3]).abs() < f32::EPSILON,
            "expected duplicate scheduled ids for the same live RenderHybridGiTraceRegion payload not to inflate inherited RT lighting support; single={single:?}, duplicate={duplicate:?}"
        );
        assert!(
            single[3] > 0.0,
            "expected the live trace payload to remain visible after scheduled-id de-duplication; single={single:?}"
        );
    }

    #[test]
    fn legacy_trace_region_inheritance_respects_live_payload_region_limit() {
        let rt_lighting = inherited_trace_region_rt_lighting_with_budget_excess_tail_payload();

        assert!(
            rt_lighting[3] <= f32::EPSILON,
            "expected legacy trace-region inheritance to respect the same live-payload GPU budget as trace-region encoding instead of reading a 17th scheduled payload; rt_lighting={rt_lighting:?}"
        );
    }

    #[test]
    fn scene_prepare_voxel_cell_resource_samples_override_spatial_fallback_when_runtime_authority_is_absent(
    ) {
        let scene_prepare = HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
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
            surface_cache_page_contents: Vec::new(),
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
            surface_cache_page_contents: Vec::new(),
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
            surface_cache_page_contents: Vec::new(),
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
            surface_cache_page_contents: Vec::new(),
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

    #[test]
    fn scene_prepare_card_capture_request_falls_back_to_atlas_resource_sample_when_capture_resource_sample_is_absent(
    ) {
        let scene_prepare = HybridGiScenePrepareFrame {
            card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
                card_id: 11,
                page_id: 22,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::new(20.0, 20.0, 20.0),
                bounds_radius: 0.25,
            }],
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 4.0,
            }],
            voxel_cells: runtime_owner_voxel_cells(false),
        };

        let warm = scene_prepare_rt_lighting_with_resources(
            scene_prepare.clone(),
            scene_prepare_resources_snapshot_with_surface_cache_samples(
                vec![(3, [224, 112, 64, 255])],
                vec![(4, [0, 0, 0, 0])],
            ),
        );
        let cool = scene_prepare_rt_lighting_with_resources(
            scene_prepare,
            scene_prepare_resources_snapshot_with_surface_cache_samples(
                vec![(3, [64, 112, 224, 255])],
                vec![(4, [0, 0, 0, 0])],
            ),
        );

        assert!(
            warm[0] > cool[0] + 0.2,
            "expected absent capture-side resource samples to fall back to atlas-side truth instead of flattening request-owned voxel fallback to black; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            cool[2] > warm[2] + 0.2,
            "expected atlas-side resource samples to stay color-authoritative when capture-side resource samples are absent; warm={warm:?}, cool={cool:?}"
        );
    }

    #[test]
    fn scene_prepare_card_capture_request_ignores_absent_resource_samples_and_keeps_synthesized_fallback(
    ) {
        let scene_prepare = HybridGiScenePrepareFrame {
            card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
                card_id: 11,
                page_id: 22,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::new(20.0, 20.0, 20.0),
                bounds_radius: 0.25,
            }],
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 4.0,
            }],
            voxel_cells: runtime_owner_voxel_cells(false),
        };

        let baseline = scene_prepare_rt_lighting_with_resources(
            scene_prepare.clone(),
            scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
        );
        let absent = scene_prepare_rt_lighting_with_resources(
            scene_prepare,
            scene_prepare_resources_snapshot_with_surface_cache_samples(
                vec![(3, [0, 0, 0, 0])],
                vec![(4, [0, 0, 0, 0])],
            ),
        );

        assert_eq!(
            absent,
            baseline,
            "expected zero-alpha request resource samples to be treated as absent and fall through to the synthesized request fallback instead of becoming a false black authority; baseline={baseline:?}, absent={absent:?}"
        );
    }

    #[test]
    fn scene_prepare_persisted_surface_cache_page_samples_override_spatial_fallback_when_owner_request_is_absent(
    ) {
        let warm = scene_prepare_rt_lighting_with_resources(
            HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.25,
                    atlas_sample_rgba: [224, 112, 64, 255],
                    capture_sample_rgba: [240, 96, 48, 255],
                }],
                voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                    clipmap_id: 7,
                    center: Vec3::ZERO,
                    half_extent: 4.0,
                }],
                voxel_cells: runtime_owner_voxel_cells(false),
            },
            scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
        );
        let cool = scene_prepare_rt_lighting_with_resources(
            HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.25,
                    atlas_sample_rgba: [64, 112, 224, 255],
                    capture_sample_rgba: [48, 96, 240, 255],
                }],
                voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                    clipmap_id: 7,
                    center: Vec3::ZERO,
                    half_extent: 4.0,
                }],
                voxel_cells: runtime_owner_voxel_cells(false),
            },
            scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
        );

        assert!(
            warm[0] > cool[0] + 0.2,
            "expected clean-frame persisted surface-cache samples to drive owner-card voxel fallback when no current card-capture request exists; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            cool[2] > warm[2] + 0.2,
            "expected clean-frame persisted surface-cache samples to preserve blue authority on owner-card voxel fallback when no current card-capture request exists; warm={warm:?}, cool={cool:?}"
        );
    }

    #[test]
    fn scene_prepare_persisted_surface_cache_page_samples_provide_spatial_fallback_without_runtime_voxel_support(
    ) {
        let warm = scene_prepare_rt_lighting_with_resources(
            HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.6,
                    atlas_sample_rgba: [224, 112, 64, 255],
                    capture_sample_rgba: [240, 96, 48, 255],
                }],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            },
            scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
        );
        let cool = scene_prepare_rt_lighting_with_resources(
            HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.6,
                    atlas_sample_rgba: [64, 112, 224, 255],
                    capture_sample_rgba: [48, 96, 240, 255],
                }],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            },
            scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
        );

        assert!(
            warm[3] > 0.18,
            "expected persisted surface-cache pages to remain a valid GI fallback source even when runtime voxel support is absent; warm={warm:?}"
        );
        assert!(
            warm[0] > cool[0] + 0.2,
            "expected nearby persisted surface-cache page samples to provide warm authority without runtime voxel support; warm={warm:?}, cool={cool:?}"
        );
        assert!(
            cool[2] > warm[2] + 0.2,
            "expected nearby persisted surface-cache page samples to preserve blue authority without runtime voxel support; warm={warm:?}, cool={cool:?}"
        );
    }

    #[test]
    fn scene_prepare_absent_surface_cache_page_samples_do_not_create_false_black_fallback_without_runtime_voxel_support(
    ) {
        let absent = scene_prepare_rt_lighting_with_resources(
            HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.6,
                    atlas_sample_rgba: [0, 0, 0, 0],
                    capture_sample_rgba: [0, 0, 0, 0],
                }],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            },
            scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
        );
        let explicit_black = scene_prepare_rt_lighting_with_resources(
            HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.6,
                    atlas_sample_rgba: [0, 0, 0, 255],
                    capture_sample_rgba: [0, 0, 0, 255],
                }],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            },
            scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new()),
        );

        assert!(
            absent[3] <= f32::EPSILON,
            "expected truly absent persisted surface-cache samples to produce no fallback support instead of a false black GI source; absent={absent:?}"
        );
        assert!(
            explicit_black[3] > 0.18,
            "expected explicit-black persisted surface-cache samples to remain a valid GI fallback source; explicit_black={explicit_black:?}"
        );
        assert!(
            rgb_energy(explicit_black) < 0.05,
            "expected explicit-black persisted surface-cache samples to stay black instead of reintroducing a color heuristic; explicit_black={explicit_black:?}"
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

    fn scene_prepare_rt_lighting_with_exact_runtime(
        scene_prepare: HybridGiScenePrepareFrame,
        direct_runtime_rgb: [u8; 3],
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
            card_budget: 1,
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
            .with_hybrid_gi_scene_prepare(Some(scene_prepare))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_rt_lighting_rgb: BTreeMap::from([(probe.probe_id, direct_runtime_rgb)]),
                ..Default::default()
            }));

        hybrid_gi_hierarchy_rt_lighting(&frame, &probe)
    }

    fn scene_prepare_rt_lighting_with_exact_runtime_and_scene_driven_flag(
        scene_prepare: HybridGiScenePrepareFrame,
        exact_runtime: [u8; 4],
        scene_driven: bool,
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
            card_budget: 1,
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
            .with_hybrid_gi_scene_prepare(Some(scene_prepare))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_hierarchy_rt_lighting_rgb_and_weight: BTreeMap::from([(
                    probe.probe_id,
                    exact_runtime,
                )]),
                probe_scene_driven_hierarchy_rt_lighting_ids: scene_driven
                    .then(|| BTreeSet::from([probe.probe_id]))
                    .unwrap_or_default(),
                ..Default::default()
            }));

        hybrid_gi_hierarchy_rt_lighting(&frame, &probe)
    }

    fn scene_prepare_rt_lighting_with_scene_driven_exact_and_descendant(
        exact_runtime: [u8; 4],
        descendant_runtime: [u8; 4],
    ) -> [f32; 4] {
        let probe = RenderHybridGiProbe {
            probe_id: 200,
            resident: true,
            ray_budget: 128,
            radius: 1.8,
            ..Default::default()
        };
        let descendant_probe = RenderHybridGiProbe {
            probe_id: 260,
            parent_probe_id: Some(probe.probe_id),
            resident: false,
            ray_budget: 88,
            radius: 1.2,
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
            trace_budget: 0,
            card_budget: 1,
            voxel_budget: 1,
            probes: vec![probe, descendant_probe],
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
            .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: Vec::new(),
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_hierarchy_rt_lighting_rgb_and_weight: BTreeMap::from([
                    (probe.probe_id, exact_runtime),
                    (descendant_probe.probe_id, descendant_runtime),
                ]),
                probe_scene_driven_hierarchy_rt_lighting_ids: BTreeSet::from([
                    probe.probe_id,
                    descendant_probe.probe_id,
                ]),
                ..Default::default()
            }));

        hybrid_gi_hierarchy_rt_lighting(&frame, &probe)
    }

    fn scene_prepare_rt_lighting_with_scene_driven_lineage_and_surface_cache_page(
        scene_truth_on_ancestor: bool,
        lineage_runtime: [u8; 4],
        page_capture_sample_rgba: [u8; 4],
    ) -> [f32; 4] {
        let parent_probe = RenderHybridGiProbe {
            probe_id: 100,
            resident: true,
            ray_budget: 128,
            radius: 1.8,
            ..Default::default()
        };
        let child_probe = RenderHybridGiProbe {
            probe_id: 200,
            parent_probe_id: Some(parent_probe.probe_id),
            resident: true,
            ray_budget: 96,
            radius: 1.2,
            ..Default::default()
        };
        let encoded_probe = if scene_truth_on_ancestor {
            child_probe
        } else {
            parent_probe
        };
        let runtime_probe_id = if scene_truth_on_ancestor {
            parent_probe.probe_id
        } else {
            child_probe.probe_id
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
            trace_budget: 0,
            card_budget: 1,
            voxel_budget: 1,
            probes: vec![parent_probe, child_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: encoded_probe.probe_id,
                    slot: 0,
                    ray_budget: encoded_probe.ray_budget,
                    irradiance_rgb: [112, 112, 112],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                card_capture_requests: Vec::new(),
                surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                    page_id: 11,
                    owner_card_id: 11,
                    atlas_slot_id: 3,
                    capture_slot_id: 4,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.6,
                    atlas_sample_rgba: page_capture_sample_rgba,
                    capture_sample_rgba: page_capture_sample_rgba,
                }],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_hierarchy_rt_lighting_rgb_and_weight: BTreeMap::from([(
                    runtime_probe_id,
                    lineage_runtime,
                )]),
                probe_scene_driven_hierarchy_rt_lighting_ids: BTreeSet::from([runtime_probe_id]),
                ..Default::default()
            }));

        hybrid_gi_hierarchy_rt_lighting(&frame, &encoded_probe)
    }

    fn inherited_legacy_probe_rt_lighting_with_zero_packed_hierarchy_rt() -> [f32; 4] {
        let parent_probe = RenderHybridGiProbe {
            probe_id: 100,
            resident: true,
            ray_budget: 128,
            radius: 1.8,
            ..Default::default()
        };
        let child_probe = RenderHybridGiProbe {
            probe_id: 200,
            parent_probe_id: Some(parent_probe.probe_id),
            resident: true,
            ray_budget: 96,
            radius: 1.2,
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
            trace_budget: 0,
            card_budget: 0,
            voxel_budget: 0,
            probes: vec![parent_probe, child_probe],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: child_probe.probe_id,
                    slot: 0,
                    ray_budget: child_probe.ray_budget,
                    irradiance_rgb: [112, 112, 112],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids: Vec::new(),
                evictable_probe_ids: Vec::new(),
            }))
            .with_hybrid_gi_resolve_runtime(Some(HybridGiResolveRuntime {
                probe_rt_lighting_rgb: BTreeMap::from([(parent_probe.probe_id, [240, 96, 48])]),
                probe_hierarchy_resolve_weight_q8: BTreeMap::from([(
                    parent_probe.probe_id,
                    HybridGiResolveRuntime::pack_resolve_weight_q8(2.0),
                )]),
                probe_hierarchy_rt_lighting_rgb_and_weight: BTreeMap::from([(
                    parent_probe.probe_id,
                    HybridGiResolveRuntime::pack_rgb_and_weight([0.05, 0.05, 0.05], 0.0),
                )]),
                probe_scene_driven_hierarchy_rt_lighting_ids: BTreeSet::from([
                    parent_probe.probe_id
                ]),
                ..Default::default()
            }));

        hybrid_gi_hierarchy_rt_lighting(&frame, &child_probe)
    }

    fn inherited_trace_region_rt_lighting_with_scheduled_region_ids(
        scheduled_trace_region_ids: Vec<u32>,
    ) -> [f32; 4] {
        let parent_probe = RenderHybridGiProbe {
            probe_id: 100,
            resident: true,
            ray_budget: 128,
            radius: 1.8,
            ..Default::default()
        };
        let child_probe = RenderHybridGiProbe {
            probe_id: 200,
            parent_probe_id: Some(parent_probe.probe_id),
            resident: true,
            ray_budget: 96,
            radius: 1.2,
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
            card_budget: 0,
            voxel_budget: 0,
            probes: vec![parent_probe, child_probe],
            trace_regions: vec![RenderHybridGiTraceRegion {
                entity: 40,
                region_id: 40,
                bounds_center: Vec3::ZERO,
                bounds_radius: 2.0,
                screen_coverage: 1.0,
                rt_lighting_rgb: [240, 96, 48],
            }],
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: parent_probe.probe_id,
                    slot: 0,
                    ray_budget: parent_probe.ray_budget,
                    irradiance_rgb: [112, 112, 112],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids,
                evictable_probe_ids: Vec::new(),
            }));

        hybrid_gi_hierarchy_rt_lighting(&frame, &child_probe)
    }

    fn inherited_trace_region_rt_lighting_with_budget_excess_tail_payload() -> [f32; 4] {
        let parent_probe = RenderHybridGiProbe {
            probe_id: 100,
            resident: true,
            ray_budget: 128,
            radius: 1.8,
            ..Default::default()
        };
        let child_probe = RenderHybridGiProbe {
            probe_id: 200,
            parent_probe_id: Some(parent_probe.probe_id),
            resident: true,
            ray_budget: 96,
            radius: 1.2,
            ..Default::default()
        };
        let live_tail_region_id = 40;
        let budget_filler_region_ids = (0..MAX_HYBRID_GI_TRACE_REGIONS)
            .map(|index| 10_000 + index as u32)
            .collect::<Vec<_>>();
        let mut scheduled_trace_region_ids = budget_filler_region_ids.clone();
        scheduled_trace_region_ids.push(live_tail_region_id);
        let mut trace_regions = budget_filler_region_ids
            .iter()
            .copied()
            .map(|region_id| RenderHybridGiTraceRegion {
                entity: u64::from(region_id),
                region_id,
                bounds_center: Vec3::new(10_000.0, 0.0, 0.0),
                bounds_radius: 0.5,
                screen_coverage: 1.0,
                rt_lighting_rgb: [240, 96, 48],
            })
            .collect::<Vec<_>>();
        trace_regions.push(RenderHybridGiTraceRegion {
            entity: u64::from(live_tail_region_id),
            region_id: live_tail_region_id,
            bounds_center: Vec3::ZERO,
            bounds_radius: 2.0,
            screen_coverage: 1.0,
            rt_lighting_rgb: [240, 96, 48],
        });

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
            card_budget: 0,
            voxel_budget: 0,
            probes: vec![parent_probe, child_probe],
            trace_regions,
            ..Default::default()
        });

        let frame = ViewportRenderFrame::from_extract(extract, UVec2::new(32, 32))
            .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                resident_probes: vec![HybridGiPrepareProbe {
                    probe_id: parent_probe.probe_id,
                    slot: 0,
                    ray_budget: parent_probe.ray_budget,
                    irradiance_rgb: [112, 112, 112],
                }],
                pending_updates: Vec::new(),
                scheduled_trace_region_ids,
                evictable_probe_ids: Vec::new(),
            }));

        hybrid_gi_hierarchy_rt_lighting(&frame, &child_probe)
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

    fn scene_prepare_resources_snapshot_with_surface_cache_samples(
        atlas_slot_rgba_samples: Vec<(u32, [u8; 4])>,
        capture_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    ) -> HybridGiScenePrepareResourcesSnapshot {
        let occupied_atlas_slots = atlas_slot_rgba_samples
            .iter()
            .map(|(slot_id, _)| *slot_id)
            .collect();
        let occupied_capture_slots = capture_slot_rgba_samples
            .iter()
            .map(|(slot_id, _)| *slot_id)
            .collect();
        HybridGiScenePrepareResourcesSnapshot {
            occupied_atlas_slots,
            occupied_capture_slots,
            atlas_slot_rgba_samples,
            capture_slot_rgba_samples,
            ..scene_prepare_resources_snapshot(Vec::new(), Vec::new(), Vec::new())
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
