use crate::scene::ecs::{
    Component, ComponentId, ComponentLifecycleEvent, LifecycleEventKind, ObserverId,
};
use crate::scene::EntityId;

use super::World;

impl World {
    pub fn observe_component_lifecycle<T>(
        &mut self,
        kind: LifecycleEventKind,
        observer: impl Fn(&mut World, ComponentLifecycleEvent) + Send + Sync + 'static,
    ) -> ObserverId
    where
        T: Component,
    {
        let component_id = self.component_id::<T>();
        self.observers
            .observe_lifecycle(kind, component_id, observer)
    }

    pub fn observe_event<E>(
        &mut self,
        observer: impl Fn(&mut World, &E) + Send + Sync + 'static,
    ) -> ObserverId
    where
        E: 'static + Send + Sync,
    {
        self.observers.observe_event(observer)
    }

    pub fn observe_entity_event<E>(
        &mut self,
        entity: EntityId,
        observer: impl Fn(&mut World, EntityId, &E) + Send + Sync + 'static,
    ) -> ObserverId
    where
        E: 'static + Send + Sync,
    {
        self.observers.observe_entity_event(entity, observer)
    }

    pub fn remove_observer(&mut self, observer: ObserverId) -> bool {
        self.observers.remove(observer)
    }

    pub fn trigger_event<E>(&mut self, event: E)
    where
        E: 'static + Send + Sync,
    {
        let callbacks = self.observers.event_callbacks::<E>();
        for callback in callbacks {
            callback(self, &event);
        }
    }

    pub fn trigger_entity_event<E>(&mut self, entity: EntityId, event: E)
    where
        E: 'static + Send + Sync,
    {
        let global_callbacks = self.observers.event_callbacks::<E>();
        let entity_callbacks = self.observers.entity_event_callbacks::<E>(entity);
        for callback in global_callbacks {
            callback(self, &event);
        }
        for callback in entity_callbacks {
            callback(self, entity, &event);
        }
    }

    pub(crate) fn trigger_component_lifecycle(
        &mut self,
        kind: LifecycleEventKind,
        entity: EntityId,
        component_id: ComponentId,
    ) {
        let Some(descriptor) = self.component_registry.descriptor(component_id) else {
            return;
        };
        let event =
            ComponentLifecycleEvent::new(kind, entity, component_id, descriptor.type_name.clone());
        let callbacks = self.observers.lifecycle_callbacks(kind, component_id);
        for callback in callbacks {
            callback(self, event.clone());
        }
    }
}
