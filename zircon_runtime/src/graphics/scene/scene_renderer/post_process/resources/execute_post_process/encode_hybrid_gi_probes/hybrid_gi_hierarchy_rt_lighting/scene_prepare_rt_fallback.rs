use std::collections::BTreeMap;

use crate::graphics::scene::scene_renderer::HybridGiScenePrepareResourcesSnapshot;
use crate::graphics::types::{
    hybrid_gi_voxel_clipmap_cell_center, HybridGiPrepareVoxelClipmap, HybridGiScenePrepareFrame,
    ViewportRenderFrame, HYBRID_GI_VOXEL_CLIPMAP_CELL_COUNT,
    HYBRID_GI_VOXEL_CLIPMAP_CELL_RESOLUTION,
};

use super::super::hybrid_gi_probe_source::HybridGiProbeSource;
use super::super::runtime_parent_chain::{
    blend_runtime_rgb_lineage_sources, gather_runtime_descendant_chain_rgb,
    gather_runtime_descendant_chain_rgb_without_depth_falloff, gather_runtime_parent_chain_rgb,
    gather_runtime_parent_chain_rgb_without_depth_falloff,
};
use super::super::scene_prepare_surface_cache_samples::{
    scene_prepare_surface_cache_fallback_rgb_and_support,
    scene_prepare_surface_cache_fallback_rgb_support_and_quality,
};
use super::runtime_rt_sources::{
    runtime_hierarchy_rt_lighting, runtime_rt_lighting_lineage_source,
};
use super::scene_prepare_voxel_samples::{
    scene_prepare_voxel_cell_base_rgb, scene_prepare_voxel_cell_resource_rgb,
    scene_prepare_voxel_cell_support, scene_prepare_voxel_clipmap_base_rgb,
    scene_prepare_voxel_clipmap_support, scene_prepare_voxel_owner_card_capture_rgb_and_quality,
};

const SCENE_PREPARE_VOXEL_FALLBACK_WEIGHT_SCALE: f32 = 0.6;

pub(in super::super) fn current_rt_lighting_surface_cache_proxy_rgb_and_support<
    S: HybridGiProbeSource + ?Sized,
>(
    frame: &ViewportRenderFrame,
    source: &S,
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> Option<([f32; 3], f32)> {
    current_rt_lighting_surface_cache_proxy_rgb_support_and_quality(
        frame,
        source,
        scene_prepare_resources,
    )
    .map(|(rgb, support, _)| (rgb, support))
}

pub(in super::super) fn current_rt_lighting_surface_cache_proxy_rgb_support_and_quality<
    S: HybridGiProbeSource + ?Sized,
>(
    frame: &ViewportRenderFrame,
    source: &S,
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
        .map(|runtime| runtime.hierarchy_rt_lighting_includes_scene_truth(source.probe_id()))
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
                    source.probe_id(),
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
                    source.probe_id(),
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
                    source.probe_id(),
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
                    source.probe_id(),
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

pub(super) fn scene_prepare_voxel_fallback_rt_lighting<S: HybridGiProbeSource + ?Sized>(
    frame: &ViewportRenderFrame,
    source: &S,
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
            source.position(),
            source.radius(),
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

fn current_scene_prepare_rt_lighting_surface_cache_proxy_rgb_support_and_quality<
    S: HybridGiProbeSource + ?Sized,
>(
    scene_prepare: &HybridGiScenePrepareFrame,
    source: &S,
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
            source.position(),
            source.radius(),
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
                        source.position(),
                        source.radius(),
                        clipmap.center,
                        clipmap.half_extent,
                    )
            });
    if total_clipmap_support > f32::EPSILON {
        return None;
    }

    scene_prepare_surface_cache_fallback_rgb_support_and_quality(
        scene_prepare,
        source.position(),
        source.radius(),
        scene_prepare_resources,
    )
    .filter(|(_, support, _)| *support > f32::EPSILON)
}

fn scene_prepare_voxel_clipmap_fallback_rt_lighting<S: HybridGiProbeSource + ?Sized>(
    scene_prepare: &HybridGiScenePrepareFrame,
    source: &S,
    voxel_clipmaps: &[HybridGiPrepareVoxelClipmap],
    scene_prepare_resources: Option<&HybridGiScenePrepareResourcesSnapshot>,
) -> Option<[f32; 4]> {
    let mut weighted_rgb = [0.0_f32; 3];
    let mut total_support = 0.0_f32;

    for clipmap in voxel_clipmaps {
        let support = scene_prepare_voxel_clipmap_support(
            source.position(),
            source.radius(),
            clipmap.center,
            clipmap.half_extent,
        );
        if support <= f32::EPSILON {
            continue;
        }

        let base_rgb = scene_prepare_voxel_clipmap_base_rgb(
            source.position(),
            clipmap,
            scene_prepare_resources,
        );
        weighted_rgb[0] += base_rgb[0] * support;
        weighted_rgb[1] += base_rgb[1] * support;
        weighted_rgb[2] += base_rgb[2] * support;
        total_support += support;
    }

    if total_support <= f32::EPSILON {
        return scene_prepare_surface_cache_fallback_rgb_and_support(
            scene_prepare,
            source.position(),
            source.radius(),
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
