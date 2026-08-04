use std::collections::{HashMap, HashSet};

use crate::core::framework::render::{
    GEOMETRY_SOURCE_ID_STATIC_MESH, GeometrySourceDescriptor, GeometrySourceId,
    ShaderPipelineDiagnosticStage, ShaderQualityTier, ShaderVariantKey, ShaderVariantMissReport,
};
use crate::graphics::pipeline::{
    PipelineAsyncCompiler, PipelineAsyncQueueResult, RuntimePipelineCache,
};
use crate::graphics::scene::resources::PipelineKey;
use crate::graphics::scene::scene_renderer::environment::{
    SceneLightmapResources, SceneReflectionProbeResources,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshPassPipelineKind, MeshPipelineVariantId,
};
use crate::graphics::shader::ShaderVariantCacheDisk;

use super::{MeshPipelineVariantRegistry, MeshPipelineVariantResolver};

pub(in crate::graphics::scene::scene_renderer::mesh) const MAX_ASYNC_BASE_PIPELINES_IN_FLIGHT:
    usize = 64;
pub(in crate::graphics::scene::scene_renderer::mesh) const MAX_ASYNC_SHADER_SOURCE_VALIDATIONS_IN_FLIGHT: usize = 64;
const MAX_PENDING_PIPELINE_CREATION_DIAGNOSTICS: usize = 64;

pub(in crate::graphics::scene::scene_renderer::mesh) struct AsyncBasePipelineProduct {
    pub(super) shader_key: String,
    pub(super) shader_module: wgpu::ShaderModule,
    pub(super) pipeline: wgpu::RenderPipeline,
}

pub(in crate::graphics::scene::scene_renderer::mesh) type AsyncBasePipelineCompileResult =
    Result<AsyncBasePipelineProduct, String>;

#[derive(Clone, Copy)]
pub(in crate::graphics::scene::scene_renderer::mesh) enum PipelineCreationTarget {
    MeshPass(MeshPassPipelineKind),
    Oit,
}

pub(super) struct PendingPipelineCreationDiagnostic {
    shader_variant_key: ShaderVariantKey,
    target: PipelineCreationTarget,
    variant_id: MeshPipelineVariantId,
    shader_key: String,
    error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ShaderSourceValidationKey {
    shader_variant_key: ShaderVariantKey,
    source_identity: String,
}

pub(crate) struct MeshPipelineCache {
    pub(in crate::graphics::scene::scene_renderer::mesh) target_format: wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer::mesh) mesh_pipeline_layout: wgpu::PipelineLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) oit_fragment_store_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) oit_mesh_pipeline_layout:
        wgpu::PipelineLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_receiver_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_compare_sampler:
        wgpu::Sampler,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_light_grid_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_light_grid_empty_zbins_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_light_grid_empty_tile_masks_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_atlas_fallback_slot_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_atlas_fallback_globals_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) fallback_shadow_atlas_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_volumetric_apply:
        crate::graphics::scene::scene_renderer::advanced_lighting::froxel::VolumetricApplyFallbackResources,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_volumetric_disabled_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) transmission_scene_color:
        crate::graphics::scene::scene_renderer::advanced_lighting::transmission::TransmissionSceneColorFallbackResources,
    pub(in crate::graphics::scene::scene_renderer) light_cookies:
        crate::graphics::scene::scene_renderer::advanced_lighting::light_cookie::LightCookieAtlasResources,
    pub(in crate::graphics::scene::scene_renderer) irradiance_volume:
        crate::graphics::scene::scene_renderer::advanced_lighting::irradiance_volume::IrradianceVolumeResources,
    pub(in crate::graphics::scene::scene_renderer) reflection_probes: SceneReflectionProbeResources,
    pub(in crate::graphics::scene::scene_renderer) lightmaps: SceneLightmapResources,
    pub(in crate::graphics::scene::scene_renderer::mesh) shader_modules:
        HashMap<String, wgpu::ShaderModule>,
    pub(in crate::graphics::scene::scene_renderer::mesh) mesh_variant_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    // A viewer startup explicitly requested nonblocking PSO creation for these
    // variants. They must remain `SkipDraw` if the worker becomes unavailable.
    pub(super) background_base_pipeline_variants: HashSet<MeshPipelineVariantId>,
    // Retain a terminal worker or assembly error so one-shot image capture can
    // report it instead of continuously redrawing a placeholder frame.
    pub(super) background_base_pipeline_failures: HashMap<MeshPipelineVariantId, String>,
    pub(in crate::graphics::scene::scene_renderer::mesh) oit_mesh_variant_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) gbuffer_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) depth_prepass_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) velocity_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) shadow_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) taa_reactive_mask_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) taa_reactive_material_mask_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) pipeline_variant_registry:
        MeshPipelineVariantRegistry,
    pub(in crate::graphics::scene::scene_renderer::mesh) geometry_source_descriptors:
        HashMap<GeometrySourceId, GeometrySourceDescriptor>,
    pub(in crate::graphics::scene::scene_renderer::mesh) shader_variant_disk_cache:
        ShaderVariantCacheDisk,
    // WGPU error-scope futures are not `Send`. Resolve each scope at creation
    // time, then retain its bounded result until the existing frame/prewarm
    // diagnostic consumer can invalidate the affected cache entry.
    pub(super) pending_pipeline_creation_diagnostics: Vec<PendingPipelineCreationDiagnostic>,
    // Naga remapping stays available for authoring diagnostics, but validation
    // is always performed by this bounded worker instead of the frame path.
    pub(super) shader_source_validation_compiler:
        Option<PipelineAsyncCompiler<ShaderSourceValidationKey, Result<(), String>>>,
    // Fields drop in declaration order. Join the compiler before persisting the
    // driver cache so no worker can mutate it while `get_data` is running.
    pub(super) async_base_pipeline_compiler:
        Option<PipelineAsyncCompiler<MeshPipelineVariantId, AsyncBasePipelineCompileResult>>,
    pub(super) runtime_pipeline_cache: RuntimePipelineCache,
    pub(super) allow_async_pipeline_compile: bool,
    pub(super) force_synchronous_base_pipeline_compile: bool,
    pub(super) async_variant_first_frame_miss_count: u32,
}

