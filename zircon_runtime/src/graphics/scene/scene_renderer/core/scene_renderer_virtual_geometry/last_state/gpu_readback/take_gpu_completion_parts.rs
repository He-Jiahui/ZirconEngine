use crate::graphics::scene::scene_renderer::core::SceneRenderer;
use crate::graphics::scene::scene_renderer::VirtualGeometryGpuReadbackCompletionParts;

impl SceneRenderer {
    pub(in crate::graphics) fn take_last_virtual_geometry_gpu_completion_parts(
        &mut self,
    ) -> Option<VirtualGeometryGpuReadbackCompletionParts> {
        self.advanced_plugin_outputs
            .take_virtual_geometry_gpu_completion_parts()
    }
}
