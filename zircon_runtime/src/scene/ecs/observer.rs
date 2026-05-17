use std::any::{Any, TypeId};
use std::fmt;
use std::sync::Arc;

use crate::scene::ecs::{ComponentId, ComponentLifecycleEvent, LifecycleEventKind};
use crate::scene::{EntityId, World};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObserverId(u64);

impl ObserverId {
    pub const fn new(index: u64) -> Self {
        Self(index)
    }

    pub const fn index(self) -> u64 {
        self.0
    }
}

type LifecycleCallback = Arc<dyn Fn(&mut World, ComponentLifecycleEvent) + Send + Sync>;
type EventCallback = Arc<dyn Fn(&mut World, &dyn Any) + Send + Sync>;
type EntityEventCallback = Arc<dyn Fn(&mut World, EntityId, &dyn Any) + Send + Sync>;

#[derive(Default)]
pub struct ObserverStore {
    next_id: u64,
    lifecycle_observers: Vec<LifecycleObserver>,
    event_observers: Vec<EventObserver>,
    entity_event_observers: Vec<EntityEventObserver>,
}

struct LifecycleObserver {
    id: ObserverId,
    kind: LifecycleEventKind,
    component_id: ComponentId,
    callback: LifecycleCallback,
}

struct EventObserver {
    id: ObserverId,
    event_type: TypeId,
    callback: EventCallback,
}

struct EntityEventObserver {
    id: ObserverId,
    event_type: TypeId,
    entity: EntityId,
    callback: EntityEventCallback,
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
        let before = self.total_len();
        self.lifecycle_observers
            .retain(|observer| observer.id != id);
        self.event_observers.retain(|observer| observer.id != id);
        self.entity_event_observers
            .retain(|observer| observer.id != id);
        self.total_len() != before
    }

    pub(crate) fn lifecycle_callbacks(
        &self,
        kind: LifecycleEventKind,
        component_id: ComponentId,
    ) -> Vec<LifecycleCallback> {
        self.lifecycle_observers
            .iter()
            .filter(|observer| observer.kind == kind && observer.component_id == component_id)
            .map(|observer| observer.callback.clone())
            .collect()
    }

    pub(crate) fn event_callbacks<E>(&self) -> Vec<EventCallback>
    where
        E: 'static,
    {
        let event_type = TypeId::of::<E>();
        self.event_observers
            .iter()
            .filter(|observer| observer.event_type == event_type)
            .map(|observer| observer.callback.clone())
            .collect()
    }

    pub(crate) fn entity_event_callbacks<E>(&self, entity: EntityId) -> Vec<EntityEventCallback>
    where
        E: 'static,
    {
        let event_type = TypeId::of::<E>();
        self.entity_event_observers
            .iter()
            .filter(|observer| observer.event_type == event_type && observer.entity == entity)
            .map(|observer| observer.callback.clone())
            .collect()
    }

    fn allocate_id(&mut self) -> ObserverId {
        let id = ObserverId::new(self.next_id);
        self.next_id += 1;
        id
    }

    fn total_len(&self) -> usize {
        self.lifecycle_observers.len()
            + self.event_observers.len()
            + self.entity_event_observers.len()
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
