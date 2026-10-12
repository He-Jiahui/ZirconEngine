use crate::core::framework::scene::EntityId;

/// 由本帧和前帧 emitter 集合推导的粒子上传与移除请求。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityParticleUploadPlan {
    pub emitter_entities: Vec<EntityId>,
    pub dirty_emitters: Vec<EntityId>,
    pub removed_emitters: Vec<EntityId>,
}
