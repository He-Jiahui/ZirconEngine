use zircon_runtime::core::framework::physics::{
    PhysicsColliderSyncState, PhysicsContactEvent, PhysicsSettings, PhysicsWorldSyncState,
};
use zircon_runtime::core::math::{Real, Vec3};

use super::filter::colliders_can_interact;
use super::geometry::{midpoint, normalized_offset_or_zero};
use super::overlap::colliders_overlap;

pub(super) fn compute_contact_events(
    sync: &PhysicsWorldSyncState,
    settings: &PhysicsSettings,
) -> Vec<PhysicsContactEvent> {
    let mut contacts = Vec::new();
    for left_index in 0..sync.colliders.len() {
        for right_index in left_index + 1..sync.colliders.len() {
            let left = &sync.colliders[left_index];
            let right = &sync.colliders[right_index];
            if !colliders_can_contact(left, right, settings) || !colliders_overlap(left, right) {
                continue;
            }

            let left_center = left.transform.translation;
            let right_center = right.transform.translation;
            let mut normal = normalized_offset_or_zero(right_center, left_center);
            if normal.length_squared() <= Real::EPSILON {
                normal = Vec3::Y;
            }
            let point = midpoint(left_center, right_center);
            contacts.push(PhysicsContactEvent {
                world: sync.world,
                entity: left.entity,
                other_entity: right.entity,
                point: point.to_array(),
                normal: normal.to_array(),
            });
        }
    }
    contacts
}

fn colliders_can_contact(
    left: &PhysicsColliderSyncState,
    right: &PhysicsColliderSyncState,
    settings: &PhysicsSettings,
) -> bool {
    if left.sensor || right.sensor {
        return false;
    }

    colliders_can_interact(left, right, settings)
}
