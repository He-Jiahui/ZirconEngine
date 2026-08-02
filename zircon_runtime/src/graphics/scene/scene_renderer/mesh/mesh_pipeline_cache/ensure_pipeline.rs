use crate::asset::AssetReference;
use crate::core::framework::render::{ShaderQualityTier, ShaderVariantKey};
use crate::graphics::pipeline::{
    PipelineAsyncCompileError, PipelineAsyncQueueResult, PipelinePlaceholderPolicy,
};
use crate::graphics::scene::resources::{
    PipelineKey, ResourceStreamer, default_pipeline_key, fallback_shader_uri,
};
use crate::graphics::shader::{ShaderVariantCacheDiskKey, ShaderVariantCacheDiskLookup};
use crate::graphics::types::GraphicsError;

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::create_mesh_pipeline;
use super::shader_source::{
    MeshPipelineShaderSource, mesh_pipeline_shader_source_for_geometry_descriptor_with_features,
};
use super::{AsyncBasePipelineProduct, MeshPipelineCache};

const MESH_SHADER_NAGA_VERSION: &str = "naga-29.0.1";
const MESH_SHADER_WGPU_VERSION: &str = "wgpu-29.0.1";
const BASE_PIPELINE_PLACEHOLDER_POLICY: PipelinePlaceholderPolicy =
    PipelinePlaceholderPolicy::SkipDraw;

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
        self.ensure_pipeline_for_variant_with_async_placeholder(device, streamer, variant_id, true)
    }

    fn ensure_pipeline_for_variant_with_async_placeholder<'a>(
        &'a mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
        allow_async_placeholder: bool,
    ) -> Option<&'a wgpu::RenderPipeline> {
        let allow_async_placeholder =
            self.allow_async_base_pipeline_placeholder(allow_async_placeholder);
        self.drain_ready_base_pipelines();
        if self.mesh_variant_pipelines.contains_key(&variant_id) {
            return self.mesh_variant_pipelines.get(&variant_id);
        }
        let (kind, pipeline_key, shader_variant_key) =
            self.pipeline_and_shader_key_for_variant(variant_id)?;
        if kind != MeshPassPipelineKind::Base {
            return None;
        }
        if self.async_base_pipeline_is_pending(variant_id) {
            if allow_async_placeholder {
                return None;
            }
            self.finish_pending_base_pipeline_variant(variant_id);
            if self.mesh_variant_pipelines.contains_key(&variant_id) {
                return self.mesh_variant_pipelines.get(&variant_id);
            }
        }
        let geometry_source = self.geometry_source_descriptor_for_variant(&shader_variant_key)?;
        let shader_source = match mesh_pipeline_shader_source_for_geometry_descriptor_with_features(
            streamer,
            &pipeline_key,
            &geometry_source,
            shader_variant_key.features,
        ) {
            Ok(source) => source,
            Err(error) => {
                self.record_shader_variant_assembly_error(&shader_variant_key, error);
                return None;
            }
        };
        let shader_key = mesh_shader_module_cache_key(
            &pipeline_key,
            &shader_variant_key,
            &shader_source.source_hash,
        );
        let cached_shader = self.shader_modules.get(&shader_key).cloned();
        let compiled_source = if cached_shader.is_none() {
            Some(self.mesh_pipeline_shader_source_with_cache(shader_source, &shader_variant_key)?)
        } else {
            None
        };
        if allow_async_placeholder
            && self.allow_async_pipeline_compile
            && self.async_base_pipeline_compiler.is_some()
        {
            let device = device.clone();
            let layout = self.mesh_pipeline_layout.clone();
            let target_format = self.target_format;
            let async_pipeline_key = pipeline_key.clone();
            let async_shader_key = shader_key.clone();
            let async_cached_shader = cached_shader.clone();
            let async_compiled_source = compiled_source.clone();
            let async_runtime_pipeline_cache = self.runtime_pipeline_cache.cache().cloned();
            let queue_result = self
                .async_base_pipeline_compiler
                .as_mut()
                .expect("async pipeline compiler checked above")
                .try_queue(variant_id, move || {
                    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
                    let shader_module = async_cached_shader.unwrap_or_else(|| {
                        device.create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("zircon-mesh-shader"),
                            source: wgpu::ShaderSource::Wgsl(
                                async_compiled_source
                                    .expect("uncached async shader source")
                                    .into(),
                            ),
                        })
                    });
                    let pipeline = create_mesh_pipeline(
                        &device,
                        &layout,
                        &shader_module,
                        target_format,
                        &async_pipeline_key,
                        async_runtime_pipeline_cache.as_ref(),
                    );
                    if let Some(error) = pollster::block_on(error_scope.pop()) {
                        return Err(error.to_string());
                    }
                    Ok(AsyncBasePipelineProduct {
                        shader_key: async_shader_key,
                        shader_module,
                        pipeline,
                    })
                });
            match queue_result {
                PipelineAsyncQueueResult::Queued => {
                    self.async_variant_first_frame_miss_count =
                        self.async_variant_first_frame_miss_count.saturating_add(1);
                    debug_assert_eq!(BASE_PIPELINE_PLACEHOLDER_POLICY.label(), "skip_draw");
                    return None;
                }
                PipelineAsyncQueueResult::AlreadyPending | PipelineAsyncQueueResult::Full => {
                    return None;
                }
                PipelineAsyncQueueResult::WorkerUnavailable => {}
            }
        }
        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        if !self.shader_modules.contains_key(&shader_key) {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(
                    compiled_source
                        .expect("uncached synchronous shader source")
                        .into(),
                ),
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
                self.runtime_pipeline_cache.cache(),
            );
            self.mesh_variant_pipelines.insert(variant_id, pipeline);
        }
        self.track_pipeline_creation_error_scope(&shader_variant_key, error_scope);
        self.mesh_variant_pipelines.get(&variant_id)
    }

    pub(crate) fn set_async_pipeline_compile_enabled(&mut self, enabled: bool) {
        if self.allow_async_pipeline_compile && !enabled {
            self.finish_pending_base_pipelines();
        }
        self.allow_async_pipeline_compile = enabled;
    }

    pub(crate) const fn async_pipeline_compile_enabled(&self) -> bool {
        self.allow_async_pipeline_compile
    }

    fn allow_async_base_pipeline_placeholder(&self, requested: bool) -> bool {
        requested && !self.force_synchronous_base_pipeline_compile
    }

    fn async_base_pipeline_is_pending(&self, variant_id: MeshPipelineVariantId) -> bool {
        self.async_base_pipeline_compiler
            .as_ref()
            .is_some_and(|compiler| compiler.is_pending(&variant_id))
    }

    fn drain_ready_base_pipelines(&mut self) {
        let mut completions = Vec::new();
        if let Some(compiler) = self.async_base_pipeline_compiler.as_mut() {
            compiler.drain_ready(|variant_id, result| completions.push((variant_id, result)));
        }
        self.install_async_base_pipeline_completions(completions);
    }

    fn finish_pending_base_pipelines(&mut self) {
        let mut completions = Vec::new();
        if let Some(compiler) = self.async_base_pipeline_compiler.as_mut() {
            compiler.finish_pending(|variant_id, result| completions.push((variant_id, result)));
        }
        self.install_async_base_pipeline_completions(completions);
    }

    fn finish_pending_base_pipeline_variant(&mut self, variant_id: MeshPipelineVariantId) -> bool {
        let mut completions = Vec::new();
        if let Some(compiler) = self.async_base_pipeline_compiler.as_mut() {
            compiler.finish_pending_through(&variant_id, |variant_id, result| {
                completions.push((variant_id, result));
            });
        }
        self.install_async_base_pipeline_completions(completions);
        self.mesh_variant_pipelines.contains_key(&variant_id)
    }

    fn install_async_base_pipeline_completions(
        &mut self,
        completions: Vec<(
            MeshPipelineVariantId,
            Result<super::AsyncBasePipelineCompileResult, PipelineAsyncCompileError>,
        )>,
    ) {
        for (variant_id, completion) in completions {
            let product = match completion {
                Ok(Ok(product)) => product,
                Ok(Err(error)) => {
                    if let Some((_, _, shader_variant_key)) =
                        self.pipeline_and_shader_key_for_variant(variant_id)
                    {
                        self.record_shader_variant_pipeline_creation_message(
                            &shader_variant_key,
                            error,
                        );
                    }
                    continue;
                }
                Err(error) => {
                    if let Some((_, _, shader_variant_key)) =
                        self.pipeline_and_shader_key_for_variant(variant_id)
                    {
                        self.record_shader_variant_pipeline_creation_error(
                            &shader_variant_key,
                            error,
                        );
                    }
                    continue;
                }
            };
            self.shader_modules
                .entry(product.shader_key)
                .or_insert(product.shader_module);
            self.mesh_variant_pipelines
                .insert(variant_id, product.pipeline);
        }
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
        self.pipeline_variant_registry
            .enable_environment_only_pbr_base_profile();
        let variant_id = self.resolve_variant(
            MeshPassPipelineKind::Base,
            &pipeline_key,
            ShaderQualityTier::default(),
        );
        let pipeline_creation_started = std::time::Instant::now();
        let cache_hit = if self.mesh_variant_pipelines.contains_key(&variant_id) {
            true
        } else if self.async_base_pipeline_is_pending(variant_id) {
            self.finish_pending_base_pipeline_variant(variant_id)
        } else {
            false
        };
        let pipeline_ready = self
            .ensure_pipeline_for_variant_with_async_placeholder(device, streamer, variant_id, false)
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
    ) -> Option<String> {
        let MeshPipelineShaderSource {
            wgsl_source,
            cache_content_hashes,
            template_revision,
            segments,
            ..
        } = source;
        self.queue_shader_source_validation(variant_key, wgsl_source.clone(), segments);
        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            variant_key,
            cache_content_hashes.iter().map(String::as_str),
        );
        let compiled_source = match self.shader_variant_disk_cache.lookup(&disk_key) {
            ShaderVariantCacheDiskLookup::Hit(entry) if entry.wgsl_source == wgsl_source => {
                self.record_shader_variant_disk_hit(variant_key);
                entry.wgsl_source
            }
            ShaderVariantCacheDiskLookup::Hit(_) => {
                self.record_shader_variant_disk_error(variant_key);
                match self.shader_variant_disk_cache.write(
                    &disk_key,
                    &wgsl_source,
                    &template_revision,
                    MESH_SHADER_NAGA_VERSION,
                    MESH_SHADER_WGPU_VERSION,
                ) {
                    Ok(_) => self.record_shader_variant_disk_write(variant_key),
                    Err(_) => self.record_shader_variant_disk_error(variant_key),
                }
                wgsl_source
            }
            ShaderVariantCacheDiskLookup::Miss => {
                self.record_shader_variant_compile_miss(variant_key);
                match self.shader_variant_disk_cache.write(
                    &disk_key,
                    &wgsl_source,
                    &template_revision,
                    MESH_SHADER_NAGA_VERSION,
                    MESH_SHADER_WGPU_VERSION,
                ) {
                    Ok(_) => self.record_shader_variant_disk_write(variant_key),
                    Err(_) => self.record_shader_variant_disk_error(variant_key),
                }
                wgsl_source
            }
            ShaderVariantCacheDiskLookup::Error(_) => {
                self.record_shader_variant_disk_error(variant_key);
                wgsl_source
            }
        };
        Some(compiled_source)
    }
}

impl Drop for MeshPipelineCache {
    fn drop(&mut self) {
        self.finish_pending_base_pipelines();
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