impl MeshPipelineCache {
    pub(crate) fn resolve_variant(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
        shader_quality: ShaderQualityTier,
    ) -> MeshPipelineVariantId {
        self.resolve_variant_for_geometry(
            kind,
            pipeline_key,
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            shader_quality,
        )
    }

    pub(crate) fn resolve_variant_for_geometry(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
        geometry_source: GeometrySourceId,
        shader_quality: ShaderQualityTier,
    ) -> MeshPipelineVariantId {
        self.pipeline_variant_registry.resolve_variant_for_geometry(
            kind,
            pipeline_key,
            geometry_source,
            shader_quality,
        )
    }

    /// Local reflection providers require the generic environment shader ABI.
    /// This is intentionally one-way for the renderer lifetime: retaining the
    /// generic key after a provider appears avoids variant thrashing when a
    /// preview scene toggles provider visibility.
    pub(in crate::graphics::scene::scene_renderer) fn disable_environment_only_pbr_base_profile(
        &mut self,
    ) {
        self.pipeline_variant_registry
            .disable_environment_only_pbr_base_profile();
        self.force_synchronous_base_pipeline_compile = true;
    }

    pub(crate) fn pipeline_and_shader_key_for_variant(
        &self,
        variant_id: MeshPipelineVariantId,
    ) -> Option<(MeshPassPipelineKind, PipelineKey, ShaderVariantKey)> {
        let key = self.pipeline_variant_registry.key_for_variant(variant_id)?;
        Some((
            key.kind(),
            key.pipeline_key().clone(),
            key.shader_variant_key().clone(),
        ))
    }

