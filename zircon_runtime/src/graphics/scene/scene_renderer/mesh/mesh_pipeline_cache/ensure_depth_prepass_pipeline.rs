use crate::core::framework::render::ShaderVariantKey;
use crate::graphics::scene::resources::{PipelineKey, ResourceStreamer};

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::create_depth_prepass_mesh_pipeline;
use super::MeshPipelineCache;
use super::shader_source::mesh_pipeline_depth_prepass_template_source_for_geometry_descriptor_with_streamer;

const DEPTH_PREPASS_MESH_SHADER_KEY_PREFIX: &str = "zircon.builtin.depth-prepass-mesh@1";

impl MeshPipelineCache {
    fn ensure_depth_prepass_pipeline<'a>(
        &'a mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
        key: &PipelineKey,
        shader_variant_key: &ShaderVariantKey,
    ) -> Option<&'a wgpu::RenderPipeline> {
        let geometry_source = self.geometry_source_descriptor_for_variant(shader_variant_key)?;
        let shader_source =
            match mesh_pipeline_depth_prepass_template_source_for_geometry_descriptor_with_streamer(
                streamer,
                key,
                &geometry_source,
            ) {
                Ok(source) => source,
                Err(_) => {
                    self.record_shader_variant_disk_error(shader_variant_key);
                    return None;
                }
            };
        let shader_key =
            depth_prepass_mesh_shader_key(shader_variant_key, &shader_source.source_hash);
        if !self.shader_modules.contains_key(&shader_key) {
            let source =
                self.mesh_pipeline_shader_source_with_cache(shader_source, shader_variant_key);
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-depth-prepass-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            self.shader_modules.insert(shader_key.clone(), module);
        }
        if !self.depth_prepass_mesh_pipelines.contains_key(&variant_id) {
            let shader = self
                .shader_modules
                .get(&shader_key)
                .expect("depth prepass mesh shader module cached");
            let pipeline =
                create_depth_prepass_mesh_pipeline(device, &self.mesh_pipeline_layout, shader, key);
            self.depth_prepass_mesh_pipelines
                .insert(variant_id, pipeline);
        }
        self.depth_prepass_mesh_pipelines.get(&variant_id)
    }

    pub(crate) fn ensure_depth_prepass_pipeline_for_variant<'a>(
        &'a mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
    ) -> Option<&'a wgpu::RenderPipeline> {
        if self.depth_prepass_mesh_pipelines.contains_key(&variant_id) {
            return self.depth_prepass_mesh_pipelines.get(&variant_id);
        }
        let (kind, pipeline_key, shader_variant_key) =
            self.pipeline_and_shader_key_for_variant(variant_id)?;
        if kind != MeshPassPipelineKind::DepthPrepass {
            return None;
        }
        self.ensure_depth_prepass_pipeline(
            device,
            streamer,
            variant_id,
            &pipeline_key,
            &shader_variant_key,
        )
    }
}

fn depth_prepass_mesh_shader_key(variant_key: &ShaderVariantKey, source_hash: &str) -> String {
    format!(
        "{}#{}#{}",
        DEPTH_PREPASS_MESH_SHADER_KEY_PREFIX,
        variant_key.canonical_string(),
        source_hash
    )
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::ShaderPassType;
    use crate::graphics::scene::resources::default_pipeline_key;

    use super::super::mesh_pipeline_depth_prepass_template_source_for_geometry;
    use super::{DEPTH_PREPASS_MESH_SHADER_KEY_PREFIX, depth_prepass_mesh_shader_key};

    #[test]
    fn depth_prepass_mesh_shader_key_includes_shader_variant_identity_and_source_hash() {
        let variant_key =
            default_pipeline_key().shader_variant_key(ShaderPassType::DepthPrepass, "wgpu-runtime");
        let source = match mesh_pipeline_depth_prepass_template_source_for_geometry(
            &default_pipeline_key(),
            variant_key.geometry_source,
        ) {
            Ok(source) => source,
            Err(error) => panic!("depth prepass template source assembly failed: {error:?}"),
        };
        let key = depth_prepass_mesh_shader_key(&variant_key, &source.source_hash);

        assert!(key.starts_with(DEPTH_PREPASS_MESH_SHADER_KEY_PREFIX));
        assert!(key.contains(&variant_key.canonical_string()));
        assert!(key.contains(&source.source_hash));
    }
}
