use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::core::framework::scene::{EntityId, SceneResource};
use crate::core::math::Transform;

/// 单根骨骼的局部姿态与混合权重，是动画目标和模拟反馈共用的载荷。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkeletalPoseTarget {
    pub bone_name: String,
    pub local_transform: Transform,
    pub normalized_weight: f32,
}

/// 动画系统发布的逐实体目标姿态；物理固定更新在驱动布娃娃前读取。
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

    pub fn remove(&mut self, entity: EntityId) -> Option<Arc<[SkeletalPoseTarget]>> {
        self.entities.remove(&entity)
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }
}

impl SceneResource for SkeletalPoseTargets {}

/// 物理系统回传给动画的逐实体姿态；每次同步重新生成，世界替换时须清空旧值。
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
