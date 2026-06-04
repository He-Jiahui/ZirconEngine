use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicsSkeletonJointBinding {
    pub skeleton_entity: EntityId,
    pub bone_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_bone_path: Option<String>,
}
