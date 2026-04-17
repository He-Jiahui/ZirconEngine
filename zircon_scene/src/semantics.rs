use crate::{EntityId, LevelSystem};

pub trait RuntimeObject {
    fn object_kind(&self) -> &'static str;
}

pub trait RuntimeSystem: RuntimeObject {
    fn system_name(&self) -> &'static str;
}

pub trait EntityIdentity: Copy + Eq + Send + Sync {
    fn entity_id(self) -> EntityId;
}

pub trait ComponentData: Send + Sync + 'static {}

impl RuntimeObject for LevelSystem {
    fn object_kind(&self) -> &'static str {
        "system"
    }
}

impl RuntimeSystem for LevelSystem {
    fn system_name(&self) -> &'static str {
        "LevelSystem"
    }
}

impl EntityIdentity for EntityId {
    fn entity_id(self) -> EntityId {
        self
    }
}
