use crate::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderHybridGiExtract, RenderMeshSnapshot,
    RenderPointLightSnapshot, RenderSpotLightSnapshot,
};

use crate::graphics::types::{
    HybridGiPrepareFrame, HybridGiResolveRuntime, HybridGiScenePrepareFrame,
};

use super::super::pending_probe_inputs::pending_probe_inputs;
use super::super::resident_probe_inputs::resident_probe_inputs;
use super::super::trace_region_inputs::trace_region_inputs;
use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;

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
    let cache_entries = prepare
        .resident_probes
        .iter()
        .map(|probe| [probe.probe_id, probe.slot])
        .collect::<Vec<_>>();
    let resident_probe_inputs = resident_probe_inputs(prepare, resolve_runtime, extract);
    let pending_probe_inputs = pending_probe_inputs(prepare, resolve_runtime, extract);
    let trace_region_inputs = trace_region_inputs(prepare, extract);
    let scene_card_capture_requests = scene_prepare
        .map(|prepare| prepare.card_capture_requests.clone())
        .unwrap_or_default();
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
        scene_voxel_clipmaps,
        scene_voxel_cells,
        scene_meshes: scene_meshes.to_vec(),
        directional_lights: directional_lights.to_vec(),
        point_lights: point_lights.to_vec(),
        spot_lights: spot_lights.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use crate::core::math::Vec3;
    use crate::graphics::types::{
        HybridGiPrepareCardCaptureRequest, HybridGiPrepareFrame, HybridGiPrepareVoxelCell,
        HybridGiPrepareVoxelClipmap, HybridGiScenePrepareFrame,
    };

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
        assert_eq!(inputs.scene_voxel_clipmaps, scene_prepare.voxel_clipmaps);
        assert_eq!(inputs.scene_voxel_cells, scene_prepare.voxel_cells);
        assert!(inputs.scene_meshes.is_empty());
        assert!(inputs.directional_lights.is_empty());
        assert!(inputs.point_lights.is_empty());
        assert!(inputs.spot_lights.is_empty());
    }
}
