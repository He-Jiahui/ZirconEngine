use crate::core::framework::animation::AnimationPoseHandle;
use crate::core::framework::scene::EntityId;
use crate::core::resource::ResourceId;

#[derive(Clone, Debug, PartialEq)]
pub struct RenderSkeletalPoseExtract {
    pub entity: EntityId,
    pub skeleton: ResourceId,
    pub pose: AnimationPoseHandle,
}
