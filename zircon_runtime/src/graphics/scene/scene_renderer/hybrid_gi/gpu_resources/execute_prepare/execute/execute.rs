use crate::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderHybridGiExtract, RenderMeshSnapshot,
    RenderPointLightSnapshot, RenderSpotLightSnapshot,
};
use crate::graphics::scene::resources::ResourceStreamer;

use crate::graphics::types::{
    GraphicsError, HybridGiPrepareFrame, HybridGiResolveRuntime, HybridGiScenePrepareFrame,
};

use super::super::super::super::HybridGiGpuPendingReadback;
use super::super::super::HybridGiGpuResources;
use super::collect_inputs::collect_inputs;
use super::copy_readbacks::copy_readbacks;
use super::create_bind_group::create_bind_group;
use super::create_buffers::create_buffers;
use super::dispatch::dispatch;
use super::hybrid_gi_prepare_execution_buffers::HybridGiPrepareExecutionBuffers;
use super::queue_params::queue_params;

impl HybridGiGpuResources {
    pub(crate) fn execute_prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &ResourceStreamer,
        prepare: Option<&HybridGiPrepareFrame>,
        scene_prepare: Option<&HybridGiScenePrepareFrame>,
        resolve_runtime: Option<&HybridGiResolveRuntime>,
        extract: Option<&RenderHybridGiExtract>,
        scene_meshes: &[RenderMeshSnapshot],
        directional_lights: &[RenderDirectionalLightSnapshot],
        point_lights: &[RenderPointLightSnapshot],
        spot_lights: &[RenderSpotLightSnapshot],
        probe_budget: Option<u32>,
        tracing_budget: Option<u32>,
    ) -> Result<Option<HybridGiGpuPendingReadback>, GraphicsError> {
        let Some(prepare) = prepare else {
            return Ok(None);
        };

        let inputs = collect_inputs(
            prepare,
            resolve_runtime,
            extract,
            scene_prepare,
            scene_meshes,
            directional_lights,
            point_lights,
            spot_lights,
        );
        let buffers = create_buffers(device, encoder, streamer, &inputs);
        queue_params(
            self,
            queue,
            prepare,
            &inputs,
            directional_lights,
            probe_budget,
            tracing_budget,
        );
        let bind_group = create_bind_group(self, device, &buffers);
        dispatch(self, encoder, &bind_group, &inputs);
        copy_readbacks(encoder, &buffers, &inputs);
        let HybridGiPrepareExecutionBuffers {
            cache_readback,
            resident_probe_buffer: _resident_probe_buffer,
            pending_probe_buffer: _pending_probe_buffer,
            trace_region_buffer: _trace_region_buffer,
            scene_prepare_descriptor_buffer: _scene_prepare_descriptor_buffer,
            completed_probe_buffer: _completed_probe_buffer,
            completed_trace_buffer: _completed_trace_buffer,
            completed_probe_readback,
            completed_trace_readback,
            irradiance_buffer: _irradiance_buffer,
            irradiance_readback,
            trace_lighting_buffer: _trace_lighting_buffer,
            trace_lighting_readback,
            scene_prepare_resources,
        } = buffers;
        let (
            scene_prepare_resources,
            scene_prepare_atlas_texture,
            scene_prepare_atlas_view,
            scene_prepare_atlas_upload_buffer,
            scene_prepare_atlas_slot_sample_buffers,
            scene_prepare_capture_texture,
            scene_prepare_capture_views,
            scene_prepare_capture_upload_buffer,
            scene_prepare_capture_slot_sample_buffers,
        ) = match scene_prepare_resources {
            Some(resources) => (
                Some(resources.snapshot),
                resources.atlas_texture,
                resources.atlas_view,
                resources.atlas_upload_buffer,
                resources.atlas_slot_sample_buffers,
                resources.capture_texture,
                resources.capture_views,
                resources.capture_upload_buffer,
                resources.capture_slot_sample_buffers,
            ),
            None => (
                None,
                None,
                None,
                None,
                Vec::new(),
                None,
                Vec::new(),
                None,
                Vec::new(),
            ),
        };

        Ok(Some(HybridGiGpuPendingReadback::new(
            inputs.cache_word_count,
            cache_readback,
            inputs.completed_probe_word_count.max(1),
            completed_probe_readback,
            inputs.completed_trace_word_count.max(1),
            completed_trace_readback,
            inputs.irradiance_word_count.max(1),
            irradiance_readback,
            inputs.trace_lighting_word_count.max(1),
            trace_lighting_readback,
            scene_prepare_resources,
            scene_prepare_atlas_texture,
            scene_prepare_atlas_view,
            scene_prepare_atlas_upload_buffer,
            scene_prepare_atlas_slot_sample_buffers,
            scene_prepare_capture_texture,
            scene_prepare_capture_views,
            scene_prepare_capture_upload_buffer,
            scene_prepare_capture_slot_sample_buffers,
        )))
    }
}
