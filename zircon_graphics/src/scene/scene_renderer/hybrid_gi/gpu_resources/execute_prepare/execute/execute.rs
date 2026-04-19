use zircon_framework::render::{RenderDirectionalLightSnapshot, RenderHybridGiExtract};

use crate::types::{GraphicsError, HybridGiPrepareFrame, HybridGiResolveRuntime};

use super::super::super::super::HybridGiGpuPendingReadback;
use super::super::super::HybridGiGpuResources;
use super::collect_inputs::collect_inputs;
use super::copy_readbacks::copy_readbacks;
use super::create_bind_group::create_bind_group;
use super::create_buffers::create_buffers;
use super::dispatch::dispatch;
use super::queue_params::queue_params;

impl HybridGiGpuResources {
    pub(crate) fn execute_prepare(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        prepare: Option<&HybridGiPrepareFrame>,
        resolve_runtime: Option<&HybridGiResolveRuntime>,
        extract: Option<&RenderHybridGiExtract>,
        directional_lights: &[RenderDirectionalLightSnapshot],
        probe_budget: Option<u32>,
        tracing_budget: Option<u32>,
    ) -> Result<Option<HybridGiGpuPendingReadback>, GraphicsError> {
        let Some(prepare) = prepare else {
            return Ok(None);
        };

        let inputs = collect_inputs(prepare, resolve_runtime, extract);
        let buffers = create_buffers(device, encoder, &inputs);
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

        Ok(Some(HybridGiGpuPendingReadback::new(
            inputs.cache_word_count,
            buffers.cache_readback,
            inputs.completed_probe_word_count.max(1),
            buffers.completed_probe_readback,
            inputs.completed_trace_word_count.max(1),
            buffers.completed_trace_readback,
            inputs.irradiance_word_count.max(1),
            buffers.irradiance_readback,
            inputs.trace_lighting_word_count.max(1),
            buffers.trace_lighting_readback,
        )))
    }
}
