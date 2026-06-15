pub(crate) struct ParticleRenderer {
    pub(in crate::graphics::scene::scene_renderer::particle) pipeline: wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::particle) velocity_pipeline:
        wgpu::RenderPipeline,
}
