use crate::asset::AssetReference;
use crate::core::framework::render::{ShaderQualityTier, ShaderVariantKey};
use crate::graphics::scene::resources::{
    default_pipeline_key, fallback_shader_uri, PipelineKey, ResourceStreamer,
};
use crate::graphics::shader::{ShaderVariantCacheDiskKey, ShaderVariantCacheDiskLookup};
use crate::graphics::types::GraphicsError;

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::create_mesh_pipeline;
use super::shader_source::{
    mesh_pipeline_shader_source_for_geometry_descriptor, MeshPipelineShaderSource,
};
use super::MeshPipelineCache;

const MESH_SHADER_NAGA_VERSION: &str = "naga-29.0.1";
const MESH_SHADER_WGPU_VERSION: &str = "wgpu-29.0.1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EnvironmentOnlyPbrBasePipelinePrewarmReport {
    pipeline_ready: bool,
    cache_hit: bool,
    shader_source_resolution: std::time::Duration,
    pipeline_creation: std::time::Duration,
    elapsed: std::time::Duration,
}

impl EnvironmentOnlyPbrBasePipelinePrewarmReport {
    pub(crate) const fn pipeline_ready(self) -> bool {
        self.pipeline_ready
    }

    pub(crate) const fn cache_hit(self) -> bool {
        self.cache_hit
    }

    pub(crate) const fn created_pipeline(self) -> bool {
        self.pipeline_ready && !self.cache_hit
    }

    pub(crate) const fn shader_source_resolution(self) -> std::time::Duration {
        self.shader_source_resolution
    }

    pub(crate) const fn pipeline_creation(self) -> std::time::Duration {
        self.pipeline_creation
    }

    pub(crate) const fn elapsed(self) -> std::time::Duration {
        self.elapsed
    }
}

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
        if self.mesh_variant_pipelines.contains_key(&variant_id) {
            return self.mesh_variant_pipelines.get(&variant_id);
        }
        let (kind, pipeline_key, shader_variant_key) =
            self.pipeline_and_shader_key_for_variant(variant_id)?;
        if kind != MeshPassPipelineKind::Base {
            return None;
        }
        let geometry_source = self.geometry_source_descriptor_for_variant(&shader_variant_key)?;
        let shader_source = match mesh_pipeline_shader_source_for_geometry_descriptor(
            streamer,
            &pipeline_key,
            &geometry_source,
        ) {
            Ok(source) => source,
            Err(_) => {
                self.record_shader_variant_disk_error(&shader_variant_key);
                return None;
            }
        };
        let shader_key = mesh_shader_module_cache_key(
            &pipeline_key,
            &shader_variant_key,
            &shader_source.source_hash,
        );
        if !self.shader_modules.contains_key(&shader_key) {
            let source =
                self.mesh_pipeline_shader_source_with_cache(shader_source, &shader_variant_key);
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

    /// Creates the exact static Standard-PBR Base variant submitted by the
    /// environment-only viewer's `BaseScenePass`.
    ///
    /// This only fills the renderer-owned cache; it does not encode, submit,
    /// present, or read back a frame.
    pub(crate) fn prewarm_environment_only_pbr_base_pipeline(
        &mut self,
        device: &wgpu::Device,
        streamer: &mut ResourceStreamer,
    ) -> Result<EnvironmentOnlyPbrBasePipelinePrewarmReport, GraphicsError> {
        let started = std::time::Instant::now();
        let mut pipeline_key = default_pipeline_key();
        let shader_source_started = std::time::Instant::now();
        let (shader_id, shader_revision, _) =
            streamer.ensure_shader_source(&AssetReference::from_locator(fallback_shader_uri()))?;
        let shader_source_resolution = shader_source_started.elapsed();
        if shader_id != pipeline_key.shader_id {
            return Err(GraphicsError::Asset(format!(
                "environment-only PBR prewarm resolved {shader_id}, expected {}",
                pipeline_key.shader_id
            )));
        }
        pipeline_key.shader_revision = shader_revision;
        pipeline_key.receive_shadows = false;
        let variant_id = self.resolve_variant(
            MeshPassPipelineKind::Base,
            &pipeline_key,
            ShaderQualityTier::default(),
        );
        let cache_hit = self.mesh_variant_pipelines.contains_key(&variant_id);
        let pipeline_creation_started = std::time::Instant::now();
        let pipeline_ready = self
            .ensure_pipeline_for_variant(device, streamer, variant_id)
            .is_some();
        let pipeline_creation = pipeline_creation_started.elapsed();
        if !pipeline_ready {
            return Err(GraphicsError::Asset(
                "environment-only PBR prewarm could not create its Base pipeline".to_string(),
            ));
        }
        Ok(EnvironmentOnlyPbrBasePipelinePrewarmReport {
            pipeline_ready,
            cache_hit,
            shader_source_resolution,
            pipeline_creation,
            elapsed: started.elapsed(),
        })
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn mesh_pipeline_shader_source_with_cache(
        &mut self,
        source: MeshPipelineShaderSource,
        variant_key: &ShaderVariantKey,
    ) -> String {
        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            variant_key,
            source.cache_content_hashes.iter().map(String::as_str),
        );
        match self.shader_variant_disk_cache.lookup(&disk_key) {
            ShaderVariantCacheDiskLookup::Hit(entry) => {
                self.record_shader_variant_disk_hit(variant_key);
                entry.wgsl_source
            }
            ShaderVariantCacheDiskLookup::Miss => {
                self.record_shader_variant_compile_miss(variant_key);
                match self.shader_variant_disk_cache.write(
                    &disk_key,
                    &source.wgsl_source,
                    &source.template_revision,
                    MESH_SHADER_NAGA_VERSION,
                    MESH_SHADER_WGPU_VERSION,
                ) {
                    Ok(_) => self.record_shader_variant_disk_write(variant_key),
                    Err(_) => self.record_shader_variant_disk_error(variant_key),
                }
                source.wgsl_source
            }
            ShaderVariantCacheDiskLookup::Error(_) => {
                self.record_shader_variant_disk_error(variant_key);
                source.wgsl_source
            }
        }
    }
}

fn mesh_shader_module_cache_key(
    key: &PipelineKey,
    variant_key: &ShaderVariantKey,
    source_hash: &str,
) -> String {
    format!(
        "{}@{}#{}#{}",
        key.shader_id,
        key.shader_revision,
        variant_key.canonical_string(),
        source_hash
    )
}

#[cfg(all(test, feature = "dynamic-api"))]
#[path = "ensure_pipeline/tests.rs"]
mod tests;
