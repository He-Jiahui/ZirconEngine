use super::super::declarations::{VisibilityHistorySnapshot, VisibilityParticleUploadPlan};
use crate::core::framework::scene::EntityId;

fn sorted_difference(source: &[EntityId], comparison: &[EntityId]) -> Vec<EntityId> {
    let mut difference = Vec::new();
    let mut comparison_index = 0;
    for entity in source.iter().copied() {
        while comparison
            .get(comparison_index)
            .is_some_and(|candidate| *candidate < entity)
        {
            comparison_index += 1;
        }
        if comparison.get(comparison_index).copied() != Some(entity) {
            difference.push(entity);
        }
    }
    difference
}

pub(crate) fn build_particle_upload_plan(
    current: &VisibilityHistorySnapshot,
    previous: Option<&VisibilityHistorySnapshot>,
) -> VisibilityParticleUploadPlan {
    let emitter_entities = current.particle_emitters.clone();
    let Some(previous) = previous else {
        return VisibilityParticleUploadPlan {
            emitter_entities: emitter_entities.clone(),
            dirty_emitters: emitter_entities,
            removed_emitters: Vec::new(),
        };
    };

    if previous.particle_emitters.is_empty() {
        return VisibilityParticleUploadPlan {
            emitter_entities: emitter_entities.clone(),
            dirty_emitters: emitter_entities,
            removed_emitters: Vec::new(),
        };
    }

    let dirty_emitters = sorted_difference(&emitter_entities, &previous.particle_emitters);
    let removed_emitters = sorted_difference(&previous.particle_emitters, &emitter_entities);

    VisibilityParticleUploadPlan {
        emitter_entities,
        dirty_emitters,
        removed_emitters,
    }
}

#[cfg(test)]
#[path = "tests/build_particle_upload_plan_optimization_tests.rs"]
mod optimization_tests;
