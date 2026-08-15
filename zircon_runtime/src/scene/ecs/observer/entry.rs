use std::any::TypeId;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::scene::ecs::{ComponentId, ComponentLifecycleEvent, LifecycleEventKind};
use crate::scene::{EntityId, World};

use super::ObserverId;
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

#[derive(Clone)]
pub(super) struct EventObserver {
    pub(super) id: ObserverId,
    pub(super) callback: EventCallback,
}

#[derive(Clone)]
pub(super) struct EntityEventObserver {
    pub(super) id: ObserverId,
    pub(super) callback: EntityEventCallback,
}

pub(crate) struct LifecycleCallbackBucket {
    observers: Arc<BTreeMap<ObserverId, LifecycleObserver>>,
}

impl LifecycleCallbackBucket {
    pub(super) fn new(observers: Arc<BTreeMap<ObserverId, LifecycleObserver>>) -> Self {
        Self { observers }
    }

    pub(crate) fn dispatch(&self, world: &mut World, event: ComponentLifecycleEvent) {
        for observer in self.observers.values() {
            (observer.callback)(world, &event);
        }
    }
}

pub(crate) struct EventCallbackBucket {
    observers: Arc<BTreeMap<ObserverId, EventObserver>>,
}

impl EventCallbackBucket {
    pub(super) fn new(observers: Arc<BTreeMap<ObserverId, EventObserver>>) -> Self {
        Self { observers }
    }

    pub(crate) fn dispatch<E>(&self, world: &mut World, event: &E)
    where
        E: 'static,
    {
        for observer in self.observers.values() {
            (observer.callback)(world, event);
        }
    }
}

pub(crate) struct EntityEventCallbackBucket {
    observers: Arc<BTreeMap<ObserverId, EntityEventObserver>>,
}

impl EntityEventCallbackBucket {
    pub(super) fn new(observers: Arc<BTreeMap<ObserverId, EntityEventObserver>>) -> Self {
        Self { observers }
    }

    pub(crate) fn dispatch<E>(&self, world: &mut World, entity: EntityId, event: &E)
    where
        E: 'static,
    {
        for observer in self.observers.values() {
            (observer.callback)(world, entity, event);
        }
    }
}
