use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::core::framework::render::{
    GEOMETRY_SOURCE_ID_STATIC_MESH, GeometrySourceDescriptor, GeometrySourceId,
    ShaderPipelineDiagnosticStage, ShaderPipelineFallbackAction, ShaderPipelineFallbackState,
    ShaderQualityTier, ShaderVariantKey, ShaderVariantMissReport,
};
use crate::graphics::pipeline::{
    PipelineAdmissionReason, PipelineAsyncCompiler, PipelineUnavailable, RuntimePipelineCache,
};
use crate::graphics::scene::resources::PipelineKey;
use crate::graphics::scene::scene_renderer::environment::{
    SceneLightmapResources, SceneReflectionProbeResources,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshDrawCommand, MeshPassPipelineKind, MeshPipelineVariantId,
};
use crate::graphics::shader::{ShaderVariantCacheDisk, template::ShaderTemplateReflection};
use crate::rhi::{SubmissionStatus, SubmissionTicket};

use super::material_pipeline_generation_admission::MaterialPipelineGenerationAdmissionLedger;
use super::mesh_pipeline_submission_usage::MeshPipelineSubmissionUsage;
use super::mesh_shader_fragment_contract_wgpu::MeshShaderFragmentOutputContracts;
use super::mesh_shader_resource_contract::MeshShaderPipelineLayoutContract;
use super::mesh_shader_vertex_contract::MeshShaderVertexLayoutContract;
use super::pipeline_creation_diagnostics::PendingPipelineCreationDiagnostic;
use super::pipeline_creation_metrics::MeshPipelineCreationMetrics;
use super::pipeline_shader_module_references::PipelineShaderModuleReferences;
use super::shader_source_validation_admission::{
    CachedMeshShaderModule, MeshShaderSourceValidationStates, ShaderSourceValidationKey,
};
use super::{MeshPipelineVariantRegistry, MeshPipelineVariantResolver};

pub(in crate::graphics::scene::scene_renderer::mesh) const MAX_ASYNC_BASE_PIPELINES_IN_FLIGHT:
    usize = 64;
pub(in crate::graphics::scene::scene_renderer::mesh) const MAX_ASYNC_SHADER_SOURCE_VALIDATIONS_IN_FLIGHT: usize = 64;
pub(in crate::graphics::scene::scene_renderer::mesh) const MAX_PENDING_PIPELINE_CREATION_DIAGNOSTICS: usize = 64;

pub(in crate::graphics::scene::scene_renderer::mesh) struct AsyncBasePipelineProduct {
    pub(super) shader_key: String,
    pub(super) shader_module: CachedMeshShaderModule,
    pub(super) validation_key: Option<ShaderSourceValidationKey>,
    pub(super) pipeline: wgpu::RenderPipeline,
}

pub(in crate::graphics::scene::scene_renderer::mesh) type AsyncBasePipelineCompileResult =
    Result<AsyncBasePipelineProduct, String>;

pub(super) struct PipelineFailure {
    pub(super) reason: PipelineAdmissionReason,
    pub(super) message: String,
}

