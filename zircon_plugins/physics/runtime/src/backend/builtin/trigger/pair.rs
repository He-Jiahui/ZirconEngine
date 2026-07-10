use std::collections::BTreeMap;

use zircon_runtime::core::framework::physics::PhysicsColliderSyncState;
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::Real;

pub(crate) type PhysicsTriggerPairMap = BTreeMap<PhysicsTriggerPair, [Real; 3]>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PhysicsTriggerPair {
    pub(super) trigger_entity: EntityId,
    pub(super) other_entity: EntityId,
}

pub(super) fn trigger_pairs_for(
    left: &PhysicsColliderSyncState,
    right: &PhysicsColliderSyncState,
) -> Vec<PhysicsTriggerPair> {
    let mut pairs = Vec::with_capacity(2);
    if left.sensor {
        pairs.push(PhysicsTriggerPair {
            trigger_entity: left.entity,
            other_entity: right.entity,
        });
    }
    if right.sensor {
        pairs.push(PhysicsTriggerPair {
            trigger_entity: right.entity,
            other_entity: left.entity,
        });
    }
    pairs
}
