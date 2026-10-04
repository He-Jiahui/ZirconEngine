use crate::core::framework::animation::AnimationPoseHandle;
use crate::core::framework::scene::EntityId;
use crate::core::resource::ResourceId;

/// `LevelSystem` 将同一世界代的动画姿态附加到场景帧时使用的关联项。
/// 渲染端凭实体与骨架资源身份把姿态匹配到对应网格。
#[derive(Clone, Debug, PartialEq)]
pub struct RenderSkeletalPoseExtract {
    pub entity: EntityId,
    pub skeleton: ResourceId,
    pub pose: AnimationPoseHandle,
}
