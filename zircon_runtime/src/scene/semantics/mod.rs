use crate::scene::EntityId;

pub trait EntityIdentity: Copy + Eq + Send + Sync {
    fn entity_id(self) -> EntityId;
}

pub trait ComponentData: Send + Sync + 'static {}

impl EntityIdentity for EntityId {
    fn entity_id(self) -> EntityId {
        self
    }
}
