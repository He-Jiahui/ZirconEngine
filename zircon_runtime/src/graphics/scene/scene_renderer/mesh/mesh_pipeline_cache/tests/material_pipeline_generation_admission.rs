use crate::core::framework::render::{ShaderQualityTier, GEOMETRY_SOURCE_ID_STATIC_MESH};
use crate::core::resource::ResourceId;
use crate::graphics::scene::resources::default_pipeline_key;

use super::super::material_pipeline_publication::{
    MaterialPipelineRequirement, ResolvedMaterialPipelineRequirement,
};
use super::super::PipelineCreationTarget;
use super::MaterialPipelineGenerationAdmissionLedger;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassPipelineKind;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPipelineVariantId;

fn requirement(kind: MeshPassPipelineKind) -> MaterialPipelineRequirement {
    requirement_for_target(PipelineCreationTarget::MeshPass(kind))
}

fn requirement_for_target(target: PipelineCreationTarget) -> MaterialPipelineRequirement {
    MaterialPipelineRequirement::new(
        target,
        default_pipeline_key(),
        GEOMETRY_SOURCE_ID_STATIC_MESH,
        ShaderQualityTier::Medium,
    )
}

fn resolved(
    target: PipelineCreationTarget,
    variant_id: u32,
) -> ResolvedMaterialPipelineRequirement {
    ResolvedMaterialPipelineRequirement::new(target, MeshPipelineVariantId::new(variant_id))
}

#[test]
fn ready_requirements_are_unioned_within_one_material_generation() {
    let material_id = ResourceId::from_stable_label("res://tests/generation-ledger-union");
    let base = requirement(MeshPassPipelineKind::Base);
    let shadow = requirement(MeshPassPipelineKind::ShadowDepth);
    let mut ledger = MaterialPipelineGenerationAdmissionLedger::default();

    ledger.record_ready(
        material_id,
        7,
        [&base],
        [resolved(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            11,
        )],
    );
    ledger.record_ready(
        material_id,
        7,
        [&shadow],
        [resolved(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::ShadowDepth),
            13,
        )],
    );

    assert!(ledger.contains_all(material_id, 7, [&base, &shadow]));
    assert_eq!(ledger.generation_count(material_id), 1);
    assert_eq!(ledger.requirement_count(material_id), 2);
}

#[test]
fn retaining_live_current_previous_and_staged_generations_discards_reload_history() {
    let material_id = ResourceId::from_stable_label("res://tests/generation-ledger-retain");
    let base = requirement(MeshPassPipelineKind::Base);
    let mut ledger = MaterialPipelineGenerationAdmissionLedger::default();

    for generation in 1..=6 {
        ledger.record_ready(
            material_id,
            generation,
            [&base],
            [resolved(
                PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
                11,
            )],
        );
    }
    ledger.retain_live_generations(material_id, [Some(6), Some(5), Some(7)]);
    ledger.record_ready(
        material_id,
        7,
        [&base],
        [resolved(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            11,
        )],
    );

    assert!(ledger.contains_all(material_id, 7, [&base]));
    assert!(ledger.contains_all(material_id, 6, [&base]));
    assert!(ledger.contains_all(material_id, 5, [&base]));
    assert!(!ledger.contains_all(material_id, 4, [&base]));
    assert_eq!(ledger.generation_count(material_id), 3);
}

#[test]
fn removing_every_live_generation_removes_the_material_row() {
    let material_id = ResourceId::from_stable_label("res://tests/generation-ledger-remove");
    let base = requirement(MeshPassPipelineKind::Base);
    let mut ledger = MaterialPipelineGenerationAdmissionLedger::default();
    ledger.record_ready(
        material_id,
        1,
        [&base],
        [resolved(
            PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base),
            11,
        )],
    );

    ledger.retain_live_generations(material_id, [None, None, None]);

    assert_eq!(ledger.generation_count(material_id), 0);
    assert_eq!(ledger.material_count(), 0);
}

#[test]
fn duplicate_ready_observation_does_not_double_pin_one_generation() {
    let material_id = ResourceId::from_stable_label("res://tests/generation-ledger-dedup");
    let base = requirement(MeshPassPipelineKind::Base);
    let target = PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base);
    let resolved = resolved(target, 17);
    let mut ledger = MaterialPipelineGenerationAdmissionLedger::default();

    ledger.record_ready(material_id, 3, [&base], [resolved]);
    ledger.record_ready(material_id, 3, [&base], [resolved]);

    assert_eq!(
        ledger.resolved_pipeline_pin_count(target, resolved.variant_id()),
        1
    );
    assert_eq!(ledger.pinned_resolved_pipeline_count(), 1);
}

#[test]
fn shared_pipeline_stays_pinned_until_its_last_live_generation_leaves() {
    let material_id = ResourceId::from_stable_label("res://tests/generation-ledger-shared");
    let base = requirement(MeshPassPipelineKind::Base);
    let target = PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base);
    let resolved = resolved(target, 19);
    let mut ledger = MaterialPipelineGenerationAdmissionLedger::default();

    ledger.record_ready(material_id, 4, [&base], [resolved]);
    ledger.record_ready(material_id, 5, [&base], [resolved]);
    assert_eq!(
        ledger.resolved_pipeline_pin_count(target, resolved.variant_id()),
        2
    );

    ledger.retain_live_generations(material_id, [Some(5), None, None]);
    assert_eq!(
        ledger.resolved_pipeline_pin_count(target, resolved.variant_id()),
        1
    );

    ledger.retain_live_generations(material_id, [None, None, None]);
    assert_eq!(
        ledger.resolved_pipeline_pin_count(target, resolved.variant_id()),
        0
    );
    assert_eq!(ledger.pinned_resolved_pipeline_count(), 0);
}

#[test]
fn equal_numeric_variants_keep_base_and_oit_pins_isolated() {
    let material_id = ResourceId::from_stable_label("res://tests/generation-ledger-target");
    let base = requirement(MeshPassPipelineKind::Base);
    let base_target = PipelineCreationTarget::MeshPass(MeshPassPipelineKind::Base);
    let base_resolved = resolved(base_target, 23);
    let oit = requirement_for_target(PipelineCreationTarget::Oit);
    let oit_resolved = resolved(PipelineCreationTarget::Oit, 23);
    let mut ledger = MaterialPipelineGenerationAdmissionLedger::default();

    ledger.record_ready(material_id, 9, [&base, &oit], [base_resolved, oit_resolved]);

    assert_eq!(
        ledger.resolved_pipeline_pin_count(base_target, base_resolved.variant_id()),
        1
    );
    assert_eq!(
        ledger.resolved_pipeline_pin_count(PipelineCreationTarget::Oit, oit_resolved.variant_id()),
        1
    );
    assert_eq!(ledger.pinned_resolved_pipeline_count(), 2);
}
