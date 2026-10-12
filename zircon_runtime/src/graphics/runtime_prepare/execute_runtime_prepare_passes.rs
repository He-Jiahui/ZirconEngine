use crate::graphics::backend::GpuPassTimer;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};
use crate::rhi::RenderDeviceProfile;

use super::super::super::scene_renderer_core::{
    SceneRendererAdvancedPluginReadbacks, SceneRendererCore,
};

impl SceneRendererCore {
    /// 在场景图物化前运行已登记插件准备回调；是否允许 GPU 工作由调用方本帧 admission 决定。
    pub(in crate::graphics::scene::scene_renderer::core::scene_renderer_core_render_compiled_scene) fn execute_runtime_prepare_passes(
        &mut self,
        device_profile: &RenderDeviceProfile,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        gpu_work_admitted: bool,
        gpu_pass_timer: Option<&mut GpuPassTimer>,
    ) -> Result<SceneRendererAdvancedPluginReadbacks, GraphicsError> {
        self.advanced_plugin_resources
            .execute_runtime_prepare_passes_with_gpu_work_admission(
                device_profile,
                device,
                encoder,
                streamer,
                frame,
                gpu_work_admitted,
                gpu_pass_timer,
            )
    }
}
