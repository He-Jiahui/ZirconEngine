use std::collections::HashSet;

use crate::core::framework::render::{GeometrySourceId, ShaderQualityTier};
use crate::core::resource::ResourceId;
use crate::graphics::pipeline::{PipelineAdmission, PipelineUnavailable};
use crate::graphics::scene::resources::{PipelineKey, ResourceStreamer};

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::{MeshPipelineCache, PipelineCreationTarget};

const ERROR_PROXY_SOURCE_ADMISSION_ATTEMPTS: usize = 3;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MaterialPipelineRequirement {
    target: PipelineCreationTarget,
    pipeline_key: PipelineKey,
    geometry_source: GeometrySourceId,
    shader_quality: ShaderQualityTier,
}

impl MaterialPipelineRequirement {
    pub(crate) fn new(
        target: PipelineCreationTarget,
        pipeline_key: PipelineKey,
        geometry_source: GeometrySourceId,
        shader_quality: ShaderQualityTier,
    ) -> Self {
        Self {
            target,
            pipeline_key,
            geometry_source,
            shader_quality,
        }
    }

    pub(crate) const fn target(&self) -> PipelineCreationTarget {
        self.target
    }

    pub(crate) const fn pipeline_key(&self) -> &PipelineKey {
        &self.pipeline_key
    }

