use zircon_runtime::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderPointLightSnapshot, RenderSpotLightSnapshot,
};

use crate::hybrid_gi::types::HybridGiPrepareFrame;

use super::super::super::hybrid_gi_completion_params::HybridGiCompletionParams;
use super::super::super::hybrid_gi_gpu_resources::HybridGiGpuResources;
use super::super::scene_light_seed::scene_light_seed;
use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;

pub(super) fn queue_params(
    resources: &HybridGiGpuResources,
    queue: &wgpu::Queue,
    prepare: &HybridGiPrepareFrame,
    inputs: &HybridGiPrepareExecutionInputs,
    directional_lights: &[RenderDirectionalLightSnapshot],
    point_lights: &[RenderPointLightSnapshot],
    spot_lights: &[RenderSpotLightSnapshot],
    probe_budget: Option<u32>,
    tracing_budget: Option<u32>,
) {
    let scene_light_seed = scene_light_seed(directional_lights, point_lights, spot_lights);
    let params = HybridGiCompletionParams {
        resident_probe_count: prepare.resident_probes.len() as u32,
        pending_probe_count: inputs.pending_probe_inputs.len() as u32,
        probe_budget: probe_budget.unwrap_or_default(),
        trace_region_count: inputs.trace_region_inputs.len() as u32,
        scene_card_capture_request_count: inputs.scene_card_capture_descriptor_count as u32,
        scene_voxel_clipmap_count: inputs.scene_voxel_clipmaps.len() as u32,
        scene_voxel_cell_count: inputs.scene_voxel_cells.len() as u32,
        tracing_budget: tracing_budget.unwrap_or_default(),
        evictable_probe_count: prepare.evictable_probe_ids.len() as u32,
        scene_light_seed_rgb: scene_light_seed.packed_rgb,
        scene_light_strength_q: scene_light_seed.strength_q,
        _padding1: 0,
    };
    queue.write_buffer(&resources.params_buffer, 0, bytemuck::bytes_of(&params));
}
