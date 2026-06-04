use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{EntityId, WorldHandle};
use crate::core::math::Real;

use super::PhysicsTriggerEventKind;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PhysicsTriggerEvent {
    pub world: WorldHandle,
    pub kind: PhysicsTriggerEventKind,
    pub trigger_entity: EntityId,
    pub other_entity: EntityId,
    pub point: [Real; 3],
}
