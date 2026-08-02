use crate::core::framework::render::ShaderVariantKey;
use crate::graphics::scene::resources::{PipelineKey, ResourceStreamer};

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::{
    create_taa_reactive_mask_mesh_pipeline, create_taa_reactive_material_mask_mesh_pipeline,
};
use super::MeshPipelineCache;
use super::shader_source::mesh_pipeline_taa_reactive_mask_template_source_for_geometry_descriptor_with_streamer;

const TAA_REACTIVE_MASK_MESH_SHADER_KEY_PREFIX: &str = "zircon.builtin.taa-reactive-mask-mesh@1";

impl MeshPipelineCache {
    fn ensure_taa_reactive_mask_pipeline<'a>(
        &'a mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
        key: &PipelineKey,
        shader_variant_key: &ShaderVariantKey,
    ) -> Option<&'a wgpu::RenderPipeline> {
        let geometry_source = self.geometry_source_descriptor_for_variant(shader_variant_key)?;
        let shader_source =
            match mesh_pipeline_taa_reactive_mask_template_source_for_geometry_descriptor_with_streamer(
                streamer,
                key,
                &geometry_source,
            ) {
                Ok(source) => source,
                Err(error) => {
                    self.record_shader_variant_assembly_error(shader_variant_key, error);
                    return None;
                }
            };
        let shader_key =
            taa_reactive_mask_mesh_shader_key(shader_variant_key, &shader_source.source_hash);
        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        if !self.shader_modules.contains_key(&shader_key) {
            let source =
                self.mesh_pipeline_shader_source_with_cache(shader_source, shader_variant_key)?;
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-taa-reactive-mask-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            self.shader_modules.insert(shader_key.clone(), module);
        }
        if !self
            .taa_reactive_mask_mesh_pipelines
            .contains_key(&variant_id)
        {
            let shader = self
                .shader_modules
                .get(&shader_key)
                .expect("TAA reactive mask mesh shader module cached");
            let pipeline = create_taa_reactive_mask_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                wgpu::TextureFormat::R8Unorm,
                key,
                self.runtime_pipeline_cache.cache(),
            );
            self.taa_reactive_mask_mesh_pipelines
                .insert(variant_id, pipeline);
        }
        self.track_pipeline_creation_error_scope(shader_variant_key, error_scope);
        self.taa_reactive_mask_mesh_pipelines.get(&variant_id)
    }

    pub(crate) fn ensure_taa_reactive_mask_pipeline_for_variant<'a>(
        &'a mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
    ) -> Option<&'a wgpu::RenderPipeline> {
        if self.cached_taa_reactive_pipeline(variant_id).is_some() {
            return self.cached_taa_reactive_pipeline(variant_id);
        }
        let (kind, pipeline_key, shader_variant_key) =
            self.pipeline_and_shader_key_for_variant(variant_id)?;
        match kind {
            MeshPassPipelineKind::TaaReactiveMask => self.ensure_taa_reactive_mask_pipeline(
                device,
                streamer,
                variant_id,
                &pipeline_key,
                &shader_variant_key,
            ),
            MeshPassPipelineKind::TaaReactiveMaterialMask => self
                .ensure_taa_reactive_material_mask_pipeline(
                    device,
                    streamer,
                    variant_id,
                    &pipeline_key,
                    &shader_variant_key,
                ),
            _ => None,
        }
    }

    fn cached_taa_reactive_pipeline(
        &self,
        variant_id: MeshPipelineVariantId,
    ) -> Option<&wgpu::RenderPipeline> {
        self.taa_reactive_mask_mesh_pipelines
            .get(&variant_id)
            .or_else(|| {
                self.taa_reactive_material_mask_mesh_pipelines
                    .get(&variant_id)
            })
    }

    fn ensure_taa_reactive_material_mask_pipeline<'a>(
        &'a mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
        key: &PipelineKey,
        shader_variant_key: &ShaderVariantKey,
    ) -> Option<&'a wgpu::RenderPipeline> {
        let geometry_source = self.geometry_source_descriptor_for_variant(shader_variant_key)?;
        let shader_source =
            match mesh_pipeline_taa_reactive_mask_template_source_for_geometry_descriptor_with_streamer(
                streamer,
                key,
                &geometry_source,
            ) {
                Ok(source) => source,
                Err(error) => {
                    self.record_shader_variant_assembly_error(shader_variant_key, error);
                    return None;
                }
            };
        let shader_key =
            taa_reactive_mask_mesh_shader_key(shader_variant_key, &shader_source.source_hash);
        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        if !self.shader_modules.contains_key(&shader_key) {
            let source =
                self.mesh_pipeline_shader_source_with_cache(shader_source, shader_variant_key)?;
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-taa-reactive-mask-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            self.shader_modules.insert(shader_key.clone(), module);
        }
        if !self
            .taa_reactive_material_mask_mesh_pipelines
            .contains_key(&variant_id)
        {
            let shader = self
                .shader_modules
                .get(&shader_key)
                .expect("TAA reactive mask mesh shader module cached");
            let pipeline = create_taa_reactive_material_mask_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                wgpu::TextureFormat::R8Unorm,
                key,
                self.runtime_pipeline_cache.cache(),
            );
            self.taa_reactive_material_mask_mesh_pipelines
                .insert(variant_id, pipeline);
        }
        self.track_pipeline_creation_error_scope(shader_variant_key, error_scope);
        self.taa_reactive_material_mask_mesh_pipelines
            .get(&variant_id)
    }
}

fn taa_reactive_mask_mesh_shader_key(variant_key: &ShaderVariantKey, source_hash: &str) -> String {
    format!(
        "{}#{}#{}",
        TAA_REACTIVE_MASK_MESH_SHADER_KEY_PREFIX,
        variant_key.canonical_string(),
        source_hash
    )
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::ShaderPassType;
    use crate::graphics::scene::resources::default_pipeline_key;

    use super::super::mesh_pipeline_taa_reactive_mask_template_source_for_geometry;
    use super::{TAA_REACTIVE_MASK_MESH_SHADER_KEY_PREFIX, taa_reactive_mask_mesh_shader_key};

    #[test]
    fn taa_reactive_mask_shader_key_includes_shader_variant_identity_and_source_hash() {
        let variant_key = default_pipeline_key()
            .shader_variant_key(ShaderPassType::TaaReactiveMask, "wgpu-runtime");
        let source = match mesh_pipeline_taa_reactive_mask_template_source_for_geometry(
            &default_pipeline_key(),
            variant_key.geometry_source,
        ) {
            Ok(source) => source,
            Err(error) => panic!("TAA reactive mask template source assembly failed: {error:?}"),
        };
        let key = taa_reactive_mask_mesh_shader_key(&variant_key, &source.source_hash);

        assert!(key.starts_with(TAA_REACTIVE_MASK_MESH_SHADER_KEY_PREFIX));
        assert!(key.contains(&variant_key.canonical_string()));
        assert!(key.contains("|pass=taa_reactive_mask|"));
        assert!(key.contains(&source.source_hash));
    }
}
