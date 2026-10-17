use serde::{Deserialize, Serialize};

use crate::core::framework::scene::EntityId;

/// Persisted link from a physics joint to an authored skeleton bone.
///
/// 拥有骨架实体键与骨路径文本，供关节声明跨持久化边界传递。
/// 默认实体键为零、骨路径为空；派生 serde 不核实实体或骨架存在，插件另行检查路径非空。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhysicsSkeletonJointBinding {
    /// 关联骨架的实体键，该值不持有实体，也不绑定特定 World。
    pub skeleton_entity: EntityId,
    /// 目标骨路径文本，消费方负责解释和查找。
    pub bone_path: String,
    /// 可选父骨路径，serde 缺失时为 `None`，未指定时不写出。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_bone_path: Option<String>,
}