pub(super) struct PipelineUnavailableState {
    pub(super) reason: PipelineAdmissionReason,
    pub(super) since: std::time::Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::graphics::scene::scene_renderer::mesh) enum PipelineCreationTarget {
    MeshPass(MeshPassPipelineKind),
    Oit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct PipelineAdmissionKey {
    pub(super) target: PipelineCreationTarget,
    pub(super) variant_id: MeshPipelineVariantId,
}

impl PipelineAdmissionKey {
    pub(super) const fn new(
        target: PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
    ) -> Self {
        Self { target, variant_id }
    }
}

pub(crate) struct MeshPipelineCache {
    pub(in crate::graphics::scene::scene_renderer::mesh) target_format: wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer::mesh) mesh_pipeline_layout: wgpu::PipelineLayout,
    // The fixed EnvironmentOnly Base shader omits the generic forward receiver
    // ABI at group 1. Keep its dedicated layout separate so generic variants
    // can continue to use the full scene contract.
    pub(in crate::graphics::scene::scene_renderer::mesh) environment_only_mesh_pipeline_layout:
        wgpu::PipelineLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) oit_fragment_store_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) oit_mesh_pipeline_layout:
        wgpu::PipelineLayout,
    pub(super) mesh_shader_resource_contract: MeshShaderPipelineLayoutContract,
    pub(super) environment_only_mesh_shader_resource_contract:
        MeshShaderPipelineLayoutContract,
    pub(super) oit_mesh_shader_resource_contract: MeshShaderPipelineLayoutContract,
    pub(super) mesh_shader_vertex_contract: MeshShaderVertexLayoutContract,
    pub(super) velocity_mesh_shader_vertex_contract: MeshShaderVertexLayoutContract,
    pub(super) mesh_shader_fragment_contracts: MeshShaderFragmentOutputContracts,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_receiver_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) standard_forward_receiver_bind_group_create_count:
        usize,
    pub(in crate::graphics::scene::scene_renderer::mesh) full_forward_receiver_bind_group_create_count:
        usize,
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
        HashMap<String, CachedMeshShaderModule>,
    pub(super) pipeline_shader_module_references: PipelineShaderModuleReferences,
    pub(in crate::graphics::scene::scene_renderer::mesh) mesh_variant_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    // A viewer startup explicitly requested nonblocking PSO creation for these
    // variants. Their draw admission remains deferred until compilation completes.
    pub(super) background_base_pipeline_variants: HashSet<MeshPipelineVariantId>,
    // Failure identity includes the PSO target because OIT deliberately reuses
    // a Base variant id while publishing a distinct pipeline.
    pub(super) pipeline_failures: HashMap<PipelineAdmissionKey, PipelineFailure>,
    pub(super) pipeline_unavailable_states: HashMap<PipelineAdmissionKey, PipelineUnavailableState>,
    // The IOR diagnostic viewer uses the generic Forward Base variant even
    // when the environment-only preview profile remains enabled.
    pub(super) pbr_ior_forward_base_pipeline_variant: Option<MeshPipelineVariantId>,
    pub(in crate::graphics::scene::scene_renderer::mesh) oit_mesh_variant_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) gbuffer_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) depth_prepass_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) hit_proxy_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) velocity_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) shadow_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) taa_reactive_mask_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) taa_reactive_material_mask_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(super) pipeline_submission_usage: MeshPipelineSubmissionUsage,
    pub(super) material_pipeline_generation_admissions:
        MaterialPipelineGenerationAdmissionLedger,
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
        Option<
            PipelineAsyncCompiler<
                ShaderSourceValidationKey,
                Result<Arc<ShaderTemplateReflection>, String>,
            >,
        >,
    pub(super) shader_source_validation_states: MeshShaderSourceValidationStates,
    // Fields drop in declaration order. Join the compiler before persisting the
    // driver cache so no worker can mutate it while `get_data` is running.
    pub(super) async_base_pipeline_compiler:
        Option<PipelineAsyncCompiler<MeshPipelineVariantId, AsyncBasePipelineCompileResult>>,
    pub(super) runtime_pipeline_cache: RuntimePipelineCache,
    pub(super) allow_async_pipeline_compile: bool,
    pub(super) force_synchronous_base_pipeline_compile: bool,
    pub(super) async_variant_first_frame_miss_count: u32,
    pub(super) pipeline_creation_metrics: Arc<MeshPipelineCreationMetrics>,
}

