use std::collections::BTreeMap;

use super::super::declarations::{
    VisibilityBvhUpdatePlan, VisibilityBvhUpdateStrategy, VisibilityHistoryEntry,
    VisibilityHistorySnapshot,
};

pub(crate) fn build_bvh_update_plan(
    current_instances: &[VisibilityHistoryEntry],
    previous: Option<&VisibilityHistorySnapshot>,
) -> VisibilityBvhUpdatePlan {
    let Some(previous) = previous else {
        return VisibilityBvhUpdatePlan {
            strategy: VisibilityBvhUpdateStrategy::FullRebuild,
            inserted_entities: current_instances.iter().map(|entry| entry.entity).collect(),
            updated_entities: Vec::new(),
            removed_entities: Vec::new(),
        };
    };

    if previous.instances.is_empty() {
        return VisibilityBvhUpdatePlan {
            strategy: VisibilityBvhUpdateStrategy::FullRebuild,
            inserted_entities: current_instances.iter().map(|entry| entry.entity).collect(),
            updated_entities: Vec::new(),
            removed_entities: Vec::new(),
        };
    }

    let previous_by_entity = previous
        .instances
        .iter()
        .map(|entry| (entry.entity, entry))
        .collect::<BTreeMap<_, _>>();
    let current_by_entity = current_instances
        .iter()
        .map(|entry| (entry.entity, entry))
        .collect::<BTreeMap<_, _>>();
    let inserted_entities = current_instances
        .iter()
        .filter(|entry| !previous_by_entity.contains_key(&entry.entity))
        .map(|entry| entry.entity)
        .collect::<Vec<_>>();
    let updated_entities = current_instances
        .iter()
        .filter(|entry| {
            previous_by_entity
                .get(&entry.entity)
                .is_some_and(|old| **old != **entry)
        })
        .map(|entry| entry.entity)
        .collect::<Vec<_>>();
    let removed_entities = previous
        .instances
        .iter()
        .filter(|entry| !current_by_entity.contains_key(&entry.entity))
        .map(|entry| entry.entity)
        .collect::<Vec<_>>();

    VisibilityBvhUpdatePlan {
        strategy: VisibilityBvhUpdateStrategy::Incremental,
        inserted_entities,
        updated_entities,
        removed_entities,
    }
}