    fn pipeline_kind(&self) -> MeshPassPipelineKind {
        match self.target {
            PipelineCreationTarget::MeshPass(kind) => kind,
            PipelineCreationTarget::Oit => MeshPassPipelineKind::Base,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ResolvedMaterialPipelineRequirement {
    target: PipelineCreationTarget,
    variant_id: MeshPipelineVariantId,
}

impl ResolvedMaterialPipelineRequirement {
    pub(super) const fn new(
        target: PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
    ) -> Self {
        Self { target, variant_id }
    }

    pub(crate) const fn target(self) -> PipelineCreationTarget {
        self.target
    }

    pub(crate) const fn variant_id(self) -> MeshPipelineVariantId {
        self.variant_id
    }
}

#[derive(Default)]
pub(crate) struct MaterialPipelineRequirementSet {
    ordered: Vec<MaterialPipelineRequirement>,
    unique: HashSet<MaterialPipelineRequirement>,
}

impl MaterialPipelineRequirementSet {
    pub(crate) fn insert(&mut self, requirement: MaterialPipelineRequirement) -> bool {
        if !self.unique.insert(requirement.clone()) {
            return false;
        }
        self.ordered.push(requirement);
        true
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &MaterialPipelineRequirement> {
        self.ordered.iter()
    }

    pub(crate) const fn len(&self) -> usize {
        self.ordered.len()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MaterialPipelinePublicationAdmission {
    Ready {
        requirement_count: usize,
    },
    Deferred {
        requirement_count: usize,
        ready_count: usize,
        requirement: ResolvedMaterialPipelineRequirement,
        unavailable: PipelineUnavailable,
    },
    Failed {
        requirement_count: usize,
        ready_count: usize,
        requirement: ResolvedMaterialPipelineRequirement,
        unavailable: PipelineUnavailable,
    },
}

impl MaterialPipelinePublicationAdmission {
    pub(crate) const fn requirement_count(self) -> usize {
        match self {
            Self::Ready { requirement_count }
            | Self::Deferred {
                requirement_count, ..
            }
            | Self::Failed {
                requirement_count, ..
            } => requirement_count,
        }
    }

    pub(crate) const fn ready_count(self) -> usize {
        match self {
            Self::Ready { requirement_count } => requirement_count,
            Self::Deferred { ready_count, .. } | Self::Failed { ready_count, .. } => ready_count,
        }
    }
}

#[derive(Default)]
struct MaterialPipelineAdmissionAccumulator {
    requirement_count: usize,
    ready_count: usize,
    first_deferred: Option<(ResolvedMaterialPipelineRequirement, PipelineUnavailable)>,
    first_failed: Option<(ResolvedMaterialPipelineRequirement, PipelineUnavailable)>,
}

struct MaterialPipelineAdmissionResult {
    admission: MaterialPipelinePublicationAdmission,
    resolved_pipelines: Vec<ResolvedMaterialPipelineRequirement>,
}

#[derive(Default)]
struct ErrorProxyPipelineAdmissionStats {
    attempt_count: usize,
    source_validation_sync_count: usize,
    source_validation_sync_completed_count: usize,
    source_validation_sync_wait_micros: u64,
    source_validation_queued_count: usize,
    source_validation_pending_count: usize,
    source_validation_queue_saturated_count: usize,
}

impl ErrorProxyPipelineAdmissionStats {
    fn record_source_validation_sync(
        &mut self,
        reason: crate::graphics::pipeline::PipelineAdmissionReason,
        completed_count: usize,
        elapsed: std::time::Duration,
    ) {
        use crate::graphics::pipeline::PipelineAdmissionReason;

        self.source_validation_sync_count = self.source_validation_sync_count.saturating_add(1);
        self.source_validation_sync_completed_count = self
            .source_validation_sync_completed_count
            .saturating_add(completed_count);
        self.source_validation_sync_wait_micros = self
            .source_validation_sync_wait_micros
            .saturating_add(u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX));
        match reason {
            PipelineAdmissionReason::SourceValidationQueued => {
                self.source_validation_queued_count =
                    self.source_validation_queued_count.saturating_add(1);
            }
            PipelineAdmissionReason::SourceValidationPending => {
                self.source_validation_pending_count =
                    self.source_validation_pending_count.saturating_add(1);
            }
            PipelineAdmissionReason::QueueSaturated => {
                self.source_validation_queue_saturated_count = self
                    .source_validation_queue_saturated_count
                    .saturating_add(1);
            }
            _ => {}
        }
    }

    fn profile(&self) {
        crate::profile_counter!(
            "render",
            "error_proxy_pipeline_admission_attempt_count",
            self.attempt_count
        );
        crate::profile_counter!(
            "render",
            "error_proxy_source_validation_sync_count",
            self.source_validation_sync_count
        );
        crate::profile_counter!(
            "render",
            "error_proxy_source_validation_sync_completed_count",
            self.source_validation_sync_completed_count
        );
        crate::profile_counter!(
            "render",
            "error_proxy_source_validation_sync_wait_micros",
            self.source_validation_sync_wait_micros
        );
        crate::profile_counter!(
            "render",
            "error_proxy_source_validation_queued_count",
            self.source_validation_queued_count
        );
        crate::profile_counter!(
            "render",
            "error_proxy_source_validation_pending_count",
            self.source_validation_pending_count
        );
        crate::profile_counter!(
            "render",
            "error_proxy_source_validation_queue_saturated_count",
            self.source_validation_queue_saturated_count
        );
    }
}

impl MaterialPipelineAdmissionAccumulator {
    fn record(
        &mut self,
        requirement: ResolvedMaterialPipelineRequirement,
        admission: PipelineAdmission<()>,
    ) {
        self.requirement_count = self.requirement_count.saturating_add(1);
        match admission {
            PipelineAdmission::Ready(()) => {
                self.ready_count = self.ready_count.saturating_add(1);
            }
            PipelineAdmission::Deferred(unavailable) => {
                self.first_deferred
                    .get_or_insert((requirement, unavailable));
            }
            PipelineAdmission::Failed(unavailable) => {
                self.first_failed.get_or_insert((requirement, unavailable));
            }
        }
    }

    fn finish(self) -> MaterialPipelinePublicationAdmission {
        if let Some((requirement, unavailable)) = self.first_failed {
            return MaterialPipelinePublicationAdmission::Failed {
                requirement_count: self.requirement_count,
                ready_count: self.ready_count,
                requirement,
                unavailable,
            };
        }
        if let Some((requirement, unavailable)) = self.first_deferred {
            return MaterialPipelinePublicationAdmission::Deferred {
                requirement_count: self.requirement_count,
                ready_count: self.ready_count,
                requirement,
                unavailable,
            };
        }
        MaterialPipelinePublicationAdmission::Ready {
            requirement_count: self.requirement_count,
        }
    }
}

impl MeshPipelineCache {
    pub(crate) fn material_pipeline_requirements_are_ready_for_generation(
        &mut self,
        streamer: &ResourceStreamer,
        material_id: ResourceId,
        generation: u64,
        requirements: &MaterialPipelineRequirementSet,
    ) -> bool {
        let live_generations = streamer.material_draw_generations(&material_id);
        self.material_pipeline_generation_admissions
            .retain_live_generations(material_id, live_generations);
        let ready = self.material_pipeline_generation_admissions.contains_all(
            material_id,
            generation,
            requirements.iter(),
        );
        self.profile_material_generation_admission_ledger(material_id);
        ready
    }

    pub(crate) fn ensure_material_pipeline_requirements_for_generation(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        material_id: ResourceId,
        generation: u64,
        requirements: &MaterialPipelineRequirementSet,
    ) -> MaterialPipelinePublicationAdmission {
        crate::profile_scope!(
            "render",
            "shader_pipeline",
            "material_generation_requirement_admission"
        );
        if self.material_pipeline_requirements_are_ready_for_generation(
            streamer,
            material_id,
            generation,
            requirements,
        ) {
            crate::profile_counter!("render", "material_generation_admission_cache_hit", 1);
            crate::profile_counter!("render", "material_generation_admission_cache_miss", 0);
            return MaterialPipelinePublicationAdmission::Ready {
                requirement_count: requirements.len(),
            };
        }
        crate::profile_counter!("render", "material_generation_admission_cache_hit", 0);
        crate::profile_counter!("render", "material_generation_admission_cache_miss", 1);
        let result = self.ensure_material_pipeline_requirements_with_resolved(
            device,
            streamer,
            requirements,
        );
        let admission = result.admission;
        if matches!(
            admission,
            MaterialPipelinePublicationAdmission::Ready { .. }
        ) {
            debug_assert_eq!(result.resolved_pipelines.len(), requirements.len());
            self.material_pipeline_generation_admissions.record_ready(
                material_id,
                generation,
                requirements.iter(),
                result.resolved_pipelines,
            );
        }
        self.profile_material_generation_admission_ledger(material_id);
        admission
    }

    fn profile_material_generation_admission_ledger(&self, material_id: ResourceId) {
        crate::profile_counter!(
            "render",
            "material_generation_admission_material_count",
            self.material_pipeline_generation_admissions
                .material_count()
        );
        crate::profile_counter!(
            "render",
            "material_generation_admission_generation_count",
            self.material_pipeline_generation_admissions
                .generation_count(material_id)
        );
        crate::profile_counter!(
            "render",
            "material_generation_admission_requirement_count",
            self.material_pipeline_generation_admissions
                .requirement_count(material_id)
        );
        crate::profile_counter!(
            "render",
            "material_generation_admission_pinned_pipeline_count",
            self.material_pipeline_generation_admissions
                .pinned_resolved_pipeline_count()
        );
    }

    pub(crate) fn ensure_material_pipeline_requirements(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        requirements: &MaterialPipelineRequirementSet,
    ) -> MaterialPipelinePublicationAdmission {
        self.ensure_material_pipeline_requirements_with_resolved(device, streamer, requirements)
            .admission
    }

    fn ensure_material_pipeline_requirements_with_resolved(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        requirements: &MaterialPipelineRequirementSet,
    ) -> MaterialPipelineAdmissionResult {
        crate::profile_scope!(
            "render",
            "shader_pipeline",
            "material_requirement_admission"
        );
        let mut accumulator = MaterialPipelineAdmissionAccumulator::default();
        let mut resolved_pipelines = Vec::with_capacity(requirements.len());
        for requirement in requirements.iter() {
            let resolved = self.resolve_material_pipeline_requirement(requirement);
            let admission = self
                .ensure_resolved_material_pipeline_requirement(device, streamer, resolved, false);
            if matches!(&admission, PipelineAdmission::Ready(())) {
                resolved_pipelines.push(resolved);
            }
            accumulator.record(resolved, admission);
        }
        let admission = accumulator.finish();
        crate::profile_counter!(
            "render",
            "material_pipeline_requirement_count",
            admission.requirement_count()
        );
        crate::profile_counter!(
            "render",
            "material_pipeline_requirement_ready_count",
            admission.ready_count()
        );
        MaterialPipelineAdmissionResult {
            admission,
            resolved_pipelines,
        }
    }

    pub(crate) fn ensure_error_proxy_pipeline_requirements(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        requirements: &MaterialPipelineRequirementSet,
    ) -> Result<(), String> {
        crate::profile_scope!(
            "render",
            "shader_pipeline",
            "error_proxy_requirement_admission"
        );
        if requirements.len() == 0 {
            return Ok(());
        }
        let mut stats = ErrorProxyPipelineAdmissionStats::default();
        let result = (|| {
            for requirement in requirements.iter() {
                let resolved = self.resolve_material_pipeline_requirement(requirement);
                for attempt in 0..ERROR_PROXY_SOURCE_ADMISSION_ATTEMPTS {
                    stats.attempt_count = stats.attempt_count.saturating_add(1);
                    let admission = self.ensure_resolved_material_pipeline_requirement(
                        device, streamer, resolved, true,
                    );
                    match admission {
                        PipelineAdmission::Ready(()) => {
                            let shader_variant_key = self
                                .pipeline_and_shader_key_for_variant(resolved.variant_id())
                                .expect("resolved error proxy requirement must retain its shader key")
                                .2;
                            self.finish_pipeline_creation_diagnostics_for_variant(
                                &shader_variant_key,
                            )
                            .map_err(|message| {
                                format!(
                                    "error proxy {:?} pipeline validation failed: {message}",
                                    resolved.target()
                                )
                            })?;
                            break;
                        }
                        PipelineAdmission::Deferred(unavailable)
                            if attempt + 1 < ERROR_PROXY_SOURCE_ADMISSION_ATTEMPTS
                                && matches!(
                                    unavailable.reason(),
                                    crate::graphics::pipeline::PipelineAdmissionReason::SourceValidationQueued
                                        | crate::graphics::pipeline::PipelineAdmissionReason::SourceValidationPending
                                        | crate::graphics::pipeline::PipelineAdmissionReason::QueueSaturated
                                ) =>
                        {
                            let sync_started = std::time::Instant::now();
                            let completed_count =
                                self.finish_pending_shader_source_validations();
                            stats.record_source_validation_sync(
                                unavailable.reason(),
                                completed_count,
                                sync_started.elapsed(),
                            );
                        }
                        PipelineAdmission::Deferred(unavailable)
                        | PipelineAdmission::Failed(unavailable) => {
                            return Err(format!(
                                "error proxy {:?} pipeline admission failed for variant {:?}: {}",
                                resolved.target(),
                                resolved.variant_id(),
                                unavailable.reason().label()
                            ));
                        }
                    }
                }
            }
            Ok(())
        })();
        stats.profile();
        if result.is_ok() {
            crate::profile_counter!(
                "render",
                "error_proxy_pipeline_requirement_ready_count",
                requirements.len()
            );
        }
        result
    }

    fn resolve_material_pipeline_requirement(
        &mut self,
        requirement: &MaterialPipelineRequirement,
    ) -> ResolvedMaterialPipelineRequirement {
        let variant_id = self.resolve_variant_for_geometry(
            requirement.pipeline_kind(),
            &requirement.pipeline_key,
            requirement.geometry_source,
            requirement.shader_quality,
        );
        ResolvedMaterialPipelineRequirement::new(requirement.target(), variant_id)
    }

    fn ensure_resolved_material_pipeline_requirement(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        resolved: ResolvedMaterialPipelineRequirement,
        synchronous_error_proxy: bool,
    ) -> PipelineAdmission<()> {
        match resolved.target() {
            PipelineCreationTarget::Oit => self.ensure_oit_pipeline_admission_for_base_variant(
                device,
                streamer,
                resolved.variant_id(),
            ),
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base)
                if synchronous_error_proxy =>
            {
                self.ensure_synchronous_base_pipeline_admission_for_variant(
                    device,
                    streamer,
                    resolved.variant_id(),
                )
            }
            PipelineCreationTarget::MeshPass(kind) => self
                .ensure_material_mesh_pass_pipeline_requirement(
                    device,
                    streamer,
                    kind,
                    resolved.variant_id(),
                ),
        }
    }

    fn ensure_material_mesh_pass_pipeline_requirement(
        &mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        kind: MeshPassPipelineKind,
        variant_id: MeshPipelineVariantId,
    ) -> PipelineAdmission<()> {
        match kind {
            MeshPassPipelineKind::Base => {
                self.ensure_pipeline_admission_for_variant(device, streamer, variant_id)
            }
            MeshPassPipelineKind::GBuffer => {
                self.ensure_gbuffer_pipeline_admission_for_variant(device, streamer, variant_id)
            }
            MeshPassPipelineKind::DepthPrepass => self
                .ensure_depth_prepass_pipeline_admission_for_variant(device, streamer, variant_id),
            MeshPassPipelineKind::HitProxy => {
                self.ensure_hit_proxy_pipeline_admission_for_variant(device, streamer, variant_id)
            }
            MeshPassPipelineKind::ShadowDepth | MeshPassPipelineKind::ShadowDepthAlphaMask => self
                .ensure_shadow_pipeline_admission_for_variant(device, streamer, kind, variant_id),
            MeshPassPipelineKind::Velocity => {
                self.ensure_velocity_pipeline_admission_for_variant(device, streamer, variant_id)
            }
            MeshPassPipelineKind::TaaReactiveMask
            | MeshPassPipelineKind::TaaReactiveMaterialMask => self
                .ensure_taa_reactive_pipeline_admission_for_variant(
                    device, streamer, kind, variant_id,
                ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::core::framework::render::{GEOMETRY_SOURCE_ID_STATIC_MESH, ShaderQualityTier};
    use crate::graphics::pipeline::{PipelineAdmission, PipelineAdmissionReason};
    use crate::graphics::scene::resources::default_pipeline_key;

    use super::{
        MaterialPipelineAdmissionAccumulator, MaterialPipelinePublicationAdmission,
        MaterialPipelineRequirement, MaterialPipelineRequirementSet,
        ResolvedMaterialPipelineRequirement,
    };
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
        MeshPassPipelineKind, MeshPipelineVariantId,
    };
    use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::PipelineCreationTarget;

    #[test]
    fn requirement_set_deduplicates_exact_targets_without_collapsing_oit_and_base() {
        let base = MaterialPipelineRequirement::new(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            default_pipeline_key(),
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            ShaderQualityTier::Medium,
        );
        let oit = MaterialPipelineRequirement::new(
            PipelineCreationTarget::Oit,
            default_pipeline_key(),
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            ShaderQualityTier::Medium,
        );
        let mut requirements = MaterialPipelineRequirementSet::default();

        assert!(requirements.insert(base.clone()));
        assert!(!requirements.insert(base.clone()));
        assert!(requirements.insert(oit.clone()));
        assert_eq!(
            requirements.iter().cloned().collect::<Vec<_>>(),
            vec![base, oit]
        );
    }

    #[test]
    fn terminal_requirement_failure_wins_over_deferred_after_advancing_the_whole_set() {
        let deferred = ResolvedMaterialPipelineRequirement::new(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            MeshPipelineVariantId::new(3),
        );
        let failed = ResolvedMaterialPipelineRequirement::new(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::GBuffer),
            MeshPipelineVariantId::new(5),
        );
        let ready = ResolvedMaterialPipelineRequirement::new(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::DepthPrepass),
            MeshPipelineVariantId::new(7),
        );
        let mut accumulator = MaterialPipelineAdmissionAccumulator::default();
        accumulator.record(
            deferred,
            PipelineAdmission::unavailable(
                PipelineAdmissionReason::CompilePending,
                Duration::from_micros(11),
            ),
        );
        accumulator.record(
            failed,
            PipelineAdmission::unavailable(
                PipelineAdmissionReason::PipelineValidationFailed,
                Duration::from_micros(13),
            ),
        );
        accumulator.record(ready, PipelineAdmission::Ready(()));

        assert!(matches!(
            accumulator.finish(),
            MaterialPipelinePublicationAdmission::Failed {
                requirement_count: 3,
                ready_count: 1,
                requirement,
                unavailable,
            } if requirement == failed
                && unavailable.reason() == PipelineAdmissionReason::PipelineValidationFailed
        ));
    }

    #[test]
    fn material_requirement_admission_has_a_total_profile_scope() {
        let source = include_str!("material_pipeline_publication.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("material pipeline publication test boundary");

        assert!(source.contains("\"material_requirement_admission\""));
    }

    #[test]
    fn generation_admission_prunes_live_rows_and_records_only_complete_ready_sets() {
        let source = include_str!("material_pipeline_publication.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("material pipeline publication test boundary");
        let generation_admission = source
            .split("fn ensure_material_pipeline_requirements_for_generation(")
            .nth(1)
            .and_then(|source| {
                source
                    .split("pub(crate) fn ensure_material_pipeline_requirements(")
                    .next()
            })
            .expect("generation-qualified material admission");
        let retain = generation_admission
            .find("retain_live_generations")
            .expect("live generation pruning");
        let cache_lookup = generation_admission
            .find("contains_all")
            .expect("generation cache lookup");
        let admission = generation_admission
            .find("self.ensure_material_pipeline_requirements_with_resolved(")
            .expect("pipeline requirement admission");
        let ready = generation_admission
            .rfind("MaterialPipelinePublicationAdmission::Ready")
            .expect("complete Ready gate");
        let record = generation_admission
            .rfind("record_ready")
            .expect("generation admission publication");

        assert!(retain < cache_lookup);
        assert!(cache_lookup < admission);
        assert!(admission < ready);
        assert!(ready < record);
        assert!(generation_admission.contains("result.resolved_pipelines"));
        assert!(generation_admission.contains("material_generation_admission_cache_hit"));
        assert!(generation_admission.contains("material_generation_admission_cache_miss"));
        assert!(source.contains("material_generation_admission_pinned_pipeline_count"));
    }

    #[test]
    fn error_proxy_admission_uses_synchronous_base_and_finishes_validation() {
        let source = include_str!("material_pipeline_publication.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("material pipeline publication test boundary");
        let fallback = source
            .split("fn ensure_error_proxy_pipeline_requirements(")
            .nth(1)
            .expect("error proxy pipeline admission");

        assert!(fallback.contains("ensure_synchronous_base_pipeline_admission_for_variant"));
        assert!(fallback.contains("finish_pending_shader_source_validations"));
        assert!(fallback.contains("finish_pipeline_creation_diagnostics_for_variant"));
    }

    #[test]
    fn error_proxy_admission_profiles_source_validation_queue_debt() {
        let source = include_str!("material_pipeline_publication.rs")
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("material pipeline publication test boundary");

        for counter in [
            "error_proxy_pipeline_admission_attempt_count",
            "error_proxy_source_validation_sync_count",
            "error_proxy_source_validation_sync_completed_count",
            "error_proxy_source_validation_sync_wait_micros",
            "error_proxy_source_validation_queued_count",
            "error_proxy_source_validation_pending_count",
            "error_proxy_source_validation_queue_saturated_count",
        ] {
            assert!(
                source.contains(counter),
                "missing profile counter {counter}"
            );
        }
        assert!(source.contains("let completed_count ="));
        assert!(source.contains("self.finish_pending_shader_source_validations()"));
        let admission = source
            .split("fn ensure_error_proxy_pipeline_requirements(")
            .nth(1)
            .expect("error proxy pipeline admission");
        let empty_gate = admission
            .find("if requirements.len() == 0")
            .expect("empty fallback requirement gate");
        let stats = admission
            .find("ErrorProxyPipelineAdmissionStats::default()")
            .expect("error proxy queue-debt stats");
        assert!(empty_gate < stats);
    }
}
