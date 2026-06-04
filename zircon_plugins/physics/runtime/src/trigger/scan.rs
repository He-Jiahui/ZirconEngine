use zircon_runtime::core::framework::physics::{
    PhysicsSettings, PhysicsTriggerEvent, PhysicsTriggerEventKind, PhysicsWorldSyncState,
};

use crate::query_contact::{colliders_can_interact, colliders_overlap};

use super::event::trigger_event;
use super::pair::{trigger_pairs_for, PhysicsTriggerPairMap};
use super::point::trigger_point;

pub(super) fn compute_trigger_events(
    sync: &PhysicsWorldSyncState,
    settings: &PhysicsSettings,
    previous: &PhysicsTriggerPairMap,
) -> (PhysicsTriggerPairMap, Vec<PhysicsTriggerEvent>) {
    let current = collect_current_trigger_pairs(sync, settings);

    let mut events = Vec::new();
    for (pair, point) in &current {
        events.push(trigger_event(
            sync.world,
            *pair,
            if previous.contains_key(pair) {
                PhysicsTriggerEventKind::Stay
            } else {
                PhysicsTriggerEventKind::Enter
            },
            *point,
        ));
    }
    for (pair, point) in previous {
        if !current.contains_key(pair) {
            events.push(trigger_event(
                sync.world,
                *pair,
                PhysicsTriggerEventKind::Exit,
                *point,
            ));
        }
    }

    (current, events)
}

fn collect_current_trigger_pairs(
    sync: &PhysicsWorldSyncState,
    settings: &PhysicsSettings,
) -> PhysicsTriggerPairMap {
    let mut current = PhysicsTriggerPairMap::new();
    for left_index in 0..sync.colliders.len() {
        for right_index in left_index + 1..sync.colliders.len() {
            let left = &sync.colliders[left_index];
            let right = &sync.colliders[right_index];
            if !colliders_can_interact(left, right, settings) || !colliders_overlap(left, right) {
                continue;
            }

            for pair in trigger_pairs_for(left, right) {
                current.insert(pair, trigger_point(left, right));
            }
        }
    }
    current
}