    pub(crate) fn register_geometry_source_descriptor(
        &mut self,
        descriptor: GeometrySourceDescriptor,
    ) {
        self.geometry_source_descriptors
            .insert(descriptor.id, descriptor);
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn geometry_source_descriptor(
        &self,
        geometry_source: GeometrySourceId,
    ) -> Option<GeometrySourceDescriptor> {
        self.geometry_source_descriptors
            .get(&geometry_source)
            .cloned()
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn geometry_source_descriptor_for_variant(
        &mut self,
        key: &ShaderVariantKey,
    ) -> Option<GeometrySourceDescriptor> {
        match self.geometry_source_descriptor(key.geometry_source) {
            Some(descriptor) => Some(descriptor),
            None => {
                self.record_shader_variant_disk_error(key);
                None
            }
        }
    }

    pub(crate) fn reset_shader_variant_miss_report(&mut self) {
        self.pipeline_variant_registry.reset_miss_report();
        self.async_variant_first_frame_miss_count = 0;
    }

    pub(crate) fn drain_pipeline_creation_diagnostics(&mut self, device: &wgpu::Device) {
        self.drain_shader_source_validation_diagnostics();
        let _ = device.poll(wgpu::PollType::Poll);
        let mut errors = Vec::new();
        for diagnostic in self.pending_pipeline_creation_diagnostics.drain(..) {
            let PendingPipelineCreationDiagnostic {
                shader_variant_key,
                target,
                variant_id,
                shader_key,
                error,
            } = diagnostic;
            if let Some(error) = error {
                errors.push(((shader_variant_key, target, variant_id, shader_key), error));
            }
        }
        for ((shader_variant_key, target, variant_id, shader_key), error) in errors {
            self.invalidate_pipeline_creation_target(&target, &variant_id, &shader_key);
            self.record_shader_variant_pipeline_creation_message(&shader_variant_key, error);
        }
    }

    pub(crate) fn finish_pipeline_creation_diagnostics_for_variant(
        &mut self,
        device: &wgpu::Device,
        key: &ShaderVariantKey,
    ) -> Result<bool, String> {
        let _ = device.poll(wgpu::PollType::Poll);
        let mut pending = Vec::with_capacity(self.pending_pipeline_creation_diagnostics.len());
        let mut errors = Vec::new();
        let mut matched_scope_count = 0;
        for diagnostic in self.pending_pipeline_creation_diagnostics.drain(..) {
            if diagnostic.shader_variant_key != *key {
                pending.push(diagnostic);
                continue;
            }
            matched_scope_count += 1;
            let PendingPipelineCreationDiagnostic {
                shader_variant_key,
                target,
                variant_id,
                shader_key,
                error,
            } = diagnostic;
            if let Some(error) = error {
                errors.push(((shader_variant_key, target, variant_id, shader_key), error));
            }
        }
        self.pending_pipeline_creation_diagnostics = pending;
        let mut messages = Vec::with_capacity(errors.len());
        for ((shader_variant_key, target, variant_id, shader_key), error) in errors {
            self.invalidate_pipeline_creation_target(&target, &variant_id, &shader_key);
            self.record_shader_variant_pipeline_creation_message(
                &shader_variant_key,
                error.clone(),
            );
            messages.push(error);
        }
        if messages.is_empty() {
            Ok(matched_scope_count != 0)
        } else {
            Err(messages.join("; "))
        }
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn queue_shader_source_validation(
        &mut self,
        key: &ShaderVariantKey,
        source_identity: String,
        wgsl_source: String,
        segments: Vec<crate::graphics::shader::ShaderAssemblySegment>,
    ) {
        let outcome = self
            .shader_source_validation_compiler
            .as_mut()
            .map(|compiler| {
                let key = ShaderSourceValidationKey {
                    shader_variant_key: key.clone(),
                    source_identity,
                };
                compiler.try_queue(key, move || {
                    super::shader_source::MeshPipelineShaderSource::validate_wgsl_with_segments(
                        &wgsl_source,
                        &segments,
                    )
                })
            })
            .unwrap_or(PipelineAsyncQueueResult::WorkerUnavailable);
        match outcome {
            PipelineAsyncQueueResult::Queued | PipelineAsyncQueueResult::AlreadyPending => {}
            PipelineAsyncQueueResult::Full => self.record_shader_variant_validation_diagnostic(
                key,
                "background WGSL validation skipped because its bounded queue is full",
            ),
            PipelineAsyncQueueResult::WorkerUnavailable => {
                self.record_shader_variant_validation_diagnostic(
                    key,
                    "background WGSL validation skipped because its worker is unavailable",
                );
            }
        }
    }

    fn drain_shader_source_validation_diagnostics(&mut self) {
        let mut completions = Vec::new();
        if let Some(compiler) = self.shader_source_validation_compiler.as_mut() {
            compiler.drain_ready(|key, result| completions.push((key, result)));
        }
        for (key, result) in completions {
            match result {
                Ok(Ok(())) => {}
                Ok(Err(message)) => {
                    self.record_shader_variant_validation_error(&key.shader_variant_key, message)
                }
                Err(error) => self.record_shader_variant_validation_error(
                    &key.shader_variant_key,
                    format!("{error:?}"),
                ),
            }
        }
    }

    pub(crate) fn shader_variant_miss_report(&self) -> ShaderVariantMissReport {
        self.pipeline_variant_registry.miss_report()
    }

    pub(crate) fn async_pipeline_compile_pending_count(&self) -> u32 {
        self.async_base_pipeline_compiler
            .as_ref()
            .map_or(0, PipelineAsyncCompiler::pending_count)
            .min(u32::MAX as usize) as u32
    }

    pub(crate) const fn async_variant_first_frame_miss_count(&self) -> u32 {
        self.async_variant_first_frame_miss_count
    }

    pub(crate) fn record_shader_variant_disk_hit(&mut self, key: &ShaderVariantKey) {
        self.pipeline_variant_registry.record_disk_hit(key);
    }

    pub(crate) fn record_shader_variant_disk_write(&mut self, key: &ShaderVariantKey) {
        self.pipeline_variant_registry.record_disk_write(key);
    }

    pub(crate) fn record_shader_variant_disk_error(&mut self, key: &ShaderVariantKey) {
        self.pipeline_variant_registry.record_disk_error(key);
    }

    pub(crate) fn record_shader_variant_assembly_error(
        &mut self,
        key: &ShaderVariantKey,
        error: impl std::fmt::Debug,
    ) {
        self.record_shader_variant_disk_error(key);
        self.pipeline_variant_registry.record_pipeline_diagnostic(
            key,
            ShaderPipelineDiagnosticStage::SourceAssembly,
            format!("{error:?}"),
        );
    }

    pub(crate) fn record_shader_variant_validation_error(
        &mut self,
        key: &ShaderVariantKey,
        message: String,
    ) {
        self.record_shader_variant_disk_error(key);
        self.record_shader_variant_validation_diagnostic(key, message);
    }

    fn record_shader_variant_validation_diagnostic(
        &mut self,
        key: &ShaderVariantKey,
        message: impl Into<String>,
    ) {
        self.pipeline_variant_registry.record_pipeline_diagnostic(
            key,
            ShaderPipelineDiagnosticStage::WgslValidation,
            message,
        );
    }

    pub(crate) fn record_shader_variant_pipeline_creation_error(
        &mut self,
        key: &ShaderVariantKey,
        error: impl std::fmt::Debug,
    ) {
        self.record_shader_variant_pipeline_creation_message(key, format!("{error:?}"));
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn track_pipeline_creation_error_scope(
        &mut self,
        key: &ShaderVariantKey,
        target: PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
        shader_key: String,
        error_scope: wgpu::ErrorScopeGuard,
    ) {
        if self.pending_pipeline_creation_diagnostics.len()
            >= MAX_PENDING_PIPELINE_CREATION_DIAGNOSTICS
        {
            self.invalidate_pipeline_creation_target(&target, &variant_id, &shader_key);
            self.record_shader_variant_pipeline_creation_message(
                key,
                "pipeline creation diagnostic queue is saturated; discarded the newly created pipeline",
            );
            let _ = pollster::block_on(error_scope.pop());
            return;
        }
        self.pending_pipeline_creation_diagnostics
            .push(PendingPipelineCreationDiagnostic {
                shader_variant_key: key.clone(),
                target,
                variant_id,
                shader_key,
                error: pollster::block_on(error_scope.pop()).map(|error| error.to_string()),
            });
    }

    fn invalidate_pipeline_creation_target(
        &mut self,
        target: &PipelineCreationTarget,
        variant_id: &MeshPipelineVariantId,
        shader_key: &str,
    ) {
        match target {
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base) => {
                self.mesh_variant_pipelines.remove(variant_id);
            }
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::GBuffer) => {
                self.gbuffer_mesh_pipelines.remove(variant_id);
            }
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::DepthPrepass) => {
                self.depth_prepass_mesh_pipelines.remove(variant_id);
            }
            PipelineCreationTarget::MeshPass(
                MeshPassPipelineKind::ShadowDepth | MeshPassPipelineKind::ShadowDepthAlphaMask,
            ) => {
                self.shadow_mesh_pipelines.remove(variant_id);
            }
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Velocity) => {
                self.velocity_mesh_pipelines.remove(variant_id);
            }
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::TaaReactiveMask) => {
                self.taa_reactive_mask_mesh_pipelines.remove(variant_id);
            }
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::TaaReactiveMaterialMask) => {
                self.taa_reactive_material_mask_mesh_pipelines
                    .remove(variant_id);
            }
            PipelineCreationTarget::Oit => {
                self.oit_mesh_variant_pipelines.remove(variant_id);
            }
        }
        self.shader_modules.remove(shader_key);
    }

    pub(in crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache) fn record_shader_variant_pipeline_creation_message(
        &mut self,
        key: &ShaderVariantKey,
        message: impl Into<String>,
    ) {
        self.pipeline_variant_registry.record_pipeline_diagnostic(
            key,
            ShaderPipelineDiagnosticStage::PipelineCreation,
            message,
        );
    }

    pub(crate) fn record_shader_variant_compile_miss(&mut self, key: &ShaderVariantKey) {
        self.pipeline_variant_registry.record_compile_miss(key);
    }

    #[cfg(test)]
    pub(crate) fn replace_shader_variant_disk_cache_for_tests(
        &mut self,
        cache: ShaderVariantCacheDisk,
    ) {
        self.shader_variant_disk_cache = cache;
    }
}

