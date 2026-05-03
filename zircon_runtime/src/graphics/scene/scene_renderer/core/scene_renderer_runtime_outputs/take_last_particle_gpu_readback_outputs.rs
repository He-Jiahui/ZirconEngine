use crate::core::framework::render::RenderParticleGpuReadbackOutputs;

use super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    // Particle GPU readbacks are host-visible before a runtime feedback consumer exists.
    #[allow(dead_code)]
    pub(in crate::graphics) fn take_last_particle_gpu_readback_outputs(
        &mut self,
    ) -> RenderParticleGpuReadbackOutputs {
        self.advanced_plugin_outputs
            .take_particle_gpu_readback_outputs()
    }
}
