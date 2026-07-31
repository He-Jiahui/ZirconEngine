use std::any::TypeId;

use crate::scene::EntityId;
use crate::scene::ecs::{ComponentId, LifecycleEventKind};

use super::ObserverId;
use super::{EntityEventCallback, EventCallback, LifecycleCallback};

pub(super) struct LifecycleObserver {
    pub(super) id: ObserverId,
    pub(super) kind: LifecycleEventKind,
    pub(super) component_id: ComponentId,
    pub(super) callback: LifecycleCallback,
}

pub(super) struct EventObserver {
    pub(super) id: ObserverId,
    pub(super) event_type: TypeId,
    pub(super) callback: EventCallback,
}

pub(super) struct EntityEventObserver {
    pub(super) id: ObserverId,
    pub(super) event_type: TypeId,
    pub(super) entity: EntityId,
    pub(super) callback: EntityEventCallback,
}