impl MeshPipelineVariantResolver for MeshPipelineCache {
    fn resolve_variant_for_geometry(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
        geometry_source: GeometrySourceId,
        shader_quality: ShaderQualityTier,
    ) -> MeshPipelineVariantId {
        MeshPipelineCache::resolve_variant_for_geometry(
            self,
            kind,
            pipeline_key,
            geometry_source,
            shader_quality,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::ShaderPassType;
    use crate::graphics::scene::resources::default_pipeline_key;

    use super::{MeshPipelineCache, ShaderSourceValidationKey};

    #[test]
    fn mesh_pipeline_cache_is_send_for_render_framework_state() {
        fn assert_send<T: Send>() {}

        assert_send::<MeshPipelineCache>();
    }

    fn assert_cache_hit_precedes_variant_projection(
        source: &str,
        function_name: &str,
        cache_lookup: &str,
    ) {
        let function = source
            .split_once(function_name)
            .map(|(_, function)| function)
            .expect("ensure function must exist");
        let cache_lookup = function
            .find(cache_lookup)
            .expect("ensure function must check its pipeline cache");
        let variant_projection = function
            .find("pipeline_and_shader_key_for_variant")
            .expect("ensure function must project its variant on a cache miss");

        assert!(
            cache_lookup < variant_projection,
            "pipeline cache hit must return before variant cloning and shader source assembly"
        );
    }

    #[test]
    fn mesh_pipeline_cache_hits_precede_variant_and_shader_projection() {
        let cases = [
            (
                include_str!("ensure_pipeline.rs"),
                "ensure_pipeline_for_variant",
                "mesh_variant_pipelines",
            ),
            (
                include_str!("ensure_oit_pipeline.rs"),
                "ensure_oit_pipeline_for_base_variant",
                "oit_mesh_variant_pipelines",
            ),
            (
                include_str!("ensure_gbuffer_pipeline.rs"),
                "ensure_gbuffer_pipeline_for_variant",
                "gbuffer_mesh_pipelines",
            ),
            (
                include_str!("ensure_depth_prepass_pipeline.rs"),
                "ensure_depth_prepass_pipeline_for_variant",
                "depth_prepass_mesh_pipelines",
            ),
            (
                include_str!("ensure_shadow_pipeline.rs"),
                "ensure_shadow_pipeline_for_variant",
                "shadow_mesh_pipelines",
            ),
            (
                include_str!("ensure_velocity_pipeline.rs"),
                "ensure_velocity_pipeline_for_variant",
                "velocity_mesh_pipelines",
            ),
            (
                include_str!("ensure_taa_reactive_mask_pipeline.rs"),
                "ensure_taa_reactive_mask_pipeline_for_variant",
                "cached_taa_reactive_pipeline",
            ),
        ];

        for (source, function_name, cache_lookup) in cases {
            assert_cache_hit_precedes_variant_projection(source, function_name, cache_lookup);
        }
    }

    #[test]
    fn shader_source_validation_key_distinguishes_hot_reloaded_source_identity() {
        let variant_key =
            default_pipeline_key().shader_variant_key(ShaderPassType::Forward, "wgpu-runtime");
        let previous = ShaderSourceValidationKey {
            shader_variant_key: variant_key.clone(),
            source_identity: "source-a|segment-a".to_string(),
        };
        let updated = ShaderSourceValidationKey {
            shader_variant_key: variant_key,
            source_identity: "source-b|segment-b".to_string(),
        };

        assert_ne!(previous, updated);
    }
}
