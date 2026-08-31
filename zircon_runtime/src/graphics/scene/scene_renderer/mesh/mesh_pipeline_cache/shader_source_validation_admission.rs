use std::sync::Arc;

use crate::core::framework::render::{
    SHADER_VARIANT_CACHE_NAGA_VERSION, SHADER_VARIANT_CACHE_WGPU_VERSION, ShaderVariantKey,
};
use crate::graphics::pipeline::{
    PipelineAdmission, PipelineAdmissionReason, PipelineAsyncCompileError, PipelineAsyncQueueResult,
};
use crate::graphics::scene::resources::PipelineKey;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshPassPipelineKind, MeshPipelineVariantId,
};
use crate::graphics::shader::{
    ShaderBindingStage, ShaderVariantCacheDiskKey, ShaderVariantCacheDiskLookup,
    template::ShaderTemplateReflection,
};

use super::mesh_pipeline_cache::{MeshPipelineCache, PipelineCreationTarget};
use super::mesh_shader_entry_contract::{
    MeshShaderEntryContract, MeshShaderProgramKind, ShaderEntryStage,
};
use super::mesh_shader_fragment_contract::MeshShaderFragmentOutputContract;
use super::mesh_shader_resource_contract::{
    MeshShaderPipelineLayoutContract, MeshShaderResourceRequirement,
    MeshShaderSamplingPairRequirement,
};
use super::mesh_shader_vertex_contract::MeshShaderVertexLayoutContract;
use super::shader_source::{MeshPipelineShaderSource, ValidatedMeshPipelineShaderSource};
use super::shader_source_validation_states::{
    ShaderSourceValidationStates, ShaderSourceValidationStatus,
};

#[derive(Clone)]
pub(in crate::graphics::scene::scene_renderer::mesh) struct CachedMeshShaderModule {
    module: wgpu::ShaderModule,
    reflection: Arc<ShaderTemplateReflection>,
}

impl CachedMeshShaderModule {
    pub(super) fn new(
        module: wgpu::ShaderModule,
        reflection: Arc<ShaderTemplateReflection>,
    ) -> Self {
        Self { module, reflection }
    }

    fn reflection(&self) -> &ShaderTemplateReflection {
        self.reflection.as_ref()
    }

    fn validate_shader_contract(
        &self,
        entry_contract: MeshShaderEntryContract,
        vertex_contract: &MeshShaderVertexLayoutContract,
        fragment_contract: &MeshShaderFragmentOutputContract,
        resource_contract: &MeshShaderPipelineLayoutContract,
    ) -> Result<(), String> {
        validate_reflection_shader_contract(
            self.reflection(),
            entry_contract,
            vertex_contract,
            fragment_contract,
            resource_contract,
        )
    }

    #[cfg(test)]
    pub(super) fn from_test_module(module: wgpu::ShaderModule) -> Self {
        Self::new(
            module,
            Arc::new(ShaderTemplateReflection {
                entry_points: Vec::new(),
                resource_bindings: Vec::new(),
                pipeline_override_count: 0,
                interface_requires_specialization: false,
                resource_layout_requires_specialization: false,
                interface_layout_hash: [0; 32],
                resource_layout_hash: [0; 32],
            }),
        )
    }
}

