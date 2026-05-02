use std::collections::BTreeSet;

use zircon_runtime::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderHybridGiExtract, RenderMeshSnapshot,
    RenderPointLightSnapshot, RenderSpotLightSnapshot,
};
use zircon_runtime::graphics::hybrid_gi_extract_sources::{
    enabled_hybrid_gi_extract, hybrid_gi_extract_uses_scene_representation_budget,
};

use crate::hybrid_gi::types::{
    HybridGiPrepareFrame, HybridGiResolveRuntime, HybridGiScenePrepareFrame,
};

use super::super::extract_scene_sources::extract_trace_region_ids;
use super::super::pending_probe_inputs::pending_probe_inputs;
use super::super::resident_probe_inputs::resident_probe_inputs;
use super::super::trace_region_inputs::trace_region_inputs;
use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;

fn persisted_surface_cache_page_has_present_sample(
    page_content: &crate::hybrid_gi::types::HybridGiPrepareSurfaceCachePageContent,
) -> bool {
    page_content.capture_sample_rgba[3] > 0 || page_content.atlas_sample_rgba[3] > 0
}

pub(super) fn collect_inputs(
    prepare: &HybridGiPrepareFrame,
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    extract: Option<&RenderHybridGiExtract>,
    scene_prepare: Option<&HybridGiScenePrepareFrame>,
    scene_meshes: &[RenderMeshSnapshot],
    directional_lights: &[RenderDirectionalLightSnapshot],
    point_lights: &[RenderPointLightSnapshot],
    spot_lights: &[RenderSpotLightSnapshot],
) -> HybridGiPrepareExecutionInputs {
    let scene_prepare_owns_trace_sources = extract_uses_scene_representation_budget(extract)
        || scene_prepare_has_resources(scene_prepare);
    let stripped_runtime_owns_trace_sources =
        scene_prepare.is_none() && runtime_has_scene_truth(resolve_runtime);
    let runtime_owns_trace_sources = resolve_runtime.is_some()
        && (scene_prepare_owns_trace_sources || stripped_runtime_owns_trace_sources);
    let extract_fallback_trace_sources = (resolve_runtime.is_none() && !runtime_owns_trace_sources)
        .then_some(extract)
        .flatten();
    let extract_backed_runtime_trace_region_ids = if runtime_owns_trace_sources {
        extract_trace_region_ids(extract)
    } else {
        BTreeSet::new()
    };
    let filtered_prepare = (!extract_backed_runtime_trace_region_ids.is_empty()).then(|| {
        let mut filtered_prepare = prepare.clone();
        filtered_prepare
            .scheduled_trace_region_ids
            .retain(|region_id| !extract_backed_runtime_trace_region_ids.contains(region_id));
        filtered_prepare
    });
    let prepare_for_trace_inputs = filtered_prepare.as_ref().unwrap_or(prepare);
    let cache_entries = prepare
        .resident_probes
        .iter()
        .map(|probe| [probe.probe_id, probe.slot])
        .collect::<Vec<_>>();
    let resident_probe_inputs = resident_probe_inputs(
        prepare_for_trace_inputs,
        resolve_runtime,
        extract,
        extract_fallback_trace_sources,
    );
    let pending_probe_inputs = pending_probe_inputs(
        prepare_for_trace_inputs,
        resolve_runtime,
        extract,
        extract_fallback_trace_sources,
    );
    let trace_region_inputs = trace_region_inputs(
        prepare_for_trace_inputs,
        resolve_runtime,
        extract_fallback_trace_sources,
    );
    let scene_card_capture_requests = scene_prepare
        .map(|prepare| prepare.card_capture_requests.clone())
        .unwrap_or_default();
    let scene_surface_cache_page_contents = scene_prepare
        .map(|prepare| prepare.surface_cache_page_contents.clone())
        .unwrap_or_default();
    let scene_card_capture_request_page_ids = scene_card_capture_requests
        .iter()
        .map(|request| request.page_id)
        .collect::<BTreeSet<_>>();
    let scene_card_capture_descriptor_count = scene_card_capture_requests.len()
        + scene_surface_cache_page_contents
            .iter()
            .filter(|page_content| {
                !scene_card_capture_request_page_ids.contains(&page_content.page_id)
                    && persisted_surface_cache_page_has_present_sample(page_content)
            })
            .count();
    let scene_voxel_clipmaps = scene_prepare
        .map(|prepare| prepare.voxel_clipmaps.clone())
        .unwrap_or_default();
    let scene_voxel_cells = scene_prepare
        .map(|prepare| prepare.voxel_cells.clone())
        .unwrap_or_default();

    HybridGiPrepareExecutionInputs {
        cache_word_count: cache_entries.len() * 2,
        completed_probe_word_count: pending_probe_inputs.len() + 1,
        completed_trace_word_count: trace_region_inputs.len() + 1,
        irradiance_word_count: 1
            + (resident_probe_inputs.len() + pending_probe_inputs.len()).max(1) * 2,
        trace_lighting_word_count: 1
            + (resident_probe_inputs.len() + pending_probe_inputs.len()).max(1) * 2,
        cache_entries,
        resident_probe_inputs,
        pending_probe_inputs,
        trace_region_inputs,
        scene_card_capture_requests,
        scene_surface_cache_page_contents,
        scene_card_capture_descriptor_count,
        scene_voxel_clipmaps,
        scene_voxel_cells,
        scene_meshes: scene_meshes.to_vec(),
        directional_lights: directional_lights.to_vec(),
        point_lights: point_lights.to_vec(),
        spot_lights: spot_lights.to_vec(),
    }
}

