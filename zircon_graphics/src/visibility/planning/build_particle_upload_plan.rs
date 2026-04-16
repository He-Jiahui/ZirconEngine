use std::collections::BTreeSet;

use super::super::declarations::{VisibilityHistorySnapshot, VisibilityParticleUploadPlan};

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

    let previous_emitters = previous
        .particle_emitters
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let current_emitters = current
        .particle_emitters
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let dirty_emitters = emitter_entities
        .iter()
        .copied()
        .filter(|entity| !previous_emitters.contains(entity))
        .collect::<Vec<_>>();
    let removed_emitters = previous
        .particle_emitters
        .iter()
        .copied()
        .filter(|entity| !current_emitters.contains(entity))
        .collect::<Vec<_>>();

    VisibilityParticleUploadPlan {
        emitter_entities,
        dirty_emitters,
        removed_emitters,
    }
}
