use crate::scene::{ecs::ComponentId, EntityId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LifecycleEventKind {
    Add,
    Insert,
    Replace,
    Remove,
    Despawn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentLifecycleEvent {
    kind: LifecycleEventKind,
    entity: EntityId,
    component_id: ComponentId,
    component_type_name: String,
}

impl ComponentLifecycleEvent {
    pub fn new(
        kind: LifecycleEventKind,
        entity: EntityId,
        component_id: ComponentId,
        component_type_name: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            entity,
            component_id,
            component_type_name: component_type_name.into(),
        }
    }

    pub const fn kind(&self) -> LifecycleEventKind {
        self.kind
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    pub const fn component_id(&self) -> ComponentId {
        self.component_id
    }

    pub fn component_type_name(&self) -> &str {
        &self.component_type_name
    }
}
