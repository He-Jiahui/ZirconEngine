use zircon_framework::scene::EntityId;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityParticleUploadPlan {
    pub emitter_entities: Vec<EntityId>,
    pub dirty_emitters: Vec<EntityId>,
    pub removed_emitters: Vec<EntityId>,
}

