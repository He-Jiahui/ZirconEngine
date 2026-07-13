use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{EntityId, SceneResource};
use crate::core::math::Transform;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkeletalPoseTarget {
    pub bone_name: String,
    pub local_transform: Transform,
    pub normalized_weight: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkeletalPoseTargets {
    entities: BTreeMap<EntityId, Arc<[SkeletalPoseTarget]>>,
}

impl SkeletalPoseTargets {
    pub fn replace(&mut self, entity: EntityId, targets: Arc<[SkeletalPoseTarget]>) {
        self.entities.insert(entity, targets);
    }

    pub fn targets(&self, entity: EntityId) -> Option<&Arc<[SkeletalPoseTarget]>> {
        self.entities.get(&entity)
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }
}

impl SceneResource for SkeletalPoseTargets {}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SimulatedPoseFeed {
    entities: BTreeMap<EntityId, Arc<[SkeletalPoseTarget]>>,
}

impl SimulatedPoseFeed {
    pub fn replace(&mut self, entity: EntityId, targets: Arc<[SkeletalPoseTarget]>) {
        self.entities.insert(entity, targets);
    }

    pub fn targets(&self, entity: EntityId) -> Option<&Arc<[SkeletalPoseTarget]>> {
        self.entities.get(&entity)
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }
}

impl SceneResource for SimulatedPoseFeed {}
