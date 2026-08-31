use crate::scene::EntityId;
use crate::scene::ecs::{
    Component, ComponentId, ComponentLifecycleEvent, LifecycleEventKind, ObserverId,
};

use super::{SceneResult, World};

impl World {
    pub fn observe_component_lifecycle<T>(
        &mut self,
        kind: LifecycleEventKind,
        observer: impl Fn(&mut World, &ComponentLifecycleEvent) + Send + Sync + 'static,
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

    pub fn remove_observer(&mut self, observer: ObserverId) -> SceneResult<()> {
        self.observers.remove(observer)
    }

    pub fn trigger_event<E>(&mut self, event: E)
    where
        E: 'static + Send + Sync,
    {
        let Some(callbacks) = self.observers.event_callbacks::<E>() else {
            return;
        };
        callbacks.dispatch(self, &event);
    }

    pub fn trigger_entity_event<E>(&mut self, entity: EntityId, event: E)
    where
        E: 'static + Send + Sync,
    {
        if let Some(callbacks) = self.observers.event_callbacks::<E>() {
            callbacks.dispatch(self, &event);
        }
        if let Some(callbacks) = self.observers.entity_event_callbacks::<E>(entity) {
            callbacks.dispatch(self, entity, &event);
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
            ComponentLifecycleEvent::new(kind, entity, component_id, descriptor.type_name.as_str());
        if self.record_staged_lifecycle_events {
            self.staged_lifecycle_events.push(event);
            return;
        }
        self.dispatch_component_lifecycle(event);
    }

    pub(in crate::scene) fn dispatch_component_lifecycle(
        &mut self,
        event: ComponentLifecycleEvent,
    ) {
        let Some(callbacks) = self
            .observers
            .lifecycle_callbacks(event.kind(), event.component_id())
        else {
            return;
        };
        callbacks.dispatch(self, event);
    }
}
