use crate::core::framework::render::RenderParticleGpuReadbackOutputs;

use super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(in crate::graphics) fn take_last_particle_gpu_readback_outputs(
        &mut self,
    ) -> RenderParticleGpuReadbackOutputs {
        self.advanced_plugin_outputs
            .take_particle_gpu_readback_outputs()
    }
}
