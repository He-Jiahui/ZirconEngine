use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use zircon_runtime::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderMeshBounds, RenderMeshSnapshot, RenderPointLightSnapshot,
    RenderSpotLightSnapshot, RENDER_HYBRID_GI_PROBE_TRACE_DIAGNOSTIC_WORD_COUNT,
};
use zircon_runtime::core::math::Vec3;

use crate::hybrid_gi::types::{
    HybridGiPrepareFrame, HybridGiPrepareRadianceCacheConsume, HybridGiPrepareRadianceCacheUpdate,
    HybridGiPrepareSurfaceCacheDepthSourceSample, HybridGiResolveRuntime,
    HybridGiScenePrepareFrame,
};

use super::super::super::gpu_radiance_cache_consume_input::GpuRadianceCacheConsumeInput;
use super::super::pending_probe_inputs::pending_probe_inputs;
use super::super::resident_probe_inputs::resident_probe_inputs;
use super::super::trace_region_inputs::trace_region_inputs;
use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;

fn persisted_surface_cache_page_has_present_sample(
    page_content: &crate::hybrid_gi::types::HybridGiPrepareSurfaceCachePageContent,
) -> bool {
    page_content.capture_sample_rgba[3] > 0 || page_content.atlas_sample_rgba[3] > 0
}

const SCENE_DEPTH_SOURCE_WORLD_Z_HALF_RANGE: f32 = 64.0;

pub(super) fn collect_inputs(
    prepare: &HybridGiPrepareFrame,
    resolve_runtime: Option<&HybridGiResolveRuntime>,
    scene_prepare: Option<&HybridGiScenePrepareFrame>,
    radiance_cache_updates: &[HybridGiPrepareRadianceCacheUpdate],
    radiance_cache_consumes: &[HybridGiPrepareRadianceCacheConsume],
    scene_mesh_world_bounds: Arc<[(u64, RenderMeshBounds)]>,
    scene_meshes: Arc<[RenderMeshSnapshot]>,
    directional_lights: &[RenderDirectionalLightSnapshot],
    point_lights: &[RenderPointLightSnapshot],
    spot_lights: &[RenderSpotLightSnapshot],
) -> HybridGiPrepareExecutionInputs {
    let cache_entries = prepare
        .resident_probes
        .iter()
        .map(|probe| [probe.probe_id, probe.slot])
        .collect::<Vec<_>>();
    let resident_probe_inputs = resident_probe_inputs(prepare, resolve_runtime);
    let resident_probe_index_by_id = resident_probe_inputs
        .iter()
        .enumerate()
        .map(|(index, probe)| (probe.probe_id, index.min(u32::MAX as usize) as u32))
        .collect::<BTreeMap<_, _>>();
    let pending_probe_inputs = pending_probe_inputs(prepare, resolve_runtime);
    let trace_region_inputs = trace_region_inputs(prepare, resolve_runtime);
    let radiance_cache_update_inputs = radiance_cache_updates.iter().map(Into::into).collect();
    let radiance_cache_consume_inputs = radiance_cache_consumes
        .iter()
        .filter_map(|consume| {
            resident_probe_index_by_id
                .get(&consume.probe_id)
                .copied()
                .map(|resident_probe_index| {
                    GpuRadianceCacheConsumeInput::new(consume, resident_probe_index)
                })
        })
        .collect();
    let scene_card_capture_requests = scene_prepare
        .map(|prepare| prepare.card_capture_requests.clone())
        .unwrap_or_default();
    let scene_surface_cache_page_contents = scene_prepare
        .map(|prepare| prepare.surface_cache_page_contents.clone())
        .unwrap_or_default();
    let scene_surface_cache_depth_source_samples = scene_surface_cache_depth_source_samples(
        &scene_card_capture_requests,
        &scene_surface_cache_page_contents,
    );
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
        trace_diagnostic_word_count: 1
            + (resident_probe_inputs.len() + pending_probe_inputs.len()).max(1)
                * RENDER_HYBRID_GI_PROBE_TRACE_DIAGNOSTIC_WORD_COUNT,
        cache_entries,
        resident_probe_inputs,
        pending_probe_inputs,
        radiance_cache_update_inputs,
        radiance_cache_consume_inputs,
        trace_region_inputs,
        scene_card_capture_requests,
        scene_surface_cache_depth_source_samples,
        scene_surface_cache_page_contents,
        scene_card_capture_descriptor_count,
        scene_voxel_clipmaps,
        scene_voxel_cells,
        scene_mesh_world_bounds,
        scene_meshes,
        directional_lights: directional_lights.to_vec(),
        point_lights: point_lights.to_vec(),
        spot_lights: spot_lights.to_vec(),
    }
}

