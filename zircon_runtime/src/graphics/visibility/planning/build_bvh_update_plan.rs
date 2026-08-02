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
            inserted_stable_instance_keys: current_instances
                .iter()
                .map(|entry| entry.stable_instance_key)
                .collect(),
            updated_stable_instance_keys: Vec::new(),
            removed_stable_instance_keys: Vec::new(),
        };
    };

    if previous.instances.is_empty() {
        return VisibilityBvhUpdatePlan {
            strategy: VisibilityBvhUpdateStrategy::FullRebuild,
            inserted_stable_instance_keys: current_instances
                .iter()
                .map(|entry| entry.stable_instance_key)
                .collect(),
            updated_stable_instance_keys: Vec::new(),
            removed_stable_instance_keys: Vec::new(),
        };
    }

    let previous_by_stable_instance_key = previous
        .instances
        .iter()
        .map(|entry| (entry.stable_instance_key, entry))
        .collect::<BTreeMap<_, _>>();
    let current_by_stable_instance_key = current_instances
        .iter()
        .map(|entry| (entry.stable_instance_key, entry))
        .collect::<BTreeMap<_, _>>();
    let inserted_stable_instance_keys = current_instances
        .iter()
        .filter(|entry| !previous_by_stable_instance_key.contains_key(&entry.stable_instance_key))
        .map(|entry| entry.stable_instance_key)
        .collect::<Vec<_>>();
    let updated_stable_instance_keys = current_instances
        .iter()
        .filter(|entry| {
            previous_by_stable_instance_key
                .get(&entry.stable_instance_key)
                .is_some_and(|old| **old != **entry)
        })
        .map(|entry| entry.stable_instance_key)
        .collect::<Vec<_>>();
    let removed_stable_instance_keys = previous
        .instances
        .iter()
        .filter(|entry| !current_by_stable_instance_key.contains_key(&entry.stable_instance_key))
        .map(|entry| entry.stable_instance_key)
        .collect::<Vec<_>>();

    VisibilityBvhUpdatePlan {
        strategy: VisibilityBvhUpdateStrategy::Incremental,
        inserted_stable_instance_keys,
        updated_stable_instance_keys,
        removed_stable_instance_keys,
    }
}
