use super::scene_renderer_advanced_plugin_resources::SceneRendererAdvancedPluginResources;
use crate::core::framework::render::RenderPluginRendererOutputs;
use crate::graphics::backend::GpuPassTimer;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::core::scene_renderer_core::{
    merge_plugin_renderer_outputs, SceneRendererAdvancedPluginReadbacks,
};
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::graphics::{
    RuntimePrepareDeviceEpoch, RuntimePrepareExternalBufferBinding, RuntimePrepareFramePacket,
    RuntimePrepareGpuPassProfile, RuntimePrepareGpuReadbackRequest,
};
use crate::rhi::RenderDeviceProfile;

impl SceneRendererAdvancedPluginResources {
    pub(in crate::graphics::scene::scene_renderer::core) fn execute_runtime_prepare_passes(
        &mut self,
        device_profile: &RenderDeviceProfile,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
    ) -> Result<SceneRendererAdvancedPluginReadbacks, GraphicsError> {
        self.execute_runtime_prepare_passes_with_gpu_work_admission(
            device_profile,
            device,
            encoder,
            streamer,
            frame,
            true,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::graphics::scene::scene_renderer::core) fn execute_runtime_prepare_passes_with_gpu_work_admission(
        &mut self,
        device_profile: &RenderDeviceProfile,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        gpu_work_admitted: bool,
        mut gpu_pass_timer: Option<&mut GpuPassTimer>,
    ) -> Result<SceneRendererAdvancedPluginReadbacks, GraphicsError> {
        let collectors = self.runtime_prepare_collectors_mut();
        if collectors.is_empty() {
            return Ok(SceneRendererAdvancedPluginReadbacks::new());
        }

        let mut outputs = RenderPluginRendererOutputs::default();
        let mut external_buffer_bindings = Vec::<RuntimePrepareExternalBufferBinding>::new();
        let mut gpu_readbacks = Vec::<RuntimePrepareGpuReadbackRequest>::new();
        let mut gpu_pass_profiles = Vec::<RuntimePrepareGpuPassProfile>::new();
        let mut runtime_prepare_frame_packet = RuntimePrepareFramePacket::default();
        let device_epoch = RuntimePrepareDeviceEpoch::from_device_profile(device_profile);
        for collector in collectors {
            merge_plugin_renderer_outputs(
                &mut outputs,
                collector(
                    device,
                    device_epoch,
                    encoder,
                    streamer,
                    frame,
                    &mut external_buffer_bindings,
                    &mut gpu_readbacks,
                    gpu_work_admitted,
                    gpu_pass_timer.as_deref_mut(),
                    &mut gpu_pass_profiles,
                    &mut runtime_prepare_frame_packet,
                )?,
            );
        }

        Ok(
            SceneRendererAdvancedPluginReadbacks::from_outputs_external_and_gpu_readbacks(
                device_profile,
                outputs,
                external_buffer_bindings,
                gpu_readbacks,
            )
            .with_gpu_pass_profiles(gpu_pass_profiles)
            .with_runtime_prepare_frame_packet(runtime_prepare_frame_packet),
        )
    }
}

#[cfg(test)]
#[path = "tests/runtime_prepare.rs"]
mod tests;
