use crate::core::framework::render::ShaderVariantKey;
use crate::graphics::scene::resources::{PipelineKey, ResourceStreamer};
use crate::graphics::shader::{ShaderVariantCacheDiskKey, ShaderVariantCacheDiskLookup};

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::{create_mesh_pipeline, FALLBACK_MESH_SHADER};
use super::MeshPipelineCache;

const MESH_SHADER_TEMPLATE_REVISION: &str = "mesh-template-v1";
const MESH_SHADER_NAGA_VERSION: &str = "naga-29.0.1";
const MESH_SHADER_WGPU_VERSION: &str = "wgpu-29.0.1";

impl MeshPipelineCache {
    pub(crate) fn pipeline_uses_builtin_fallback_shader(
        &self,
        streamer: &ResourceStreamer,
        key: &PipelineKey,
    ) -> bool {
        key.uses_fallback_shader() || streamer.shader_source(&key.shader_id).is_none()
    }

    pub(crate) fn ensure_pipeline_for_variant<'a>(
        &'a mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
    ) -> Option<&'a wgpu::RenderPipeline> {
        let (kind, pipeline_key, shader_variant_key) =
            self.pipeline_and_shader_key_for_variant(variant_id)?;
        if kind != MeshPassPipelineKind::Base {
            return None;
        }
        let shader_key = mesh_shader_module_cache_key(&pipeline_key, &shader_variant_key);
        if !self.shader_modules.contains_key(&shader_key) {
            let source = self.mesh_pipeline_shader_source_with_cache(
                streamer,
                &pipeline_key,
                &shader_variant_key,
            );
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            self.shader_modules.insert(shader_key.clone(), module);
        }
        if !self.mesh_variant_pipelines.contains_key(&variant_id) {
            let shader = self
                .shader_modules
                .get(&shader_key)
                .expect("shader module cached");
            let pipeline = create_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                self.target_format,
                &pipeline_key,
            );
            self.mesh_variant_pipelines.insert(variant_id, pipeline);
        }
        self.mesh_variant_pipelines.get(&variant_id)
    }

    fn mesh_pipeline_shader_source_with_cache(
        &mut self,
        streamer: &ResourceStreamer,
        key: &PipelineKey,
        variant_key: &ShaderVariantKey,
    ) -> String {
        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            variant_key,
            [mesh_pipeline_source_hash(streamer, key)],
        );
        match self.shader_variant_disk_cache.lookup(&disk_key) {
            ShaderVariantCacheDiskLookup::Hit(entry) => {
                self.record_shader_variant_disk_hit();
                entry.wgsl_source
            }
            ShaderVariantCacheDiskLookup::Miss => {
                let source = mesh_pipeline_shader_source(streamer, key).to_string();
                self.record_shader_variant_compile_miss();
                match self.shader_variant_disk_cache.write(
                    &disk_key,
                    &source,
                    MESH_SHADER_TEMPLATE_REVISION,
                    MESH_SHADER_NAGA_VERSION,
                    MESH_SHADER_WGPU_VERSION,
                ) {
                    Ok(_) => self.record_shader_variant_disk_write(),
                    Err(_) => self.record_shader_variant_disk_error(),
                }
                source
            }
            ShaderVariantCacheDiskLookup::Error(_) => {
                self.record_shader_variant_disk_error();
                mesh_pipeline_shader_source(streamer, key).to_string()
            }
        }
    }
}

fn mesh_shader_module_cache_key(key: &PipelineKey, variant_key: &ShaderVariantKey) -> String {
    format!(
        "{}@{}#{}",
        key.shader_id,
        key.shader_revision,
        variant_key.canonical_string()
    )
}

fn mesh_pipeline_shader_source<'a>(streamer: &'a ResourceStreamer, key: &PipelineKey) -> &'a str {
    if key.uses_fallback_shader() {
        FALLBACK_MESH_SHADER
    } else {
        streamer
            .shader_source(&key.shader_id)
            .unwrap_or(FALLBACK_MESH_SHADER)
    }
}

fn mesh_pipeline_source_hash(streamer: &ResourceStreamer, key: &PipelineKey) -> String {
    blake3::hash(mesh_pipeline_shader_source(streamer, key).as_bytes())
        .to_hex()
        .to_string()
}
