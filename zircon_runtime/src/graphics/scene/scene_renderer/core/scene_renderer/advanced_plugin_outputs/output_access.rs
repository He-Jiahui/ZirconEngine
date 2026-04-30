#[cfg(test)]
use crate::graphics::scene::scene_renderer::HybridGiGpuReadback;
use crate::graphics::scene::scene_renderer::{
    HybridGiGpuReadbackCompletionParts, VirtualGeometryGpuReadback,
    VirtualGeometryGpuReadbackCompletionParts,
};

use super::scene_renderer_advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;

impl SceneRendererAdvancedPluginOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn take_hybrid_gi_gpu_completion_parts(
        &mut self,
    ) -> Option<HybridGiGpuReadbackCompletionParts> {
        self.hybrid_gi_readback_mut().take_gpu_completion_parts()
    }

    #[cfg(test)]
    pub(crate) fn take_hybrid_gi_gpu_readback(&mut self) -> Option<HybridGiGpuReadback> {
        self.hybrid_gi_readback_mut().take_gpu_readback()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn has_virtual_geometry_gpu_readback(
        &self,
    ) -> bool {
        self.virtual_geometry_readback().has_gpu_readback()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_gpu_readback_mut(
        &mut self,
    ) -> Option<&mut VirtualGeometryGpuReadback> {
        self.virtual_geometry_readback_mut().gpu_readback_mut()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_gpu_readback(
        &self,
    ) -> Option<&VirtualGeometryGpuReadback> {
        self.virtual_geometry_readback().gpu_readback()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn take_virtual_geometry_gpu_completion_parts(
        &mut self,
    ) -> Option<VirtualGeometryGpuReadbackCompletionParts> {
        self.virtual_geometry_readback_mut()
            .take_gpu_completion_parts()
    }

    #[cfg(test)]
    pub(crate) fn take_virtual_geometry_gpu_readback(
        &mut self,
    ) -> Option<VirtualGeometryGpuReadback> {
        self.virtual_geometry_readback_mut().take_gpu_readback()
    }
}
