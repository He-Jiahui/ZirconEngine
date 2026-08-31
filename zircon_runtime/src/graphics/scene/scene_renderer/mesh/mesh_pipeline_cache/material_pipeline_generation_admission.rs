use std::collections::{HashMap, HashSet};

use crate::core::resource::ResourceId;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPipelineVariantId;

use super::material_pipeline_publication::{
    MaterialPipelineRequirement, ResolvedMaterialPipelineRequirement,
};

#[derive(Default)]
struct MaterialPipelineGenerationAdmission {
    requirements: HashSet<MaterialPipelineRequirement>,
    resolved_pipelines: HashSet<ResolvedMaterialPipelineRequirement>,
}

#[derive(Default)]
pub(super) struct MaterialPipelineGenerationAdmissionLedger {
    materials: HashMap<ResourceId, HashMap<u64, MaterialPipelineGenerationAdmission>>,
    resolved_pipeline_pins: HashMap<ResolvedMaterialPipelineRequirement, usize>,
}

impl MaterialPipelineGenerationAdmissionLedger {
    pub(super) fn contains_all<'a>(
        &self,
        material_id: ResourceId,
        generation: u64,
        requirements: impl IntoIterator<Item = &'a MaterialPipelineRequirement>,
    ) -> bool {
        let Some(ready) = self
            .materials
            .get(&material_id)
            .and_then(|generations| generations.get(&generation))
        else {
            return false;
        };
        requirements
            .into_iter()
            .all(|requirement| ready.requirements.contains(requirement))
    }

    pub(super) fn record_ready<'a, R>(
        &mut self,
        material_id: ResourceId,
        generation: u64,
        requirements: impl IntoIterator<Item = &'a MaterialPipelineRequirement>,
        resolved_pipelines: R,
    ) where
        R: IntoIterator<Item = ResolvedMaterialPipelineRequirement>,
    {
        let admission = self
            .materials
            .entry(material_id)
            .or_default()
            .entry(generation)
            .or_default();
        admission
            .requirements
            .extend(requirements.into_iter().cloned());
        for resolved in resolved_pipelines {
            if !admission.resolved_pipelines.insert(resolved) {
                continue;
            }
            let pin_count = self.resolved_pipeline_pins.entry(resolved).or_default();
            *pin_count = pin_count
                .checked_add(1)
                .expect("material generation pipeline pin count overflow");
        }
    }

    pub(super) fn retain_live_generations(
        &mut self,
        material_id: ResourceId,
        live_generations: [Option<u64>; 3],
    ) {
        let Some(generations) = self.materials.get_mut(&material_id) else {
            return;
        };
        let mut retired_pipelines = Vec::new();
        generations.retain(|generation, admission| {
            let retained = live_generations
                .iter()
                .flatten()
                .any(|live| live == generation);
            if !retained {
                retired_pipelines.extend(admission.resolved_pipelines.iter().copied());
            }
            retained
        });
        let remove_material = generations.is_empty();
        for resolved in retired_pipelines {
            self.unpin_resolved_pipeline(resolved);
        }
        if remove_material {
            self.materials.remove(&material_id);
        }
    }

    fn unpin_resolved_pipeline(&mut self, resolved: ResolvedMaterialPipelineRequirement) {
        let remove = {
            let pin_count = self
                .resolved_pipeline_pins
                .get_mut(&resolved)
                .expect("retired material generation must own a resolved pipeline pin");
            *pin_count = pin_count
                .checked_sub(1)
                .expect("material generation pipeline pin count underflow");
            *pin_count == 0
        };
        if remove {
            self.resolved_pipeline_pins.remove(&resolved);
        }
    }

    pub(super) fn resolved_pipeline_pin_count(
        &self,
        target: super::PipelineCreationTarget,
        variant_id: MeshPipelineVariantId,
    ) -> usize {
        self.resolved_pipeline_pins
            .get(&ResolvedMaterialPipelineRequirement::new(
                target, variant_id,
            ))
            .copied()
            .unwrap_or(0)
    }

    pub(super) fn pinned_resolved_pipeline_count(&self) -> usize {
        self.resolved_pipeline_pins.len()
    }

    pub(super) fn material_count(&self) -> usize {
        self.materials.len()
    }

    pub(super) fn generation_count(&self, material_id: ResourceId) -> usize {
        self.materials.get(&material_id).map_or(0, HashMap::len)
    }

    pub(super) fn requirement_count(&self, material_id: ResourceId) -> usize {
        self.materials
            .get(&material_id)
            .map(|generations| {
                generations
                    .values()
                    .map(|admission| admission.requirements.len())
                    .sum()
            })
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{GEOMETRY_SOURCE_ID_STATIC_MESH, ShaderQualityTier};
    use crate::core::resource::ResourceId;
    use crate::graphics::scene::resources::default_pipeline_key;

    use super::super::PipelineCreationTarget;
    use super::super::material_pipeline_publication::{
        MaterialPipelineRequirement, ResolvedMaterialPipelineRequirement,
    };
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
            ledger.resolved_pipeline_pin_count(
                PipelineCreationTarget::Oit,
                oit_resolved.variant_id()
            ),
            1
        );
        assert_eq!(ledger.pinned_resolved_pipeline_count(), 2);
    }
}