impl std::ops::Deref for CachedMeshShaderModule {
    type Target = wgpu::ShaderModule;

    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ShaderSourceValidationKey {
    shader_variant_key: ShaderVariantKey,
    source_identity: String,
}

impl ShaderSourceValidationKey {
    pub(super) fn new(shader_variant_key: &ShaderVariantKey, source_identity: String) -> Self {
        Self {
            shader_variant_key: shader_variant_key.clone(),
            source_identity,
        }
    }
}

#[derive(Clone)]
pub(super) struct ShaderSourceValidationFailure {
    pub(super) reason: PipelineAdmissionReason,
    pub(super) message: Arc<str>,
}

pub(super) type MeshShaderSourceValidationStates = ShaderSourceValidationStates<
    ShaderSourceValidationKey,
    Arc<ShaderTemplateReflection>,
    ShaderSourceValidationFailure,
>;

impl MeshPipelineCache {
    pub(super) fn mesh_pipeline_shader_source_with_cache(
        &mut self,
        source: MeshPipelineShaderSource,
        variant_key: &ShaderVariantKey,
        target: PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
        pipeline_key: &PipelineKey,
    ) -> PipelineAdmission<ValidatedMeshPipelineShaderSource> {
        let validation_source_identity = source.validation_cache_key();
        let validation_key =
            ShaderSourceValidationKey::new(variant_key, validation_source_identity);
        let MeshPipelineShaderSource {
            wgsl_source,
            source_hash,
            cache_content_hashes,
            template_revision,
            segments,
            ..
        } = source;
        let reflection = match self.shader_source_validation_status(&validation_key) {
            ShaderSourceValidationStatus::Missing => {
                let reason = match self.queue_shader_source_validation(
                    validation_key,
                    wgsl_source.clone(),
                    segments,
                ) {
                    PipelineAsyncQueueResult::Queued => {
                        PipelineAdmissionReason::SourceValidationQueued
                    }
                    PipelineAsyncQueueResult::AlreadyPending => {
                        PipelineAdmissionReason::SourceValidationPending
                    }
                    PipelineAsyncQueueResult::Full => PipelineAdmissionReason::QueueSaturated,
                    PipelineAsyncQueueResult::WorkerUnavailable => {
                        self.mark_pipeline_failure_for_target(
                            target,
                            variant_id,
                            PipelineAdmissionReason::WorkerUnavailable,
                            "shader source validation worker is unavailable",
                        );
                        PipelineAdmissionReason::WorkerUnavailable
                    }
                };
                return self.unavailable_pipeline_for_target(target, variant_id, reason);
            }
            ShaderSourceValidationStatus::Pending => {
                return self.unavailable_pipeline_for_target(
                    target,
                    variant_id,
                    PipelineAdmissionReason::SourceValidationPending,
                );
            }
            ShaderSourceValidationStatus::Ready(reflection) => {
                let validation = validate_reflection_shader_contract(
                    reflection.as_ref(),
                    mesh_shader_entry_contract(target, pipeline_key),
                    self.shader_vertex_contract_for_target(target),
                    self.shader_fragment_contract_for_target(target),
                    self.shader_resource_contract_for_target(target, variant_id),
                );
                if let Err(message) = validation {
                    self.record_shader_variant_validation_error(variant_key, message.clone());
                    self.mark_pipeline_failure_for_target(
                        target,
                        variant_id,
                        PipelineAdmissionReason::ShaderInterfaceMismatch,
                        message,
                    );
                    return self.unavailable_pipeline_for_target(
                        target,
                        variant_id,
                        PipelineAdmissionReason::ShaderInterfaceMismatch,
                    );
                }
                reflection
            }
            ShaderSourceValidationStatus::Failed(failure) => {
                self.mark_pipeline_failure_for_target(
                    target,
                    variant_id,
                    failure.reason,
                    failure.message.as_ref(),
                );
                return self.unavailable_pipeline_for_target(target, variant_id, failure.reason);
            }
        };
        let disk_key = ShaderVariantCacheDiskKey::from_variant_key(
            variant_key,
            source_hash,
            &cache_content_hashes,
            &template_revision,
            SHADER_VARIANT_CACHE_NAGA_VERSION,
            SHADER_VARIANT_CACHE_WGPU_VERSION,
        );
        let compiled_source = match self.shader_variant_disk_cache.lookup(&disk_key) {
            ShaderVariantCacheDiskLookup::Hit(entry) => {
                self.record_shader_variant_disk_hit(variant_key);
                entry.wgsl_source
            }
            ShaderVariantCacheDiskLookup::Miss => {
                self.record_shader_variant_compile_miss(variant_key);
                match self
                    .shader_variant_disk_cache
                    .write(&disk_key, &wgsl_source)
                {
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
        PipelineAdmission::Ready(ValidatedMeshPipelineShaderSource {
            wgsl_source: compiled_source,
            reflection,
            validation_key,
        })
    }

    pub(super) fn cached_shader_module_entry_admission(
        &mut self,
        shader_key: &str,
        variant_key: &ShaderVariantKey,
        target: PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
        pipeline_key: &PipelineKey,
    ) -> PipelineAdmission<()> {
        let contract = mesh_shader_entry_contract(target, pipeline_key);
        let vertex_contract = self.shader_vertex_contract_for_target(target);
        let fragment_contract = self.shader_fragment_contract_for_target(target);
        let resource_contract = self.shader_resource_contract_for_target(target, variant_id);
        let result = self
            .shader_modules
            .get(shader_key)
            .expect("cached shader entry admission requires a cached module")
            .validate_shader_contract(
                contract,
                vertex_contract,
                fragment_contract,
                resource_contract,
            );
        if let Err(message) = result {
            self.record_shader_variant_validation_error(variant_key, message.clone());
            self.mark_pipeline_failure_for_target(
                target,
                variant_id,
                PipelineAdmissionReason::ShaderInterfaceMismatch,
                message,
            );
            return self.unavailable_pipeline_for_target(
                target,
                variant_id,
                PipelineAdmissionReason::ShaderInterfaceMismatch,
            );
        }
        PipelineAdmission::Ready(())
    }

    pub(super) fn shader_source_validation_status(
        &mut self,
        key: &ShaderSourceValidationKey,
    ) -> ShaderSourceValidationStatus<Arc<ShaderTemplateReflection>, ShaderSourceValidationFailure>
    {
        self.drain_shader_source_validation_diagnostics();
        self.shader_source_validation_states.status(key)
    }

    fn queue_shader_source_validation(
        &mut self,
        validation_key: ShaderSourceValidationKey,
        wgsl_source: String,
        segments: Vec<crate::graphics::shader::ShaderAssemblySegment>,
    ) -> PipelineAsyncQueueResult {
        let shader_variant_key = validation_key.shader_variant_key.clone();
        let validation_source_identity = validation_key.source_identity.clone();
        let validation_metrics = Arc::clone(&self.pipeline_creation_metrics);
        let queued_at = std::time::Instant::now();
        let outcome = self
            .shader_source_validation_compiler
            .as_mut()
            .map(|compiler| {
                compiler.try_queue(validation_key.clone(), move || {
                    crate::profile_scope!("render", "shader_pipeline", "source_validation_worker");
                    validation_metrics.record_shader_source_validation_started(
                        &validation_source_identity,
                        queued_at.elapsed(),
                    );
                    let validation_started = std::time::Instant::now();
                    let result =
                        super::shader_source::MeshPipelineShaderSource::validate_wgsl_with_segments(
                            &wgsl_source,
                            &segments,
                        );
                    validation_metrics.record_shader_source_validation_completed(
                        validation_started.elapsed(),
                        result.is_ok(),
                    );
                    result
                })
            })
            .unwrap_or(PipelineAsyncQueueResult::WorkerUnavailable);
        self.pipeline_creation_metrics
            .record_shader_source_validation_queue_result(outcome);
        match outcome {
            PipelineAsyncQueueResult::Queued | PipelineAsyncQueueResult::AlreadyPending => {
                self.shader_source_validation_states
                    .mark_pending(validation_key);
            }
            PipelineAsyncQueueResult::Full => self.record_shader_variant_validation_diagnostic(
                &shader_variant_key,
                "background WGSL validation deferred because its bounded queue is full",
            ),
            PipelineAsyncQueueResult::WorkerUnavailable => {
                self.record_shader_variant_validation_diagnostic(
                    &shader_variant_key,
                    "background WGSL validation failed because its worker is unavailable",
                );
            }
        }
        self.record_shader_source_validation_state_counts();
        outcome
    }

    pub(super) fn drain_shader_source_validation_diagnostics(&mut self) {
        let mut completions = Vec::new();
        if let Some(compiler) = self.shader_source_validation_compiler.as_mut() {
            compiler.drain_ready(|key, result| completions.push((key, result)));
        }
        self.publish_shader_source_validation_completions(completions);
    }

    pub(super) fn finish_pending_shader_source_validations(&mut self) -> usize {
        let mut completions = Vec::new();
        let completed = self
            .shader_source_validation_compiler
            .as_mut()
            .map(|compiler| compiler.finish_pending(|key, result| completions.push((key, result))))
            .unwrap_or(0);
        self.publish_shader_source_validation_completions(completions);
        completed
    }

    fn publish_shader_source_validation_completions(
        &mut self,
        completions: Vec<(
            ShaderSourceValidationKey,
            Result<Result<Arc<ShaderTemplateReflection>, String>, PipelineAsyncCompileError>,
        )>,
    ) {
        for (key, result) in completions {
            match result {
                Ok(Ok(reflection)) => {
                    self.shader_source_validation_states
                        .publish_ready(&key, reflection);
                }
                Ok(Err(message)) => self.publish_shader_source_validation_failure(
                    key,
                    PipelineAdmissionReason::SourceValidationFailed,
                    message,
                ),
                Err(error) => {
                    let reason = match error {
                        PipelineAsyncCompileError::JobPanicked => {
                            PipelineAdmissionReason::JobPanicked
                        }
                        PipelineAsyncCompileError::WorkerUnavailable => {
                            PipelineAdmissionReason::WorkerUnavailable
                        }
                    };
                    self.publish_shader_source_validation_failure(
                        key,
                        reason,
                        format!("{error:?}"),
                    );
                }
            }
        }
        self.record_shader_source_validation_state_counts();
    }

    fn publish_shader_source_validation_failure(
        &mut self,
        key: ShaderSourceValidationKey,
        reason: PipelineAdmissionReason,
        message: String,
    ) {
        let message: Arc<str> = Arc::from(message);
        self.shader_source_validation_states.publish_failed(
            &key,
            ShaderSourceValidationFailure {
                reason,
                message: Arc::clone(&message),
            },
        );
        self.record_shader_variant_validation_error(
            &key.shader_variant_key,
            message.as_ref().to_owned(),
        );
    }

    pub(super) fn take_ready_shader_source_validation(
        &mut self,
        key: &ShaderSourceValidationKey,
    ) -> Option<Arc<ShaderTemplateReflection>> {
        let reflection = self.shader_source_validation_states.take_ready(key);
        self.record_shader_source_validation_state_counts();
        reflection
    }

    fn record_shader_source_validation_state_counts(&self) {
        crate::profile_counter!(
            "render",
            "mesh_shader_validation_pending_count",
            self.shader_source_validation_states.pending_count()
        );
        crate::profile_counter!(
            "render",
            "mesh_shader_validation_ready_count",
            self.shader_source_validation_states.ready_count()
        );
        crate::profile_counter!(
            "render",
            "mesh_shader_validation_failed_count",
            self.shader_source_validation_states.failed_count()
        );
        crate::profile_counter!(
            "render",
            "mesh_shader_validation_identity_count",
            self.shader_source_validation_states.len()
        );
    }
}

pub(super) fn mesh_shader_entry_contract(
    target: PipelineCreationTarget,
    pipeline_key: &PipelineKey,
) -> MeshShaderEntryContract {
    let kind = match target {
        PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base) => MeshShaderProgramKind::Base,
        PipelineCreationTarget::MeshPass(MeshPassPipelineKind::GBuffer) => {
            MeshShaderProgramKind::GBuffer
        }
        PipelineCreationTarget::MeshPass(MeshPassPipelineKind::DepthPrepass) => {
            MeshShaderProgramKind::DepthPrepass {
                alpha_masked: pipeline_key.is_alpha_mask(),
            }
        }
        PipelineCreationTarget::MeshPass(MeshPassPipelineKind::ShadowDepth) => {
            MeshShaderProgramKind::ShadowDepth
        }
        PipelineCreationTarget::MeshPass(MeshPassPipelineKind::ShadowDepthAlphaMask) => {
            MeshShaderProgramKind::ShadowDepthAlphaMask
        }
        PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Velocity) => {
            MeshShaderProgramKind::Velocity
        }
        PipelineCreationTarget::MeshPass(MeshPassPipelineKind::TaaReactiveMask) => {
            MeshShaderProgramKind::TaaReactiveMask
        }
        PipelineCreationTarget::MeshPass(MeshPassPipelineKind::TaaReactiveMaterialMask) => {
            MeshShaderProgramKind::TaaReactiveMaterialMask
        }
        PipelineCreationTarget::MeshPass(MeshPassPipelineKind::HitProxy) => {
            MeshShaderProgramKind::HitProxy
        }
        PipelineCreationTarget::Oit => MeshShaderProgramKind::Oit,
    };
    MeshShaderEntryContract::for_program(kind)
}

pub(super) fn validate_reflection_entry_contract(
    reflection: &ShaderTemplateReflection,
    contract: MeshShaderEntryContract,
) -> Result<(), String> {
    contract.validate(|stage, name| {
        reflection.entry_points.iter().any(|entry| {
            entry.name == name
                && matches!(
                    (stage, entry.stage),
                    (ShaderEntryStage::Vertex, naga::ShaderStage::Vertex)
                        | (ShaderEntryStage::Fragment, naga::ShaderStage::Fragment)
                        | (ShaderEntryStage::Compute, naga::ShaderStage::Compute)
                )
        })
    })
}

pub(super) fn validate_reflection_shader_contract(
    reflection: &ShaderTemplateReflection,
    entry_contract: MeshShaderEntryContract,
    vertex_contract: &MeshShaderVertexLayoutContract,
    fragment_contract: &MeshShaderFragmentOutputContract,
    resource_contract: &MeshShaderPipelineLayoutContract,
) -> Result<(), String> {
    validate_reflection_entry_contract(reflection, entry_contract)?;
    vertex_contract.validate(reflection, entry_contract.vertex_entry())?;
    if let Some((vertex_entry, fragment_entry)) = entry_contract.vertex_fragment_entries() {
        reflection.validate_vertex_fragment_stage_interface(vertex_entry, fragment_entry)?;
        fragment_contract.validate(reflection, fragment_entry)?;
    }
    entry_contract.try_for_each_required_entry(|stage, name| {
        let naga_stage = naga_shader_stage(stage);
        let entry = reflection
            .entry_points
            .iter()
            .find(|entry| entry.name == name && entry.stage == naga_stage)
            .expect("entry contract validation must resolve the required entry");
        for identity in &entry.resource_bindings {
            resource_contract
                .validate_requirement(
                    MeshShaderResourceRequirement::new(
                        identity.group,
                        identity.binding,
                        identity.resource_type,
                        shader_binding_stage(stage),
                    )
                    .with_min_binding_size(identity.min_binding_size),
                )
                .map_err(|message| {
                    format!(
                        "shader @{naga_stage:?} entry `{name}` resource ABI mismatch: {message}"
                    )
                })?;
        }
        for pair in &entry.sampling_pairs {
            resource_contract
                .validate_sampling_pair(MeshShaderSamplingPairRequirement::new(
                    pair.texture_group,
                    pair.texture_binding,
                    pair.sampler_group,
                    pair.sampler_binding,
                ))
                .map_err(|message| {
                    format!(
                        "shader @{naga_stage:?} entry `{name}` sampling ABI mismatch: {message}"
                    )
                })?;
        }
        Ok(())
    })
}

impl MeshPipelineCache {
    fn shader_fragment_contract_for_target(
        &self,
        target: PipelineCreationTarget,
    ) -> &MeshShaderFragmentOutputContract {
        self.mesh_shader_fragment_contracts.for_target(target)
    }
}

const fn naga_shader_stage(stage: ShaderEntryStage) -> naga::ShaderStage {
    match stage {
        ShaderEntryStage::Vertex => naga::ShaderStage::Vertex,
        ShaderEntryStage::Fragment => naga::ShaderStage::Fragment,
        ShaderEntryStage::Compute => naga::ShaderStage::Compute,
    }
}

const fn shader_binding_stage(stage: ShaderEntryStage) -> ShaderBindingStage {
    match stage {
        ShaderEntryStage::Vertex => ShaderBindingStage::Vertex,
        ShaderEntryStage::Fragment => ShaderBindingStage::Fragment,
        ShaderEntryStage::Compute => ShaderBindingStage::Compute,
    }
}
