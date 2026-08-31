use std::any::TypeId;
use std::sync::Arc;

use crate::scene::ecs::{ComponentId, ComponentLifecycleEvent, LifecycleEventKind};
use crate::scene::{EntityId, World};

use super::ObserverId;
use super::callback_registry::IndexedObserver;
use super::{EntityEventCallback, EventCallback, LifecycleCallback};

pub(super) type LifecycleObserverKey = (LifecycleEventKind, ComponentId);
pub(super) type EntityEventObserverKey = (TypeId, EntityId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum ObserverBucket {
    Lifecycle(LifecycleObserverKey),
    Event(TypeId),
    EntityEvent(EntityEventObserverKey),
}

#[derive(Clone)]
pub(super) struct LifecycleObserver {
    pub(super) id: ObserverId,
    pub(super) callback: LifecycleCallback,
}

impl IndexedObserver for LifecycleObserver {
    fn observer_id(&self) -> ObserverId {
        self.id
    }
}

#[derive(Clone)]
pub(super) struct EventObserver {
    pub(super) id: ObserverId,
    pub(super) callback: EventCallback,
}

impl IndexedObserver for EventObserver {
    fn observer_id(&self) -> ObserverId {
        self.id
    }
}

#[derive(Clone)]
pub(super) struct EntityEventObserver {
    pub(super) id: ObserverId,
    pub(super) callback: EntityEventCallback,
}

impl IndexedObserver for EntityEventObserver {
    fn observer_id(&self) -> ObserverId {
        self.id
    }
}

pub(crate) struct LifecycleCallbackBucket {
    observers: Arc<Vec<LifecycleObserver>>,
}

impl LifecycleCallbackBucket {
    pub(super) fn new(observers: Arc<Vec<LifecycleObserver>>) -> Self {
        Self { observers }
    }

    pub(crate) fn dispatch(&self, world: &mut World, event: ComponentLifecycleEvent) {
        for observer in self.observers.iter() {
            (observer.callback)(world, &event);
        }
    }
}

pub(crate) struct EventCallbackBucket {
    observers: Arc<Vec<EventObserver>>,
}

impl EventCallbackBucket {
    pub(super) fn new(observers: Arc<Vec<EventObserver>>) -> Self {
        Self { observers }
    }

    pub(crate) fn dispatch<E>(&self, world: &mut World, event: &E)
    where
        E: 'static,
    {
        for observer in self.observers.iter() {
            (observer.callback)(world, event);
        }
    }
}

pub(crate) struct EntityEventCallbackBucket {
    observers: Arc<Vec<EntityEventObserver>>,
}

impl EntityEventCallbackBucket {
    pub(super) fn new(observers: Arc<Vec<EntityEventObserver>>) -> Self {
        Self { observers }
    }

    pub(crate) fn dispatch<E>(&self, world: &mut World, entity: EntityId, event: &E)
    where
        E: 'static,
    {
        for observer in self.observers.iter() {
            (observer.callback)(world, entity, event);
        }
    }
}