fn scene_surface_cache_depth_source_samples(
    card_capture_requests: &[crate::hybrid_gi::types::HybridGiPrepareCardCaptureRequest],
    surface_cache_page_contents: &[crate::hybrid_gi::types::HybridGiPrepareSurfaceCachePageContent],
) -> Vec<HybridGiPrepareSurfaceCacheDepthSourceSample> {
    let mut depth_source_by_slot = surface_cache_page_contents
        .iter()
        .filter(|page_content| {
            page_content.atlas_slot_id != u32::MAX
                && persisted_surface_cache_page_has_present_sample(page_content)
        })
        .map(|page_content| {
            (
                page_content.atlas_slot_id,
                HybridGiPrepareSurfaceCacheDepthSourceSample {
                    page_id: page_content.page_id,
                    atlas_slot_id: page_content.atlas_slot_id,
                    depth_rgba: depth_source_rgba_from_scene_bounds(
                        page_content.bounds_center,
                        page_content.bounds_radius,
                    ),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    depth_source_by_slot.extend(
        card_capture_requests
            .iter()
            .filter(|request| request.atlas_slot_id != u32::MAX)
            .map(|request| {
                (
                    request.atlas_slot_id,
                    HybridGiPrepareSurfaceCacheDepthSourceSample {
                        page_id: request.page_id,
                        atlas_slot_id: request.atlas_slot_id,
                        depth_rgba: depth_source_rgba_from_scene_bounds(
                            request.bounds_center,
                            request.bounds_radius,
                        ),
                    },
                )
            }),
    );

    depth_source_by_slot.into_values().collect()
}

fn depth_source_rgba_from_scene_bounds(bounds_center: Vec3, bounds_radius: f32) -> [u8; 4] {
    let near_z = bounds_center.z - bounds_radius.max(0.0);
    let normalized = ((near_z + SCENE_DEPTH_SOURCE_WORLD_Z_HALF_RANGE)
        / (SCENE_DEPTH_SOURCE_WORLD_Z_HALF_RANGE * 2.0))
        .clamp(0.0, 1.0);
    let encoded = (normalized * 254.0).round() as u8;
    [encoded, encoded, encoded, u8::MAX]
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::hybrid_gi::types::{
        HybridGiPrepareCardCaptureRequest, HybridGiPrepareProbe,
        HybridGiPrepareRadianceCacheConsume, HybridGiPrepareRadianceCacheUpdate,
        HybridGiPrepareSurfaceCachePageContent, HybridGiResolveProbeSceneData,
        HybridGiResolveTraceRegionSceneData,
    };

    use super::*;

    #[test]
    fn collect_inputs_preserves_scene_prepare_and_runtime_sideband_contracts() {
        let prepare = HybridGiPrepareFrame {
            resident_probes: vec![HybridGiPrepareProbe {
                probe_id: 7,
                slot: 0,
                stable_instance_key: 0,
                source_mask: zircon_runtime::core::framework::render::HYBRID_GI_SOURCE_FULL_DYNAMIC,
                dynamic_weight_q8: u8::MAX,
                ray_budget: 32,
                irradiance_rgb: [8, 16, 24],
            }],
            scheduled_trace_region_ids: vec![9],
            ..HybridGiPrepareFrame::default()
        };
        let runtime = HybridGiResolveRuntime::fixture()
            .with_probe_scene_data(BTreeMap::from([(
                7,
                HybridGiResolveProbeSceneData::new(2112, 2048, 2048, 96),
            )]))
            .with_trace_region_scene_data(BTreeMap::from([(
                9,
                HybridGiResolveTraceRegionSceneData::new(2112, 2048, 2048, 192, 128, [64, 96, 128]),
            )]))
            .build();
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
                owner_card_id: 11,
                atlas_slot_id: 3,
                capture_slot_id: 4,
                bounds_center: Vec3::new(1.0, 2.0, 3.0),
                bounds_radius: 0.5,
                atlas_sample_rgba: [10, 20, 30, 255],
                capture_sample_rgba: [40, 50, 60, 255],
            }],
            radiance_cache_updates: vec![HybridGiPrepareRadianceCacheUpdate {
                slot: 3,
                generation: 9,
                radiance_rgb: [40, 50, 60],
                confidence_q8: 200,
                reuse_committed_radiance: false,
            }],
            radiance_cache_consumes: vec![HybridGiPrepareRadianceCacheConsume {
                probe_id: 7,
                generation: 9,
                slots: [3; 8],
                weights_q16: [u16::MAX, 0, 0, 0, 0, 0, 0, 0],
            }],
            ..HybridGiScenePrepareFrame::default()
        };

        let inputs = collect_inputs(
            &prepare,
            Some(&runtime),
            Some(&scene_prepare),
            &scene_prepare.radiance_cache_updates,
            &scene_prepare.radiance_cache_consumes,
            &[],
            &[],
            &[],
            &[],
            &[],
        );

        assert_eq!(inputs.resident_probe_inputs.len(), 1);
        assert_eq!(inputs.resident_probe_inputs[0].position_x_q, 2112);
        assert_eq!(inputs.trace_region_inputs.len(), 1);
        assert_eq!(inputs.trace_region_inputs[0].region_id, 9);
        assert_eq!(inputs.scene_card_capture_descriptor_count, 1);
        assert_eq!(inputs.scene_surface_cache_page_contents.len(), 1);
        assert_eq!(inputs.scene_surface_cache_depth_source_samples.len(), 1);
        assert_eq!(inputs.radiance_cache_update_inputs.len(), 1);
        assert_eq!(inputs.radiance_cache_update_inputs[0].slot, 3);
        assert_eq!(inputs.radiance_cache_update_inputs[0].generation_low, 9);
        assert_eq!(
            inputs.radiance_cache_update_inputs[0]
                .radiance_confidence
                .to_le_bytes(),
            [40, 50, 60, 200]
        );
        assert_eq!(inputs.radiance_cache_consume_inputs.len(), 1);
        assert_eq!(inputs.radiance_cache_consume_inputs[0].probe_id, 7);
        assert_eq!(
            inputs.radiance_cache_consume_inputs[0].resident_probe_index,
            0
        );
        assert_eq!(inputs.radiance_cache_consume_inputs[0].slots, [3; 8]);
        assert_eq!(
            inputs.radiance_cache_consume_inputs[0].weights_q16[0],
            u32::from(u16::MAX)
        );
    }
}