fn runtime_has_scene_truth(resolve_runtime: Option<&HybridGiResolveRuntime>) -> bool {
    let Some(runtime) = resolve_runtime else {
        return false;
    };

    runtime
        .scene_truth_irradiance_probe_ids()
        .any(|probe_id| runtime_probe_has_irradiance_scene_truth(runtime, probe_id))
        || runtime
            .scene_truth_rt_lighting_probe_ids()
            .any(|probe_id| runtime_probe_has_rt_lighting_scene_truth(runtime, probe_id))
}

fn extract_uses_scene_representation_budget(extract: Option<&RenderHybridGiExtract>) -> bool {
    enabled_hybrid_gi_extract(extract)
        .map(hybrid_gi_extract_uses_scene_representation_budget)
        .unwrap_or(false)
}

fn scene_prepare_has_resources(scene_prepare: Option<&HybridGiScenePrepareFrame>) -> bool {
    scene_prepare
        .map(|scene_prepare| {
            !scene_prepare.card_capture_requests.is_empty()
                || !scene_prepare.surface_cache_page_contents.is_empty()
                || !scene_prepare.voxel_clipmaps.is_empty()
                || !scene_prepare.voxel_cells.is_empty()
        })
        .unwrap_or(false)
}

fn runtime_probe_has_irradiance_scene_truth(
    runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> bool {
    runtime.hierarchy_irradiance_includes_scene_truth(probe_id)
        && runtime
            .hierarchy_irradiance(probe_id)
            .map(|source| source[3] > f32::EPSILON)
            .unwrap_or(false)
}

fn runtime_probe_has_rt_lighting_scene_truth(
    runtime: &HybridGiResolveRuntime,
    probe_id: u32,
) -> bool {
    runtime.hierarchy_rt_lighting_includes_scene_truth(probe_id)
        && (runtime
            .hierarchy_rt_lighting(probe_id)
            .map(|source| source[3] > f32::EPSILON)
            .unwrap_or(false)
            || runtime.has_probe_rt_lighting(probe_id))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::hybrid_gi::types::{
        HybridGiPrepareCardCaptureRequest, HybridGiPrepareFrame, HybridGiPrepareProbe,
        HybridGiPrepareSurfaceCachePageContent, HybridGiPrepareUpdateRequest,
        HybridGiPrepareVoxelCell, HybridGiPrepareVoxelClipmap, HybridGiResolveProbeSceneData,
        HybridGiResolveRuntime, HybridGiResolveTraceRegionSceneData, HybridGiScenePrepareFrame,
    };
    use zircon_runtime::core::framework::render::{
        RenderHybridGiExtract, RenderHybridGiProbe, RenderHybridGiTraceRegion,
    };
    use zircon_runtime::core::math::Vec3;

    use super::*;

    #[test]
    fn collect_inputs_preserves_scene_prepare_contract_for_renderer_consumption() {
        let prepare = HybridGiPrepareFrame::default();
        let scene_prepare = HybridGiScenePrepareFrame {
            card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
                card_id: 11,
                page_id: 22,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::new(1.0, 2.0, 3.0),
                bounds_radius: 0.5,
            }],
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 22,
                owner_card_id: 22,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::new(1.0, 2.0, 3.0),
                bounds_radius: 0.5,
                atlas_sample_rgba: [10, 20, 30, 255],
                capture_sample_rgba: [40, 50, 60, 255],
            }],
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::new(-4.0, 0.0, 2.0),
                half_extent: 16.0,
            }],
            voxel_cells: vec![HybridGiPrepareVoxelCell {
                clipmap_id: 7,
                cell_index: 21,
                occupancy_count: 2,
                dominant_card_id: 0,
                radiance_present: false,
                radiance_rgb: [0, 0, 0],
            }],
        };

        let inputs = collect_inputs(
            &prepare,
            None,
            None,
            Some(&scene_prepare),
            &[],
            &[],
            &[],
            &[],
        );

        assert_eq!(
            inputs.scene_card_capture_requests,
            scene_prepare.card_capture_requests
        );
        assert_eq!(
            inputs.scene_surface_cache_page_contents,
            scene_prepare.surface_cache_page_contents
        );
        assert_eq!(inputs.scene_card_capture_descriptor_count, 1);
        assert_eq!(inputs.scene_voxel_clipmaps, scene_prepare.voxel_clipmaps);
        assert_eq!(inputs.scene_voxel_cells, scene_prepare.voxel_cells);
        assert!(inputs.scene_meshes.is_empty());
        assert!(inputs.directional_lights.is_empty());
        assert!(inputs.point_lights.is_empty());
        assert!(inputs.spot_lights.is_empty());
    }

    #[test]
    fn collect_inputs_counts_clean_frame_persisted_surface_cache_pages_as_card_descriptors() {
        let inputs = collect_inputs(
            &HybridGiPrepareFrame::default(),
            None,
            None,
            Some(&HybridGiScenePrepareFrame {
                card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
                    card_id: 11,
                    page_id: 11,
                    atlas_slot_id: 0,
                    capture_slot_id: 0,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.5,
                }],
                surface_cache_page_contents: vec![
                    HybridGiPrepareSurfaceCachePageContent {
                        page_id: 11,
                        owner_card_id: 11,
                        atlas_slot_id: 0,
                        capture_slot_id: 0,
                        bounds_center: Vec3::ZERO,
                        bounds_radius: 0.5,
                        atlas_sample_rgba: [10, 20, 30, 255],
                        capture_sample_rgba: [40, 50, 60, 255],
                    },
                    HybridGiPrepareSurfaceCachePageContent {
                        page_id: 22,
                        owner_card_id: 22,
                        atlas_slot_id: 1,
                        capture_slot_id: 1,
                        bounds_center: Vec3::new(1.0, 0.0, 0.0),
                        bounds_radius: 0.75,
                        atlas_sample_rgba: [11, 21, 31, 255],
                        capture_sample_rgba: [41, 51, 61, 255],
                    },
                ],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }),
            &[],
            &[],
            &[],
            &[],
        );

        assert_eq!(
            inputs.scene_card_capture_descriptor_count,
            2,
            "expected clean-frame persisted page contents to stage an additional card descriptor when no current dirty card-capture request owns that resident page"
        );
    }

    #[test]
    fn collect_inputs_skips_absent_clean_frame_persisted_surface_cache_pages_when_counting_card_descriptors(
    ) {
        let inputs = collect_inputs(
            &HybridGiPrepareFrame::default(),
            None,
            None,
            Some(&HybridGiScenePrepareFrame {
                card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
                    card_id: 11,
                    page_id: 11,
                    atlas_slot_id: 0,
                    capture_slot_id: 0,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.5,
                }],
                surface_cache_page_contents: vec![
                    HybridGiPrepareSurfaceCachePageContent {
                        page_id: 11,
                        owner_card_id: 11,
                        atlas_slot_id: 0,
                        capture_slot_id: 0,
                        bounds_center: Vec3::ZERO,
                        bounds_radius: 0.5,
                        atlas_sample_rgba: [10, 20, 30, 255],
                        capture_sample_rgba: [40, 50, 60, 255],
                    },
                    HybridGiPrepareSurfaceCachePageContent {
                        page_id: 22,
                        owner_card_id: 22,
                        atlas_slot_id: 1,
                        capture_slot_id: 1,
                        bounds_center: Vec3::new(1.0, 0.0, 0.0),
                        bounds_radius: 0.75,
                        atlas_sample_rgba: [0, 0, 0, 0],
                        capture_sample_rgba: [0, 0, 0, 0],
                    },
                ],
                voxel_clipmaps: Vec::new(),
                voxel_cells: Vec::new(),
            }),
            &[],
            &[],
            &[],
            &[],
        );

        assert_eq!(
            inputs.scene_card_capture_descriptor_count,
            1,
            "expected absent persisted page samples to stay out of GPU scene-card descriptor count so missing authority does not stage a false black descriptor"
        );
    }

    #[test]
    fn collect_inputs_treats_disabled_legacy_trace_schedule_as_empty_for_scene_truth_skips() {
        let resident_probe_id = 300;
        let pending_probe_id = 400;
        let stale_region_id = 900;
        let prepare = HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: resident_probe_id,
                slot: 0,
                ray_budget: 64,
                irradiance_rgb: [0, 0, 0],
            }],
            pending_updates: vec![HybridGiPrepareUpdateRequest {
                probe_id: pending_probe_id,
                ray_budget: 64,
                generation: 1,
            }],
            scheduled_trace_region_ids: vec![stale_region_id],
            evictable_probe_ids: Vec::new(),
        };
        let runtime = HybridGiResolveRuntime::fixture()
            .with_probe_hierarchy_irradiance_rgb_and_weight(BTreeMap::from([
                (
                    resident_probe_id,
                    HybridGiResolveRuntime::pack_rgb_and_weight([0.1, 0.2, 0.3], 1.0),
                ),
                (
                    pending_probe_id,
                    HybridGiResolveRuntime::pack_rgb_and_weight([0.2, 0.3, 0.4], 1.0),
                ),
            ]))
            .with_probe_hierarchy_rt_lighting_rgb_and_weight(BTreeMap::from([
                (
                    resident_probe_id,
                    HybridGiResolveRuntime::pack_rgb_and_weight([0.3, 0.2, 0.1], 1.0),
                ),
                (
                    pending_probe_id,
                    HybridGiResolveRuntime::pack_rgb_and_weight([0.4, 0.3, 0.2], 1.0),
                ),
            ]))
            .with_probe_scene_driven_hierarchy_irradiance_ids(BTreeSet::from([
                resident_probe_id,
                pending_probe_id,
            ]))
            .with_probe_scene_driven_hierarchy_rt_lighting_ids(BTreeSet::from([
                resident_probe_id,
                pending_probe_id,
            ]))
            .build();
        let disabled_extract = RenderHybridGiExtract {
            enabled: false,
            trace_regions: vec![RenderHybridGiTraceRegion {
                region_id: stale_region_id,
                ..Default::default()
            }],
            ..Default::default()
        };

        let inputs = collect_inputs(
            &prepare,
            Some(&runtime),
            Some(&disabled_extract),
            None,
            &[],
            &[],
            &[],
            &[],
        );

        assert!(
            inputs.trace_region_inputs.is_empty(),
            "disabled old RenderHybridGiTraceRegion payloads must not reach GPU prepare"
        );
        assert_eq!(
            inputs.resident_probe_inputs[0].skip_scene_prepare_for_irradiance_q,
            1,
            "stale scheduled trace ids from a disabled legacy extract must not block runtime scene-truth irradiance reuse for resident probes"
        );
        assert_eq!(
            inputs.resident_probe_inputs[0].skip_scene_prepare_for_trace_q,
            1,
            "stale scheduled trace ids from a disabled legacy extract must not block runtime scene-truth trace reuse for resident probes"
        );
        assert_eq!(
            inputs.pending_probe_inputs[0].skip_scene_prepare_for_irradiance_q,
            1,
            "stale scheduled trace ids from a disabled legacy extract must not block runtime scene-truth irradiance reuse for pending probes"
        );
        assert_eq!(
            inputs.pending_probe_inputs[0].skip_scene_prepare_for_trace_q,
            1,
            "stale scheduled trace ids from a disabled legacy extract must not block runtime scene-truth trace reuse for pending probes"
        );
    }

    #[test]
    fn collect_inputs_treats_legacy_trace_schedule_as_empty_when_scene_prepare_owns_frame() {
        let resident_probe_id = 300;
        let stale_region_id = 900;
        let prepare = HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: resident_probe_id,
                slot: 0,
                ray_budget: 64,
                irradiance_rgb: [0, 0, 0],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![stale_region_id],
            evictable_probe_ids: Vec::new(),
        };
        let runtime = HybridGiResolveRuntime::fixture()
            .with_probe_hierarchy_irradiance_rgb_and_weight(BTreeMap::from([(
                resident_probe_id,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.1, 0.2, 0.3], 1.0),
            )]))
            .with_probe_scene_driven_hierarchy_irradiance_ids(BTreeSet::from([resident_probe_id]))
            .build();
        let legacy_extract = RenderHybridGiExtract {
            enabled: true,
            trace_budget: 1,
            probes: vec![RenderHybridGiProbe {
                probe_id: resident_probe_id,
                position: Vec3::ZERO,
                radius: 1.0,
                resident: true,
                ray_budget: 64,
                ..Default::default()
            }],
            trace_regions: vec![RenderHybridGiTraceRegion {
                region_id: stale_region_id,
                bounds_center: Vec3::ZERO,
                bounds_radius: 1.0,
                screen_coverage: 1.0,
                rt_lighting_rgb: [240, 96, 48],
                ..Default::default()
            }],
            ..Default::default()
        };
        let scene_prepare = HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        };

        let inputs = collect_inputs(
            &prepare,
            Some(&runtime),
            Some(&legacy_extract),
            Some(&scene_prepare),
            &[],
            &[],
            &[],
            &[],
        );

        assert!(
            inputs.trace_region_inputs.is_empty(),
            "scene prepare ownership must keep old RenderHybridGiTraceRegion payloads out of GPU prepare"
        );
        assert_eq!(
            inputs.resident_probe_inputs[0].lineage_trace_support_q, 0,
            "scene prepare ownership must not let stale legacy trace schedules feed probe lineage support"
        );
        assert_eq!(
            inputs.resident_probe_inputs[0].lineage_trace_lighting_rgb, 0,
            "scene prepare ownership must not let stale legacy trace schedules feed probe lineage lighting"
        );
        assert_eq!(
            inputs.resident_probe_inputs[0].skip_scene_prepare_for_irradiance_q,
            1,
            "stale legacy trace schedules must not block runtime scene-truth irradiance reuse once scene prepare owns the frame"
        );
    }

    #[test]
    fn collect_inputs_filters_scene_prepare_runtime_trace_data_backed_by_legacy_payload() {
        let resident_probe_id = 300;
        let stale_region_id = 900;
        let prepare = HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: resident_probe_id,
                slot: 0,
                ray_budget: 64,
                irradiance_rgb: [0, 0, 0],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![stale_region_id],
            evictable_probe_ids: Vec::new(),
        };
        let runtime = HybridGiResolveRuntime::fixture()
            .with_probe_scene_data(BTreeMap::from([(
                resident_probe_id,
                runtime_probe_scene_data(),
            )]))
            .with_trace_region_scene_data(BTreeMap::from([(
                stale_region_id,
                runtime_trace_region_scene_data([240, 96, 48]),
            )]))
            .build();
        let legacy_extract = RenderHybridGiExtract {
            enabled: true,
            trace_budget: 1,
            trace_regions: vec![legacy_trace_region(stale_region_id)],
            ..Default::default()
        };

        let inputs = collect_inputs(
            &prepare,
            Some(&runtime),
            Some(&legacy_extract),
            Some(&HybridGiScenePrepareFrame::default()),
            &[],
            &[],
            &[],
            &[],
        );

        assert!(
            inputs.trace_region_inputs.is_empty(),
            "scene prepare runtime ownership must not let runtime trace scene data backed by old RenderHybridGiTraceRegion payloads reach GPU prepare"
        );
        assert_eq!(
            inputs.resident_probe_inputs[0].lineage_trace_support_q, 0,
            "legacy-backed runtime trace scene data must not feed prepare-time probe lineage support during scene prepare"
        );
    }

    #[test]
    fn collect_inputs_keeps_scene_prepare_runtime_only_trace_data_when_legacy_payload_is_scheduled()
    {
        let resident_probe_id = 300;
        let legacy_region_id = 900;
        let runtime_only_region_id = 901;
        let prepare = HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: resident_probe_id,
                slot: 0,
                ray_budget: 64,
                irradiance_rgb: [0, 0, 0],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![
                legacy_region_id,
                runtime_only_region_id,
                runtime_only_region_id,
            ],
            evictable_probe_ids: Vec::new(),
        };
        let runtime = HybridGiResolveRuntime::fixture()
            .with_probe_scene_data(BTreeMap::from([(
                resident_probe_id,
                runtime_probe_scene_data(),
            )]))
            .with_trace_region_scene_data(BTreeMap::from([
                (
                    legacy_region_id,
                    runtime_trace_region_scene_data([240, 96, 48]),
                ),
                (
                    runtime_only_region_id,
                    runtime_trace_region_scene_data([32, 64, 240]),
                ),
            ]))
            .build();
        let legacy_extract = RenderHybridGiExtract {
            enabled: true,
            trace_budget: 1,
            trace_regions: vec![legacy_trace_region(legacy_region_id)],
            ..Default::default()
        };

        let inputs = collect_inputs(
            &prepare,
            Some(&runtime),
            Some(&legacy_extract),
            Some(&HybridGiScenePrepareFrame::default()),
            &[],
            &[],
            &[],
            &[],
        );

        assert_eq!(
            inputs.trace_region_inputs.len(),
            1,
            "scene prepare runtime ownership should filter only legacy-backed trace ids and keep runtime-only trace scene data"
        );
        assert_eq!(
            inputs.trace_region_inputs[0].region_id,
            runtime_only_region_id
        );
        assert!(
            inputs.resident_probe_inputs[0].lineage_trace_support_q > 0,
            "runtime-only trace scene data should still feed prepare-time probe lineage support"
        );
    }

    #[test]
    fn collect_inputs_filters_stripped_runtime_trace_data_backed_by_legacy_payload() {
        let resident_probe_id = 300;
        let stale_region_id = 900;
        let prepare = HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: resident_probe_id,
                slot: 0,
                ray_budget: 64,
                irradiance_rgb: [0, 0, 0],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![stale_region_id],
            evictable_probe_ids: Vec::new(),
        };
        let runtime = runtime_scene_truth_with_trace_region_scene_data(
            resident_probe_id,
            BTreeMap::from([(
                stale_region_id,
                runtime_trace_region_scene_data([240, 96, 48]),
            )]),
        );
        let legacy_extract = RenderHybridGiExtract {
            enabled: true,
            trace_regions: vec![legacy_trace_region(stale_region_id)],
            ..Default::default()
        };

        let inputs = collect_inputs(
            &prepare,
            Some(&runtime),
            Some(&legacy_extract),
            None,
            &[],
            &[],
            &[],
            &[],
        );

        assert!(
            inputs.trace_region_inputs.is_empty(),
            "stripped runtime scene truth must not let runtime trace scene data backed by old RenderHybridGiTraceRegion payloads reach GPU prepare"
        );
        assert_eq!(
            inputs.resident_probe_inputs[0].lineage_trace_support_q, 0,
            "legacy-backed runtime trace scene data must not feed prepare-time probe lineage support after scene-prepare has been stripped"
        );
    }

    #[test]
    fn collect_inputs_keeps_stripped_runtime_only_trace_data_when_legacy_payload_is_scheduled() {
        let resident_probe_id = 300;
        let legacy_region_id = 900;
        let runtime_only_region_id = 901;
        let prepare = HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: resident_probe_id,
                slot: 0,
                ray_budget: 64,
                irradiance_rgb: [0, 0, 0],
            }],
            pending_updates: Vec::new(),
            scheduled_trace_region_ids: vec![
                legacy_region_id,
                runtime_only_region_id,
                runtime_only_region_id,
            ],
            evictable_probe_ids: Vec::new(),
        };
        let runtime = runtime_scene_truth_with_trace_region_scene_data(
            resident_probe_id,
            BTreeMap::from([
                (
                    legacy_region_id,
                    runtime_trace_region_scene_data([240, 96, 48]),
                ),
                (
                    runtime_only_region_id,
                    runtime_trace_region_scene_data([32, 64, 240]),
                ),
            ]),
        );
        let legacy_extract = RenderHybridGiExtract {
            enabled: true,
            trace_regions: vec![legacy_trace_region(legacy_region_id)],
            ..Default::default()
        };

        let inputs = collect_inputs(
            &prepare,
            Some(&runtime),
            Some(&legacy_extract),
            None,
            &[],
            &[],
            &[],
            &[],
        );

        assert_eq!(
            inputs.trace_region_inputs.len(),
            1,
            "stripped runtime scene truth should filter only legacy-backed trace ids and keep runtime-only trace scene data"
        );
        assert_eq!(
            inputs.trace_region_inputs[0].region_id,
            runtime_only_region_id
        );
        assert!(
            inputs.resident_probe_inputs[0].lineage_trace_support_q > 0,
            "runtime-only trace scene data should still feed prepare-time probe lineage support after scene-prepare has been stripped"
        );
    }

    fn runtime_probe_scene_data() -> HybridGiResolveProbeSceneData {
        HybridGiResolveProbeSceneData::new(2048, 2048, 2048, 96)
    }

    fn runtime_scene_truth_with_trace_region_scene_data(
        probe_id: u32,
        trace_region_scene_data: BTreeMap<u32, HybridGiResolveTraceRegionSceneData>,
    ) -> HybridGiResolveRuntime {
        HybridGiResolveRuntime::fixture()
            .with_probe_scene_data(BTreeMap::from([(probe_id, runtime_probe_scene_data())]))
            .with_trace_region_scene_data(trace_region_scene_data)
            .with_probe_hierarchy_irradiance_rgb_and_weight(BTreeMap::from([(
                probe_id,
                HybridGiResolveRuntime::pack_rgb_and_weight([0.25, 0.45, 0.75], 0.5),
            )]))
            .with_probe_scene_driven_hierarchy_irradiance_ids(BTreeSet::from([probe_id]))
            .build()
    }

    fn runtime_trace_region_scene_data(
        rt_lighting_rgb: [u8; 3],
    ) -> HybridGiResolveTraceRegionSceneData {
        HybridGiResolveTraceRegionSceneData::new(2048, 2048, 2048, 96, 128, rt_lighting_rgb)
    }

    fn legacy_trace_region(region_id: u32) -> RenderHybridGiTraceRegion {
        RenderHybridGiTraceRegion {
            region_id,
            bounds_center: Vec3::ZERO,
            bounds_radius: 1.0,
            screen_coverage: 1.0,
            ..Default::default()
        }
    }
}
