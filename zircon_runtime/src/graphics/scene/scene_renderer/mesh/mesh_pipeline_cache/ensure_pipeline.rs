use crate::asset::AssetReference;
use crate::core::framework::render::{ShaderQualityTier, ShaderVariantKey};
use crate::graphics::pipeline::{
    PipelineAdmission, PipelineAdmissionReason, PipelineAsyncCompileError, PipelineAsyncQueueResult,
};
use crate::graphics::scene::resources::{
    PipelineKey, ResourceStreamer, default_pipeline_key, fallback_shader_uri,
};
use crate::graphics::types::GraphicsError;

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::create_mesh_pipeline;
use super::mesh_pipeline_cache::{PipelineAdmissionKey, PipelineFailure, PipelineUnavailableState};
use super::shader_source::mesh_pipeline_shader_source_for_geometry_descriptor_with_features;
use super::shader_source_validation_admission::CachedMeshShaderModule;
use super::{AsyncBasePipelineProduct, MeshPipelineCache, PipelineCreationTarget};

const BASE_PIPELINE_TARGET: PipelineCreationTarget =
    PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EnvironmentOnlyPbrBasePipelineWarmupMode {
    Synchronous,
    Background,
}

impl EnvironmentOnlyPbrBasePipelineWarmupMode {
    const fn allows_deferred_draw(self) -> bool {
        matches!(self, Self::Background)
    }

    const fn waits_for_pipeline(self) -> bool {
        matches!(self, Self::Synchronous)
    }

