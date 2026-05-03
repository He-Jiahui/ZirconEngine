use super::scene_renderer_advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;
use crate::core::framework::render::{
    RenderHybridGiReadbackOutputs, RenderParticleGpuReadbackOutputs, RenderPluginRendererOutputs,
    RenderVirtualGeometryReadbackOutputs,
};

impl SceneRendererAdvancedPluginOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn has_virtual_geometry_gpu_readback(
        &self,
    ) -> bool {
        !self
            .plugin_renderer_outputs_ref()
            .virtual_geometry
            .is_empty()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn plugin_renderer_outputs(
        &self,
    ) -> &RenderPluginRendererOutputs {
        self.plugin_renderer_outputs_ref()
    }

    // Particle GPU readback detection is test-covered before a feedback consumer lands.
    #[allow(dead_code)]
    pub(in crate::graphics::scene::scene_renderer::core) fn has_particle_gpu_readback(
        &self,
    ) -> bool {
        !self.plugin_renderer_outputs_ref().particles.is_empty()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn take_hybrid_gi_readback_outputs(
        &mut self,
    ) -> RenderHybridGiReadbackOutputs {
        std::mem::take(&mut self.plugin_renderer_outputs_mut().hybrid_gi)
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn take_particle_gpu_readback_outputs(
        &mut self,
    ) -> RenderParticleGpuReadbackOutputs {
        std::mem::take(&mut self.plugin_renderer_outputs_mut().particles)
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn take_virtual_geometry_readback_outputs(
        &mut self,
    ) -> RenderVirtualGeometryReadbackOutputs {
        std::mem::take(&mut self.plugin_renderer_outputs_mut().virtual_geometry)
    }
}