impl MeshPipelineCache {
    pub(super) fn shader_vertex_contract_for_target(
        &self,
        target: PipelineCreationTarget,
    ) -> &MeshShaderVertexLayoutContract {
        match target {
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Velocity) => {
                &self.velocity_mesh_shader_vertex_contract
            }
            PipelineCreationTarget::MeshPass(_) | PipelineCreationTarget::Oit => {
                &self.mesh_shader_vertex_contract
            }
        }
    }

    pub(super) fn shader_resource_contract_for_target(
        &self,
        target: PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
    ) -> &MeshShaderPipelineLayoutContract {
        match target {
            PipelineCreationTarget::Oit => &self.oit_mesh_shader_resource_contract,
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base)
                if !self.base_pipeline_requires_forward_receiver(variant_id) =>
            {
                &self.environment_only_mesh_shader_resource_contract
            }
            PipelineCreationTarget::MeshPass(_) => &self.mesh_shader_resource_contract,
        }
    }

    pub(super) fn bind_pipeline_shader_module_reference(
        &mut self,
        target: PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
        shader_key: &str,
    ) {
        self.pipeline_shader_module_references
            .bind(PipelineAdmissionKey::new(target, variant_id), shader_key);
    }

    pub(super) fn release_pipeline_shader_module_reference(
        &mut self,
        target: PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
        expected_shader_key: &str,
    ) {
        let admission_key = PipelineAdmissionKey::new(target, variant_id);
        debug_assert_eq!(
            self.pipeline_shader_module_references
                .shader_key(admission_key),
            Some(expected_shader_key),
            "cached pipeline must retain its exact shader module reverse edge"
        );
        if let Some(shader_key) = self
            .pipeline_shader_module_references
            .release(admission_key)
        {
            self.shader_modules.remove(shader_key.as_ref());
        }
    }

    pub(crate) fn begin_submission_usage_recording(&mut self) {
        self.pipeline_submission_usage.begin_recording();
    }

    pub(crate) fn record_bound_mesh_pass_pipeline(
        &mut self,
        kind: MeshPassPipelineKind,
        variant_id: MeshPipelineVariantId,
    ) {
        self.pipeline_submission_usage
            .record_bound(PipelineCreationTarget::MeshPass(kind), variant_id);
    }

    pub(crate) fn record_bound_oit_pipeline(&mut self, variant_id: MeshPipelineVariantId) {
        self.pipeline_submission_usage
            .record_bound(PipelineCreationTarget::Oit, variant_id);
    }

    pub(crate) fn bind_recorded_pipeline_usage_to_submission(&mut self, ticket: SubmissionTicket) {
        self.pipeline_submission_usage
            .bind_recorded_to_submission(ticket);
    }

    pub(crate) fn collect_terminal_pipeline_submissions(
        &mut self,
        status_for: impl FnMut(SubmissionTicket) -> Option<SubmissionStatus>,
    ) {
        self.pipeline_submission_usage
            .collect_terminal_submissions(status_for);
    }

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

    pub(crate) fn base_pipeline_requires_forward_receiver(
        &self,
        variant_id: MeshPipelineVariantId,
    ) -> bool {
        self.pipeline_variant_registry
            .base_pipeline_requires_forward_receiver(variant_id)
    }

    pub(crate) fn base_pipeline_for_ready_variant(
        &self,
        variant_id: MeshPipelineVariantId,
    ) -> &wgpu::RenderPipeline {
        self.mesh_variant_pipelines
            .get(&variant_id)
            .expect("Ready Base pipeline admission must retain its pipeline")
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn base_pipeline_layout_for_variant(
        &self,
        variant_id: MeshPipelineVariantId,
    ) -> &wgpu::PipelineLayout {
        if self.base_pipeline_requires_forward_receiver(variant_id) {
            &self.mesh_pipeline_layout
        } else {
            &self.environment_only_mesh_pipeline_layout
        }
    }

    pub(crate) const fn environment_only_pbr_base_profile_enabled(&self) -> bool {
        self.pipeline_variant_registry
            .environment_only_pbr_base_profile_enabled()
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

    pub(crate) fn record_base_pipeline_fallback_for_command(
        &mut self,
        command: &MeshDrawCommand,
        consumer: &'static str,
        unavailable: PipelineUnavailable,
    ) {
        self.record_pipeline_fallback_for_command_variant(
            command,
            command.pipeline_variant_id,
            consumer,
            unavailable,
        );
    }

    pub(crate) fn record_pipeline_fallback_for_command_variant(
        &mut self,
        command: &MeshDrawCommand,
        pipeline_variant_id: MeshPipelineVariantId,
        consumer: &'static str,
        unavailable: PipelineUnavailable,
    ) {
        let (state, action) = if unavailable.reason().is_terminal() {
            (
                ShaderPipelineFallbackState::Failed,
                ShaderPipelineFallbackAction::RejectDraw,
            )
        } else {
            (
                ShaderPipelineFallbackState::Deferred,
                ShaderPipelineFallbackAction::DeferDraw,
            )
        };
        self.pipeline_variant_registry.record_pipeline_fallback(
            pipeline_variant_id,
            command.source_entity,
            consumer,
            state,
            action,
            unavailable.reason().label(),
            duration_microseconds_saturating(unavailable.state_age()),
        );
    }

    pub(crate) fn shader_variant_miss_report(&self) -> ShaderVariantMissReport {
        let mut report = self.pipeline_variant_registry.miss_report();
        report.record_cached_gpu_object_counts(
            self.mesh_variant_pipelines.len()
                + self.oit_mesh_variant_pipelines.len()
                + self.gbuffer_mesh_pipelines.len()
                + self.depth_prepass_mesh_pipelines.len()
                + self.hit_proxy_mesh_pipelines.len()
                + self.velocity_mesh_pipelines.len()
                + self.shadow_mesh_pipelines.len()
                + self.taa_reactive_mask_mesh_pipelines.len()
                + self.taa_reactive_material_mask_mesh_pipelines.len(),
            self.shader_modules.len(),
        );
        let creation_metrics = self.pipeline_creation_metrics.snapshot();
        report.record_gpu_object_creation_totals(
            creation_metrics.render_pipeline_creation_count,
            creation_metrics.shader_module_creation_count,
            creation_metrics.render_pipeline_creation_cpu_microseconds,
            creation_metrics.shader_module_creation_cpu_microseconds,
        );
        report.record_async_base_pipeline_queue_wait_totals(
            creation_metrics.async_base_pipeline_queue_wait_count,
            creation_metrics.async_base_pipeline_queue_wait_microseconds,
        );
        report.record_shader_source_validation_metrics(creation_metrics.shader_source_validation);
        for target in crate::core::framework::render::ShaderPipelineTarget::ALL {
            report.record_pipeline_target_runtime_metrics(
                target,
                creation_metrics.pipeline_targets[target.index()],
            );
        }
        report
    }

    pub(super) fn record_observed_shader_source(
        &self,
        target: PipelineCreationTarget,
        source_hash: &str,
    ) {
        self.pipeline_creation_metrics
            .record_observed_shader_source(target, source_hash);
    }

    pub(super) fn record_render_pipeline_creation(
        &self,
        target: PipelineCreationTarget,
        elapsed: std::time::Duration,
    ) {
        self.pipeline_creation_metrics
            .record_render_pipeline_creation(target, elapsed);
    }

    pub(super) fn record_shader_module_creation(
        &self,
        target: PipelineCreationTarget,
        elapsed: std::time::Duration,
    ) {
        self.pipeline_creation_metrics
            .record_shader_module_creation(target, elapsed);
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

    pub(super) fn record_shader_variant_validation_diagnostic(
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

pub(super) fn duration_microseconds_saturating(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_micros()).unwrap_or(u64::MAX)
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

    use super::super::pipeline_creation_metrics::MeshPipelineCreationMetrics;
    use super::{
        MeshPassPipelineKind, MeshPipelineCache, MeshPipelineVariantId, PipelineAdmissionKey,
        PipelineCreationTarget, ShaderSourceValidationKey, duration_microseconds_saturating,
    };

    #[test]
    fn pipeline_admission_key_distinguishes_base_and_oit_for_the_same_variant() {
        let variant_id = MeshPipelineVariantId::new(17);
        let base = PipelineAdmissionKey::new(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            variant_id,
        );
        let oit = PipelineAdmissionKey::new(PipelineCreationTarget::Oit, variant_id);

        assert_ne!(base, oit);
        let mut targets = std::collections::HashSet::new();
        targets.insert(base);
        targets.insert(oit);
        assert_eq!(targets.len(), 2);
    }

    #[test]
    fn pipeline_creation_duration_conversion_saturates_at_u64_microseconds() {
        assert_eq!(
            duration_microseconds_saturating(std::time::Duration::from_micros(17)),
            17
        );
        assert_eq!(
            duration_microseconds_saturating(std::time::Duration::from_secs(u64::MAX)),
            u64::MAX
        );
    }

    #[test]
    fn pipeline_creation_metrics_accumulate_creation_calls_and_cpu_time() {
        let metrics = MeshPipelineCreationMetrics::default();
        let target = PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base);
        metrics.record_render_pipeline_creation(target, std::time::Duration::from_micros(23));
        metrics.record_shader_module_creation(target, std::time::Duration::from_micros(17));
        metrics.record_async_base_pipeline_queue_wait(std::time::Duration::from_micros(11));

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.render_pipeline_creation_count, 1);
        assert_eq!(snapshot.shader_module_creation_count, 1);
        assert_eq!(snapshot.render_pipeline_creation_cpu_microseconds, 23);
        assert_eq!(snapshot.shader_module_creation_cpu_microseconds, 17);
        assert_eq!(snapshot.async_base_pipeline_queue_wait_count, 1);
        assert_eq!(snapshot.async_base_pipeline_queue_wait_microseconds, 11);
    }

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
                "ensure_pipeline_admission_for_variant",
                "mesh_variant_pipelines",
            ),
            (
                include_str!("ensure_oit_pipeline.rs"),
                "ensure_oit_pipeline_admission_for_base_variant",
                "oit_mesh_variant_pipelines",
            ),
            (
                include_str!("ensure_gbuffer_pipeline.rs"),
                "ensure_gbuffer_pipeline_admission_for_variant",
                "gbuffer_mesh_pipelines",
            ),
            (
                include_str!("ensure_depth_prepass_pipeline.rs"),
                "ensure_depth_prepass_pipeline_admission_for_variant",
                "depth_prepass_mesh_pipelines",
            ),
            (
                include_str!("ensure_shadow_pipeline.rs"),
                "ensure_shadow_pipeline_admission_for_variant",
                "shadow_mesh_pipelines",
            ),
            (
                include_str!("ensure_velocity_pipeline.rs"),
                "ensure_velocity_pipeline_admission_for_variant",
                "velocity_mesh_pipelines",
            ),
            (
                include_str!("ensure_taa_reactive_mask_pipeline.rs"),
                "ensure_taa_reactive_pipeline_admission_for_variant",
                "taa_reactive_pipeline_is_cached",
            ),
        ];

        for (source, function_name, cache_lookup) in cases {
            assert_cache_hit_precedes_variant_projection(source, function_name, cache_lookup);
        }
    }

    #[test]
    fn gbuffer_pipeline_consumer_uses_typed_admission_without_frame_path_expect() {
        let cache = include_str!("ensure_gbuffer_pipeline.rs");
        let consumer =
            include_str!("../../deferred/deferred_scene_resources/record_gbuffer_geometry.rs");

        assert!(cache.contains("gbuffer_variant_admission_for_command_variant"));
        assert!(cache.contains("ensure_gbuffer_pipeline_admission_for_variant"));
        assert!(cache.contains("PipelineAdmission<()>"));
        assert!(cache.contains("gbuffer_pipeline_for_ready_variant"));
        assert!(consumer.contains("PipelineAdmission::Ready"));
        assert!(consumer.contains("record_pipeline_fallback_for_command_variant"));
        assert!(!consumer.contains("deferred GBuffer command must resolve a mesh pipeline"));
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
