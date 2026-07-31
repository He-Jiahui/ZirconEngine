use std::any::TypeId;
use std::fmt;
use std::sync::Arc;

use crate::scene::ecs::{ComponentId, ComponentLifecycleEvent, LifecycleEventKind};
use crate::scene::{EntityId, World};

use super::ObserverId;
use super::callback_registry::{
    entity_event_callback_count, event_callback_count, lifecycle_callback_count,
    remove_observer_by_id,
};
use super::entry::{EntityEventObserver, EventObserver, LifecycleObserver};
use super::{EntityEventCallback, EventCallback, LifecycleCallback};

#[derive(Default)]
pub struct ObserverStore {
    next_id: u64,
    lifecycle_observers: Vec<LifecycleObserver>,
    event_observers: Vec<EventObserver>,
    entity_event_observers: Vec<EntityEventObserver>,
}

impl ObserverStore {
    pub fn observe_lifecycle(
        &mut self,
        kind: LifecycleEventKind,
        component_id: ComponentId,
        callback: impl Fn(&mut World, ComponentLifecycleEvent) + Send + Sync + 'static,
    ) -> ObserverId {
        let id = self.allocate_id();
        self.lifecycle_observers.push(LifecycleObserver {
            id,
            kind,
            component_id,
            callback: Arc::new(callback),
        });
        id
    }

    pub fn observe_event<E>(
        &mut self,
        callback: impl Fn(&mut World, &E) + Send + Sync + 'static,
    ) -> ObserverId
    where
        E: 'static + Send + Sync,
    {
        let id = self.allocate_id();
        self.event_observers.push(EventObserver {
            id,
            event_type: TypeId::of::<E>(),
            callback: Arc::new(move |world, event| {
                if let Some(event) = event.downcast_ref::<E>() {
                    callback(world, event);
                }
            }),
        });
        id
    }

    pub fn observe_entity_event<E>(
        &mut self,
        entity: EntityId,
        callback: impl Fn(&mut World, EntityId, &E) + Send + Sync + 'static,
    ) -> ObserverId
    where
        E: 'static + Send + Sync,
    {
        let id = self.allocate_id();
        self.entity_event_observers.push(EntityEventObserver {
            id,
            event_type: TypeId::of::<E>(),
            entity,
            callback: Arc::new(move |world, entity, event| {
                if let Some(event) = event.downcast_ref::<E>() {
                    callback(world, entity, event);
                }
            }),
        });
        id
    }

    pub fn remove(&mut self, id: ObserverId) -> bool {
        // Observer ids are allocated from one counter, so only one observer list can match.
        if remove_observer_by_id(&mut self.lifecycle_observers, id, |observer| observer.id) {
            return true;
        }
        if remove_observer_by_id(&mut self.event_observers, id, |observer| observer.id) {
            return true;
        }
        remove_observer_by_id(&mut self.entity_event_observers, id, |observer| observer.id)
    }

    pub(crate) fn lifecycle_callbacks(
        &self,
        kind: LifecycleEventKind,
        component_id: ComponentId,
    ) -> Vec<LifecycleCallback> {
        let callback_count =
            lifecycle_callback_count(&self.lifecycle_observers, kind, component_id);
        let mut callbacks = Vec::with_capacity(callback_count);
        for observer in &self.lifecycle_observers {
            if observer.kind == kind && observer.component_id == component_id {
                callbacks.push(observer.callback.clone());
            }
        }
        callbacks
    }

    pub(crate) fn event_callbacks<E>(&self) -> Vec<EventCallback>
    where
        E: 'static,
    {
        let event_type = TypeId::of::<E>();
        let callback_count = event_callback_count(&self.event_observers, event_type);
        let mut callbacks = Vec::with_capacity(callback_count);
        for observer in &self.event_observers {
            if observer.event_type == event_type {
                callbacks.push(observer.callback.clone());
            }
        }
        callbacks
    }

    pub(crate) fn entity_event_callbacks<E>(&self, entity: EntityId) -> Vec<EntityEventCallback>
    where
        E: 'static,
    {
        let event_type = TypeId::of::<E>();
        let callback_count =
            entity_event_callback_count(&self.entity_event_observers, event_type, entity);
        let mut callbacks = Vec::with_capacity(callback_count);
        for observer in &self.entity_event_observers {
            if observer.event_type == event_type && observer.entity == entity {
                callbacks.push(observer.callback.clone());
            }
        }
        callbacks
    }

    fn allocate_id(&mut self) -> ObserverId {
        let id = ObserverId::new(self.next_id);
        self.next_id += 1;
        id
    }
}

impl fmt::Debug for ObserverStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ObserverStore")
            .field("next_id", &self.next_id)
            .field("lifecycle_count", &self.lifecycle_observers.len())
            .field("event_count", &self.event_observers.len())
            .field("entity_event_count", &self.entity_event_observers.len())
            .finish()
    }
}

impl Clone for ObserverStore {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for ObserverStore {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