    const fn allows_synchronous_fallback(self) -> bool {
        matches!(self, Self::Synchronous)
    }
}

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

    pub(crate) fn ensure_pipeline_admission_for_variant(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
    ) -> PipelineAdmission<()> {
        let allow_synchronous_fallback =
            !self.background_base_pipeline_variants.contains(&variant_id);
        self.ensure_pipeline_admission_for_variant_with_async_defer(
            device,
            streamer,
            variant_id,
            true,
            allow_synchronous_fallback,
        )
    }

    pub(super) fn ensure_synchronous_base_pipeline_admission_for_variant(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
    ) -> PipelineAdmission<()> {
        self.background_base_pipeline_variants.remove(&variant_id);
        let admission_key = PipelineAdmissionKey::new(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            variant_id,
        );
        self.pipeline_failures.remove(&admission_key);
        self.pipeline_unavailable_states.remove(&admission_key);
        self.ensure_pipeline_admission_for_variant_with_async_defer(
            device, streamer, variant_id, false, true,
        )
    }

    fn ensure_pipeline_admission_for_variant_with_async_defer(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
        allow_async_defer: bool,
        allow_synchronous_fallback: bool,
    ) -> PipelineAdmission<()> {
        let requested_async_defer = allow_async_defer;
        let allow_async_defer = self.allow_async_base_pipeline_defer(allow_async_defer);
        self.drain_ready_base_pipelines();
        if self.base_pipeline_is_ready(variant_id) {
            return PipelineAdmission::Ready(());
        }
        if let Some(reason) = self
            .pipeline_failures
            .get(&PipelineAdmissionKey::new(
                PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
                variant_id,
            ))
            .map(|failure| failure.reason)
        {
            return self.unavailable_pipeline(variant_id, reason);
        }
        let Some((kind, pipeline_key, shader_variant_key)) =
            self.pipeline_and_shader_key_for_variant(variant_id)
        else {
            return self.unavailable_pipeline(variant_id, PipelineAdmissionReason::UnknownVariant);
        };
        if kind != MeshPassPipelineKind::Base {
            return self.unavailable_pipeline(variant_id, PipelineAdmissionReason::WrongPass);
        }
        let base_pipeline_layout = self.base_pipeline_layout_for_variant(variant_id).clone();
        if allow_async_defer && self.allow_async_pipeline_compile && !allow_synchronous_fallback {
            let unavailable_reason =
                self.async_base_pipeline_compiler
                    .as_ref()
                    .and_then(|compiler| {
                        if compiler.is_pending(&variant_id) {
                            Some(PipelineAdmissionReason::CompilePending)
                        } else if !compiler.has_available_slot() {
                            Some(PipelineAdmissionReason::QueueSaturated)
                        } else {
                            None
                        }
                    });
            if let Some(reason) = unavailable_reason {
                return self.unavailable_pipeline(variant_id, reason);
            }
        }
        if requested_async_defer && !allow_async_defer && !allow_synchronous_fallback {
            let message = "asynchronous Base pipeline admission is disabled for this variant";
            self.record_shader_variant_pipeline_creation_message(&shader_variant_key, message);
            self.mark_pipeline_failure(
                variant_id,
                PipelineAdmissionReason::CompilationDisabled,
                message,
            );
            return self
                .unavailable_pipeline(variant_id, PipelineAdmissionReason::CompilationDisabled);
        }
        if self.async_base_pipeline_is_pending(variant_id) {
            if allow_async_defer {
                return self
                    .unavailable_pipeline(variant_id, PipelineAdmissionReason::CompilePending);
            }
            if !allow_synchronous_fallback {
                return self
                    .unavailable_pipeline(variant_id, PipelineAdmissionReason::CompilePending);
            }
            self.finish_pending_base_pipeline_variant(variant_id);
            if self.base_pipeline_is_ready(variant_id) {
                return PipelineAdmission::Ready(());
            }
        }
        if allow_async_defer && !self.allow_async_pipeline_compile && !allow_synchronous_fallback {
            let message = "async Base pipeline compilation is disabled for this variant";
            self.record_shader_variant_pipeline_creation_message(&shader_variant_key, message);
            self.mark_pipeline_failure(
                variant_id,
                PipelineAdmissionReason::CompilationDisabled,
                message,
            );
            return self
                .unavailable_pipeline(variant_id, PipelineAdmissionReason::CompilationDisabled);
        }
        if allow_async_defer
            && self.allow_async_pipeline_compile
            && self.async_base_pipeline_compiler.is_none()
            && !allow_synchronous_fallback
        {
            let message = "async Base pipeline compiler is unavailable; rejecting the background draw admission";
            self.record_shader_variant_pipeline_creation_message(&shader_variant_key, message);
            self.mark_pipeline_failure(
                variant_id,
                PipelineAdmissionReason::WorkerUnavailable,
                message,
            );
            return self
                .unavailable_pipeline(variant_id, PipelineAdmissionReason::WorkerUnavailable);
        }
        let Some(geometry_source) =
            self.geometry_source_descriptor_for_variant(&shader_variant_key)
        else {
            self.mark_pipeline_failure(
                variant_id,
                PipelineAdmissionReason::GeometrySourceUnavailable,
                "Base pipeline geometry source descriptor is unavailable",
            );
            return self.unavailable_pipeline(
                variant_id,
                PipelineAdmissionReason::GeometrySourceUnavailable,
            );
        };
        let shader_source = match mesh_pipeline_shader_source_for_geometry_descriptor_with_features(
            streamer,
            &pipeline_key,
            &geometry_source,
            shader_variant_key.features,
        ) {
            Ok(source) => source,
            Err(error) => {
                let message = format!("{error:?}");
                self.record_shader_variant_assembly_error(&shader_variant_key, error);
                self.mark_pipeline_failure(
                    variant_id,
                    PipelineAdmissionReason::SourceAssemblyFailed,
                    message,
                );
                return self.unavailable_pipeline(
                    variant_id,
                    PipelineAdmissionReason::SourceAssemblyFailed,
                );
            }
        };
        self.record_observed_shader_source(BASE_PIPELINE_TARGET, &shader_source.source_hash);
        let shader_key = mesh_shader_module_cache_key(
            &pipeline_key,
            &shader_variant_key,
            &shader_source.source_hash,
        );
        let cached_shader = self.shader_modules.get(&shader_key).cloned();
        let compiled_source = if cached_shader.is_none() {
            let admission = self.mesh_pipeline_shader_source_with_cache(
                shader_source,
                &shader_variant_key,
                PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
                variant_id,
                &pipeline_key,
            );
            match admission {
                PipelineAdmission::Ready(source) => Some(source),
                PipelineAdmission::Deferred(unavailable) => {
                    return PipelineAdmission::Deferred(unavailable);
                }
                PipelineAdmission::Failed(unavailable) => {
                    return PipelineAdmission::Failed(unavailable);
                }
            }
        } else {
            None
        };
        if cached_shader.is_some() {
            match self.cached_shader_module_entry_admission(
                &shader_key,
                &shader_variant_key,
                PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
                variant_id,
                &pipeline_key,
            ) {
                PipelineAdmission::Ready(()) => {}
                PipelineAdmission::Deferred(unavailable) => {
                    return PipelineAdmission::Deferred(unavailable);
                }
                PipelineAdmission::Failed(unavailable) => {
                    return PipelineAdmission::Failed(unavailable);
                }
            }
        }
        if allow_async_defer
            && self.allow_async_pipeline_compile
            && self.async_base_pipeline_compiler.is_some()
        {
            let queue_async_pipeline = {
                let device = device.clone();
                let layout = base_pipeline_layout.clone();
                let target_format = self.target_format;
                let async_pipeline_key = pipeline_key.clone();
                let async_shader_key = shader_key.clone();
                let async_cached_shader = cached_shader.clone();
                let async_compiled_source = compiled_source.clone();
                let async_runtime_pipeline_cache = self.runtime_pipeline_cache.cache().cloned();
                let pipeline_creation_metrics = self.pipeline_creation_metrics.clone();
                move || {
                    let device = device.clone();
                    let layout = layout.clone();
                    let async_pipeline_key = async_pipeline_key.clone();
                    let async_shader_key = async_shader_key.clone();
                    let async_cached_shader = async_cached_shader.clone();
                    let async_compiled_source = async_compiled_source.clone();
                    let async_runtime_pipeline_cache = async_runtime_pipeline_cache.clone();
                    let pipeline_creation_metrics = pipeline_creation_metrics.clone();
                    let queued_at = std::time::Instant::now();
                    move || {
                        pipeline_creation_metrics
                            .record_async_base_pipeline_queue_wait(queued_at.elapsed());
                        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
                        let shader_module = match async_cached_shader {
                            Some(shader_module) => (shader_module, None),
                            None => {
                                let validated_source =
                                    async_compiled_source.expect("uncached async shader source");
                                let validation_key = validated_source.validation_key;
                                let creation_started = std::time::Instant::now();
                                let module =
                                    device.create_shader_module(wgpu::ShaderModuleDescriptor {
                                        label: Some("zircon-mesh-shader"),
                                        source: wgpu::ShaderSource::Wgsl(
                                            validated_source.wgsl_source.into(),
                                        ),
                                    });
                                pipeline_creation_metrics.record_shader_module_creation(
                                    BASE_PIPELINE_TARGET,
                                    creation_started.elapsed(),
                                );
                                (
                                    CachedMeshShaderModule::new(
                                        module,
                                        validated_source.reflection,
                                    ),
                                    Some(validation_key),
                                )
                            }
                        };
                        let render_pipeline_creation_started = std::time::Instant::now();
                        let pipeline = create_mesh_pipeline(
                            &device,
                            &layout,
                            &shader_module.0,
                            target_format,
                            &async_pipeline_key,
                            async_runtime_pipeline_cache.as_ref(),
                        );
                        pipeline_creation_metrics.record_render_pipeline_creation(
                            BASE_PIPELINE_TARGET,
                            render_pipeline_creation_started.elapsed(),
                        );
                        if let Some(error) = pollster::block_on(error_scope.pop()) {
                            return Err(error.to_string());
                        }
                        Ok(AsyncBasePipelineProduct {
                            shader_key: async_shader_key,
                            shader_module: shader_module.0,
                            validation_key: shader_module.1,
                            pipeline,
                        })
                    }
                }
            };
            let mut queue_result = self
                .async_base_pipeline_compiler
                .as_mut()
                .expect("async pipeline compiler checked above")
                .try_queue(variant_id, queue_async_pipeline());
            if matches!(queue_result, PipelineAsyncQueueResult::Full) && !allow_synchronous_fallback
            {
                // A completion can arrive after this frame's initial drain and reclaim a slot.
                self.drain_ready_base_pipelines();
                if self.base_pipeline_is_ready(variant_id) {
                    return PipelineAdmission::Ready(());
                }
                queue_result = self
                    .async_base_pipeline_compiler
                    .as_mut()
                    .expect("async pipeline compiler remains available for the full-queue retry")
                    .try_queue(variant_id, queue_async_pipeline());
            }
            match queue_result {
                PipelineAsyncQueueResult::Queued => {
                    self.async_variant_first_frame_miss_count =
                        self.async_variant_first_frame_miss_count.saturating_add(1);
                    return self
                        .unavailable_pipeline(variant_id, PipelineAdmissionReason::CompileQueued);
                }
                PipelineAsyncQueueResult::AlreadyPending => {
                    return self
                        .unavailable_pipeline(variant_id, PipelineAdmissionReason::CompilePending);
                }
                PipelineAsyncQueueResult::Full => {
                    return self
                        .unavailable_pipeline(variant_id, PipelineAdmissionReason::QueueSaturated);
                }
                PipelineAsyncQueueResult::WorkerUnavailable if !allow_synchronous_fallback => {
                    let message = "async Base pipeline compiler is unavailable; rejecting the background draw admission";
                    self.record_shader_variant_pipeline_creation_message(
                        &shader_variant_key,
                        message,
                    );
                    self.mark_pipeline_failure(
                        variant_id,
                        PipelineAdmissionReason::WorkerUnavailable,
                        message,
                    );
                    return self.unavailable_pipeline(
                        variant_id,
                        PipelineAdmissionReason::WorkerUnavailable,
                    );
                }
                PipelineAsyncQueueResult::WorkerUnavailable => {}
            }
        }
        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        if !self.shader_modules.contains_key(&shader_key) {
            let validated_source = compiled_source.expect("uncached synchronous shader source");
            let validation_key = validated_source.validation_key;
            let creation_started = std::time::Instant::now();
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(validated_source.wgsl_source.into()),
            });
            let creation_elapsed = creation_started.elapsed();
            self.shader_modules.insert(
                shader_key.clone(),
                CachedMeshShaderModule::new(module, validated_source.reflection),
            );
            self.take_ready_shader_source_validation(&validation_key)
                .expect("installed shader module must consume its Ready validation artifact");
            self.record_shader_module_creation(BASE_PIPELINE_TARGET, creation_elapsed);
        }
        if !self.mesh_variant_pipelines.contains_key(&variant_id) {
            let shader = self
                .shader_modules
                .get(&shader_key)
                .expect("shader module cached");
            let creation_started = std::time::Instant::now();
            let pipeline = create_mesh_pipeline(
                device,
                &base_pipeline_layout,
                shader,
                self.target_format,
                &pipeline_key,
                self.runtime_pipeline_cache.cache(),
            );
            let creation_elapsed = creation_started.elapsed();
            self.mesh_variant_pipelines.insert(variant_id, pipeline);
            self.bind_pipeline_shader_module_reference(
                PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
                variant_id,
                &shader_key,
            );
            self.record_render_pipeline_creation(BASE_PIPELINE_TARGET, creation_elapsed);
        }
        let pipeline_validation_failed = self.track_pipeline_creation_error_scope(
            &shader_variant_key,
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            variant_id,
            shader_key,
            error_scope,
        );
        if pipeline_validation_failed {
            self.drain_pipeline_creation_diagnostics();
            self.mark_pipeline_failure(
                variant_id,
                PipelineAdmissionReason::PipelineValidationFailed,
                "Base pipeline WGPU validation failed",
            );
            return self.unavailable_pipeline(
                variant_id,
                PipelineAdmissionReason::PipelineValidationFailed,
            );
        }
        self.pipeline_unavailable_states
            .remove(&PipelineAdmissionKey::new(
                PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
                variant_id,
            ));
        PipelineAdmission::Ready(())
    }

    pub(crate) fn set_async_pipeline_compile_enabled(&mut self, enabled: bool) {
        if self.allow_async_pipeline_compile && !enabled {
            self.finish_pending_base_pipelines();
        }
        if !enabled {
            self.background_base_pipeline_variants.clear();
            self.pipeline_failures.retain(|key, _| {
                key.target != PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base)
            });
            self.pipeline_unavailable_states.retain(|key, _| {
                key.target != PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base)
            });
        }
        self.allow_async_pipeline_compile = enabled;
    }

    pub(crate) const fn async_pipeline_compile_enabled(&self) -> bool {
        self.allow_async_pipeline_compile
    }

    fn allow_async_base_pipeline_defer(&self, requested: bool) -> bool {
        requested && !self.force_synchronous_base_pipeline_compile
    }

    fn base_pipeline_is_ready(&mut self, variant_id: MeshPipelineVariantId) -> bool {
        let ready = self.mesh_variant_pipelines.contains_key(&variant_id);
        if ready {
            self.pipeline_unavailable_states
                .remove(&PipelineAdmissionKey::new(
                    PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
                    variant_id,
                ));
        }
        ready
    }

    pub(super) fn unavailable_pipeline(
        &mut self,
        variant_id: MeshPipelineVariantId,
        reason: PipelineAdmissionReason,
    ) -> PipelineAdmission<()> {
        self.unavailable_pipeline_for_target(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            variant_id,
            reason,
        )
    }

    pub(super) fn unavailable_pipeline_for_target<T>(
        &mut self,
        target: PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
        reason: PipelineAdmissionReason,
    ) -> PipelineAdmission<T> {
        let now = std::time::Instant::now();
        let admission_key = PipelineAdmissionKey::new(target, variant_id);
        let state = self
            .pipeline_unavailable_states
            .entry(admission_key)
            .or_insert(PipelineUnavailableState { reason, since: now });
        if state.reason != reason {
            state.reason = reason;
            state.since = now;
        }
        PipelineAdmission::unavailable(reason, state.since.elapsed())
    }

    pub(super) fn pipeline_failure_reason_for_target(
        &self,
        target: PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
    ) -> Option<PipelineAdmissionReason> {
        self.pipeline_failures
            .get(&PipelineAdmissionKey::new(target, variant_id))
            .map(|failure| failure.reason)
    }

    pub(super) fn clear_pipeline_unavailable_state_for_target(
        &mut self,
        target: PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
    ) {
        self.pipeline_unavailable_states
            .remove(&PipelineAdmissionKey::new(target, variant_id));
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

    pub(in crate::graphics::scene::scene_renderer) fn environment_only_pbr_base_pipeline_ready(
        &mut self,
    ) -> Result<bool, GraphicsError> {
        self.drain_ready_base_pipelines();
        if let Some(error) = self
            .background_base_pipeline_variants
            .iter()
            .find_map(|variant_id| {
                self.pipeline_failures.get(&PipelineAdmissionKey::new(
                    PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
                    *variant_id,
                ))
            })
        {
            return Err(GraphicsError::Asset(format!(
                "environment-only PBR Base pipeline background compilation failed: {}",
                error.message
            )));
        }
        Ok(self.background_base_pipeline_variants.is_empty())
    }

    /// Reports whether the explicitly queued non-default-IOR generic Forward
    /// Base PSO can draw. It never treats the environment-only fast-path PSO
    /// as proof for this variant.
    pub(in crate::graphics::scene::scene_renderer) fn pbr_ior_forward_base_pipeline_ready(
        &mut self,
    ) -> Result<bool, GraphicsError> {
        self.drain_ready_base_pipelines();
        let Some(variant_id) = self.pbr_ior_forward_base_pipeline_variant else {
            return Ok(false);
        };
        if let Some(error) = self.pipeline_failures.get(&PipelineAdmissionKey::new(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            variant_id,
        )) {
            return Err(GraphicsError::Asset(format!(
                "PBR IOR Forward Base pipeline background compilation failed: {}",
                error.message
            )));
        }
        Ok(self.mesh_variant_pipelines.contains_key(&variant_id))
    }

    pub(super) fn mark_pipeline_failure(
        &mut self,
        variant_id: MeshPipelineVariantId,
        reason: PipelineAdmissionReason,
        message: impl Into<String>,
    ) {
        self.mark_pipeline_failure_for_target(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            variant_id,
            reason,
            message,
        );
    }

    pub(super) fn mark_pipeline_failure_for_target(
        &mut self,
        target: PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
        reason: PipelineAdmissionReason,
        message: impl Into<String>,
    ) {
        self.pipeline_failures
            .entry(PipelineAdmissionKey::new(target, variant_id))
            .or_insert_with(|| PipelineFailure {
                reason,
                message: message.into(),
            });
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
                            error.clone(),
                        );
                    }
                    self.mark_pipeline_failure(
                        variant_id,
                        PipelineAdmissionReason::PipelineValidationFailed,
                        error,
                    );
                    continue;
                }
                Err(error) => {
                    if let Some((_, _, shader_variant_key)) =
                        self.pipeline_and_shader_key_for_variant(variant_id)
                    {
                        self.record_shader_variant_pipeline_creation_error(
                            &shader_variant_key,
                            &error,
                        );
                    }
                    let reason = match error {
                        PipelineAsyncCompileError::JobPanicked => {
                            PipelineAdmissionReason::JobPanicked
                        }
                        PipelineAsyncCompileError::WorkerUnavailable => {
                            PipelineAdmissionReason::WorkerUnavailable
                        }
                    };
                    self.mark_pipeline_failure(variant_id, reason, format!("{error:?}"));
                    continue;
                }
            };
            let AsyncBasePipelineProduct {
                shader_key,
                shader_module,
                validation_key,
                pipeline,
            } = product;
            self.shader_modules
                .entry(shader_key.clone())
                .or_insert(shader_module);
            if let Some(validation_key) = validation_key {
                self.take_ready_shader_source_validation(&validation_key)
                    .expect("installed async shader module must consume its validation artifact");
            }
            self.mesh_variant_pipelines.insert(variant_id, pipeline);
            self.bind_pipeline_shader_module_reference(
                PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
                variant_id,
                &shader_key,
            );
            self.background_base_pipeline_variants.remove(&variant_id);
            let admission_key = PipelineAdmissionKey::new(
                PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
                variant_id,
            );
            self.pipeline_failures.remove(&admission_key);
            self.pipeline_unavailable_states.remove(&admission_key);
        }
    }

    /// Queues the exact static Standard-PBR Base variant submitted by the
    /// environment-only viewer's `BaseScenePass` without waiting for PSO creation.
    ///
    /// Until the worker completes, Base-pass draw admission remains explicitly
    /// deferred while the host continues presenting frames.
    pub(crate) fn queue_environment_only_pbr_base_pipeline(
        &mut self,
        device: &wgpu::Device,
        streamer: &mut ResourceStreamer,
    ) -> Result<EnvironmentOnlyPbrBasePipelinePrewarmReport, GraphicsError> {
        self.warm_environment_only_pbr_base_pipeline(
            device,
            streamer,
            EnvironmentOnlyPbrBasePipelineWarmupMode::Background,
            false,
        )
    }

    /// Queues the static non-default-IOR material's generic Forward Base
    /// variant. This is a diagnostic readiness gate; it keeps IOR routing out
    /// of the environment-only shader feature and PSO identity.
    pub(crate) fn queue_pbr_ior_forward_base_pipeline(
        &mut self,
        device: &wgpu::Device,
        streamer: &mut ResourceStreamer,
    ) -> Result<(), GraphicsError> {
        self.warm_environment_only_pbr_base_pipeline(
            device,
            streamer,
            EnvironmentOnlyPbrBasePipelineWarmupMode::Background,
            true,
        )?;
        Ok(())
    }

    /// Creates the exact static Standard-PBR Base variant submitted by the
    /// environment-only viewer's `BaseScenePass` synchronously.
    ///
    /// This only fills the renderer-owned cache; it does not encode, submit,
    /// present, or read back a frame.
    pub(crate) fn prewarm_environment_only_pbr_base_pipeline(
        &mut self,
        device: &wgpu::Device,
        streamer: &mut ResourceStreamer,
    ) -> Result<EnvironmentOnlyPbrBasePipelinePrewarmReport, GraphicsError> {
        self.warm_environment_only_pbr_base_pipeline(
            device,
            streamer,
            EnvironmentOnlyPbrBasePipelineWarmupMode::Synchronous,
            false,
        )
    }

    fn warm_environment_only_pbr_base_pipeline(
        &mut self,
        device: &wgpu::Device,
        streamer: &mut ResourceStreamer,
        mode: EnvironmentOnlyPbrBasePipelineWarmupMode,
        pbr_ior_override: bool,
    ) -> Result<EnvironmentOnlyPbrBasePipelinePrewarmReport, GraphicsError> {
        let started = std::time::Instant::now();
        let mut pipeline_key = default_pipeline_key();
        let shader_source_started = std::time::Instant::now();
        let (shader_id, shader_revision, shader_dependency_revision, _) =
            streamer.ensure_shader_source(&AssetReference::from_locator(fallback_shader_uri()))?;
        let shader_source_resolution = shader_source_started.elapsed();
        if shader_id != pipeline_key.shader_id {
            return Err(GraphicsError::Asset(format!(
                "environment-only PBR prewarm resolved {shader_id}, expected {}",
                pipeline_key.shader_id
            )));
        }
        pipeline_key.shader_revision = shader_revision;
        pipeline_key.shader_dependency_revision = shader_dependency_revision;
        // The viewer's static fixtures do not receive shadows. Keep the generic
        // Forward IOR warmup keyed to that submitted material, not to the
        // default receiver variant.
        pipeline_key.receive_shadows = false;
        if pbr_ior_override {
            pipeline_key.pbr_ior_override = true;
        }
        if !pbr_ior_override {
            self.pipeline_variant_registry
                .enable_environment_only_pbr_base_profile();
        }
        let variant_id = self.resolve_variant(
            MeshPassPipelineKind::Base,
            &pipeline_key,
            ShaderQualityTier::default(),
        );
        if pbr_ior_override {
            self.pbr_ior_forward_base_pipeline_variant = Some(variant_id);
        }
        if mode.allows_synchronous_fallback() {
            self.background_base_pipeline_variants.remove(&variant_id);
            self.pipeline_failures.remove(&PipelineAdmissionKey::new(
                PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
                variant_id,
            ));
        } else {
            self.background_base_pipeline_variants.insert(variant_id);
            self.pipeline_failures.remove(&PipelineAdmissionKey::new(
                PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
                variant_id,
            ));
        }
        let shader_variant_key = self
            .pipeline_and_shader_key_for_variant(variant_id)
            .expect("resolved environment-only PBR variant must have a shader key")
            .2;
        let pipeline_creation_started = std::time::Instant::now();
        let cache_hit = if self.mesh_variant_pipelines.contains_key(&variant_id) {
            true
        } else if mode.waits_for_pipeline() && self.async_base_pipeline_is_pending(variant_id) {
            self.finish_pending_base_pipeline_variant(variant_id)
        } else {
            false
        };
        let mut pipeline_admission = self.ensure_pipeline_admission_for_variant_with_async_defer(
            device,
            streamer,
            variant_id,
            mode.allows_deferred_draw(),
            mode.allows_synchronous_fallback(),
        );
        if mode.waits_for_pipeline() {
            for _ in 0..2 {
                let Some(unavailable) = pipeline_admission.unavailable_details() else {
                    break;
                };
                if !matches!(
                    unavailable.reason(),
                    PipelineAdmissionReason::SourceValidationQueued
                        | PipelineAdmissionReason::SourceValidationPending
                        | PipelineAdmissionReason::QueueSaturated
                ) {
                    break;
                }
                self.finish_pending_shader_source_validations();
                pipeline_admission = self.ensure_pipeline_admission_for_variant_with_async_defer(
                    device,
                    streamer,
                    variant_id,
                    mode.allows_deferred_draw(),
                    mode.allows_synchronous_fallback(),
                );
            }
        }
        let pipeline_ready = pipeline_admission.is_ready();
        if pipeline_ready {
            self.background_base_pipeline_variants.remove(&variant_id);
        }
        if mode.waits_for_pipeline() && !pipeline_ready {
            return Err(GraphicsError::Asset(
                "environment-only PBR prewarm could not create its Base pipeline".to_string(),
            ));
        }
        if mode.waits_for_pipeline() {
            let collected_pipeline_diagnostic = self
                .finish_pipeline_creation_diagnostics_for_variant(&shader_variant_key)
                .map_err(|error| {
                    GraphicsError::Asset(format!(
                        "environment-only PBR prewarm Base pipeline validation failed: {error}"
                    ))
                })?;
            if !cache_hit && !collected_pipeline_diagnostic {
                return Err(GraphicsError::Asset(
                    "environment-only PBR prewarm did not retain its pipeline validation diagnostic"
                        .to_string(),
                ));
            }
        }
        let pipeline_creation = pipeline_creation_started.elapsed();
        Ok(EnvironmentOnlyPbrBasePipelinePrewarmReport {
            pipeline_ready,
            cache_hit,
            shader_source_resolution,
            pipeline_creation,
            elapsed: started.elapsed(),
        })
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
