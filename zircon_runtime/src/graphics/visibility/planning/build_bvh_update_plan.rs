use std::collections::HashMap;

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

    if is_sorted_by_stable_instance_key(current_instances)
        && is_sorted_by_stable_instance_key(&previous.instances)
    {
        return build_sorted_bvh_update_plan(current_instances, &previous.instances);
    }

    let previous_by_stable_instance_key = previous
        .instances
        .iter()
        .map(|entry| (entry.stable_instance_key, entry))
        .collect::<HashMap<_, _>>();
    let current_by_stable_instance_key = current_instances
        .iter()
        .map(|entry| (entry.stable_instance_key, entry))
        .collect::<HashMap<_, _>>();
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

fn is_sorted_by_stable_instance_key(entries: &[VisibilityHistoryEntry]) -> bool {
    entries
        .windows(2)
        .all(|pair| pair[0].stable_instance_key <= pair[1].stable_instance_key)
}

fn build_sorted_bvh_update_plan(
    current_instances: &[VisibilityHistoryEntry],
    previous_instances: &[VisibilityHistoryEntry],
) -> VisibilityBvhUpdatePlan {
    let mut inserted_stable_instance_keys = Vec::new();
    let mut updated_stable_instance_keys = Vec::new();
    let mut removed_stable_instance_keys = Vec::new();
    let mut current_index = 0;
    let mut previous_index = 0;

    while let (Some(current), Some(previous)) = (
        current_instances.get(current_index),
        previous_instances.get(previous_index),
    ) {
        match current
            .stable_instance_key
            .cmp(&previous.stable_instance_key)
        {
            std::cmp::Ordering::Less => {
                inserted_stable_instance_keys.push(current.stable_instance_key);
                current_index += 1;
            }
            std::cmp::Ordering::Equal => {
                if current != previous {
                    updated_stable_instance_keys.push(current.stable_instance_key);
                }
                current_index += 1;
                previous_index += 1;
            }
            std::cmp::Ordering::Greater => {
                removed_stable_instance_keys.push(previous.stable_instance_key);
                previous_index += 1;
            }
        }
    }
    inserted_stable_instance_keys.extend(
        current_instances[current_index..]
            .iter()
            .map(|entry| entry.stable_instance_key),
    );
    removed_stable_instance_keys.extend(
        previous_instances[previous_index..]
            .iter()
            .map(|entry| entry.stable_instance_key),
    );

    VisibilityBvhUpdatePlan {
        strategy: VisibilityBvhUpdateStrategy::Incremental,
        inserted_stable_instance_keys,
        updated_stable_instance_keys,
        removed_stable_instance_keys,
    }
}

#[cfg(test)]
#[path = "tests/build_bvh_update_plan_optimization_tests.rs"]
mod optimization_tests;
