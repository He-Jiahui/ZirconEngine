use std::time::Duration;

use crate::core::framework::render::{ShaderQualityTier, GEOMETRY_SOURCE_ID_STATIC_MESH};
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
    let source = include_str!("../material_pipeline_publication.rs")
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("material pipeline publication test boundary");

    assert!(source.contains("\"material_requirement_admission\""));
}

#[test]
fn generation_admission_prunes_live_rows_and_records_only_complete_ready_sets() {
    let source = include_str!("../material_pipeline_publication.rs")
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
    // BUG: [CR-R02-runtime_wave12_graphics_mesh_pipeline-0003] 此切片从代际准入方法起始，漏掉前一辅助方法中的 retain_live_generations/contains_all；find 恒为 None，expect 先于顺序断言 panic。
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
    let source = include_str!("../material_pipeline_publication.rs")
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
    let source = include_str!("../material_pipeline_publication.rs")
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
