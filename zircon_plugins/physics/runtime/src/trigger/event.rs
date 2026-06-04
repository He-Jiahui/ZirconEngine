use zircon_runtime::core::framework::physics::{PhysicsTriggerEvent, PhysicsTriggerEventKind};
use zircon_runtime::core::framework::scene::WorldHandle;
use zircon_runtime::core::math::Real;

use super::pair::PhysicsTriggerPair;

pub(super) fn trigger_event(
    world: WorldHandle,
    pair: PhysicsTriggerPair,
    kind: PhysicsTriggerEventKind,
    point: [Real; 3],
) -> PhysicsTriggerEvent {
    PhysicsTriggerEvent {
        world,
        kind,
        trigger_entity: pair.trigger_entity,
        other_entity: pair.other_entity,
        point,
    }
}
