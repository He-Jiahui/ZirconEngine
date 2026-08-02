use crate::core::framework::render::ShaderVariantPrewarmManifest;
use crate::graphics::scene::scene_renderer::mesh::RuntimeShaderPipelinePrewarmReport;

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn prewarm_shader_pipelines(
        &mut self,
        manifest: &ShaderVariantPrewarmManifest,
    ) -> RuntimeShaderPipelinePrewarmReport {
        self.core
            .mesh_pipelines
            .prewarm_manifest(&self.backend.device, manifest)
    }
}
