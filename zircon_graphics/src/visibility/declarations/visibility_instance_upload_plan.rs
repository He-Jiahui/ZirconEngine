use zircon_framework::scene::EntityId;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VisibilityInstanceUploadPlan {
    pub static_instance_entities: Vec<EntityId>,
    pub dynamic_instance_entities: Vec<EntityId>,
    pub dirty_dynamic_entities: Vec<EntityId>,
}

