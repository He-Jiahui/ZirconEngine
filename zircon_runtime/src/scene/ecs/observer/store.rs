use std::any::TypeId;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::sync::Arc;

use crate::scene::ecs::{ComponentId, ComponentLifecycleEvent, LifecycleEventKind};
use crate::scene::{EntityId, SceneError, SceneResult, World};

use super::callback_registry::{insert_observer_into_bucket, remove_observer_from_indexed_bucket};
use super::entry::{
    EntityEventCallbackBucket, EntityEventObserver, EntityEventObserverKey, EventCallbackBucket,
    EventObserver, LifecycleCallbackBucket, LifecycleObserver, LifecycleObserverKey,
    ObserverBucket,
};
use super::ObserverId;

#[derive(Default)]
pub struct ObserverStore {
    next_id: u64,
    lifecycle_buckets: HashMap<LifecycleObserverKey, Arc<BTreeMap<ObserverId, LifecycleObserver>>>,
    event_buckets: HashMap<TypeId, Arc<BTreeMap<ObserverId, EventObserver>>>,
    entity_event_buckets:
        HashMap<EntityEventObserverKey, Arc<BTreeMap<ObserverId, EntityEventObserver>>>,
    entity_event_types_by_entity: HashMap<EntityId, HashSet<TypeId>>,
    observer_locations: HashMap<ObserverId, ObserverBucket>,
}

pub(crate) struct DetachedEntityObservers {
    entity: EntityId,
    event_types: HashSet<TypeId>,
    buckets: Vec<(
        EntityEventObserverKey,
        Arc<BTreeMap<ObserverId, EntityEventObserver>>,
    )>,
}

impl ObserverStore {
    pub fn observe_lifecycle(
        &mut self,
        kind: LifecycleEventKind,
        component_id: ComponentId,
        callback: impl Fn(&mut World, &ComponentLifecycleEvent) + Send + Sync + 'static,
    ) -> ObserverId {
        let id = self.allocate_id();
        let key = (kind, component_id);
        let observer = LifecycleObserver {
            id,
            callback: Arc::new(callback),
        };
        insert_observer_into_bucket(self.lifecycle_buckets.entry(key).or_default(), id, observer);
        self.observer_locations
            .insert(id, ObserverBucket::Lifecycle(key));
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
        let event_type = TypeId::of::<E>();
        let observer = EventObserver {
            id,
            callback: Arc::new(move |world, event| {
                if let Some(event) = event.downcast_ref::<E>() {
                    callback(world, event);
                }
            }),
        };
        insert_observer_into_bucket(
            self.event_buckets.entry(event_type).or_default(),
            id,
            observer,
        );
        self.observer_locations
            .insert(id, ObserverBucket::Event(event_type));
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
        let key = (TypeId::of::<E>(), entity);
        let observer = EntityEventObserver {
            id,
            callback: Arc::new(move |world, entity, event| {
                if let Some(event) = event.downcast_ref::<E>() {
                    callback(world, entity, event);
                }
            }),
        };
        insert_observer_into_bucket(
            self.entity_event_buckets.entry(key).or_default(),
            id,
            observer,
        );
        self.entity_event_types_by_entity
            .entry(entity)
            .or_default()
            .insert(key.0);
        self.observer_locations
            .insert(id, ObserverBucket::EntityEvent(key));
        id
    }

    pub fn remove(&mut self, id: ObserverId) -> SceneResult<()> {
        let Some(location) = self.observer_locations.get(&id).copied() else {
            return Err(SceneError::MissingObserver { observer: id });
        };
        let removed = match location {
            ObserverBucket::Lifecycle(key) => self.remove_lifecycle_observer(key, id),
            ObserverBucket::Event(event_type) => self.remove_event_observer(event_type, id),
            ObserverBucket::EntityEvent(key) => self.remove_entity_event_observer(key, id),
        };
        if !removed {
            return Err(SceneError::MissingObserver { observer: id });
        }
        self.observer_locations.remove(&id);
        Ok(())
    }

    pub(crate) fn lifecycle_callbacks(
        &self,
        kind: LifecycleEventKind,
        component_id: ComponentId,
    ) -> Option<LifecycleCallbackBucket> {
        let key = (kind, component_id);
        let observers = self.lifecycle_buckets.get(&key)?;
        Some(LifecycleCallbackBucket::new(Arc::clone(observers)))
    }

    pub(crate) fn event_callbacks<E>(&self) -> Option<EventCallbackBucket>
    where
        E: 'static,
    {
        let observers = self.event_buckets.get(&TypeId::of::<E>())?;
        Some(EventCallbackBucket::new(Arc::clone(observers)))
    }

    pub(crate) fn entity_event_callbacks<E>(
        &self,
        entity: EntityId,
    ) -> Option<EntityEventCallbackBucket>
    where
        E: 'static,
    {
        let key = (TypeId::of::<E>(), entity);
        let observers = self.entity_event_buckets.get(&key)?;
        Some(EntityEventCallbackBucket::new(Arc::clone(observers)))
    }

    pub(crate) fn remove_entity_observers(&mut self, entity: EntityId) {
        let Some(event_types) = self.entity_event_types_by_entity.remove(&entity) else {
            return;
        };
        for event_type in event_types {
            let key = (event_type, entity);
            let Some(bucket) = self.entity_event_buckets.remove(&key) else {
                continue;
            };
            for observer in bucket.values() {
                self.observer_locations.remove(&observer.id);
            }
        }
    }

    pub(crate) fn detach_entity_observers(&mut self, entity: EntityId) -> DetachedEntityObservers {
        let event_types = self
            .entity_event_types_by_entity
            .remove(&entity)
            .unwrap_or_default();
        let mut buckets = Vec::with_capacity(event_types.len());
        for event_type in &event_types {
            let key = (*event_type, entity);
            if let Some(bucket) = self.entity_event_buckets.remove(&key) {
                for observer in bucket.values() {
                    self.observer_locations.remove(&observer.id);
                }
                buckets.push((key, bucket));
            }
        }
        DetachedEntityObservers {
            entity,
            event_types,
            buckets,
        }
    }

    pub(crate) fn restore_detached_entity_observers(&mut self, detached: DetachedEntityObservers) {
        debug_assert!(detached
            .buckets
            .iter()
            .all(|(key, _)| key.1 == detached.entity));
        if detached.event_types.is_empty() {
            return;
        }
        let previous = self
            .entity_event_types_by_entity
            .insert(detached.entity, detached.event_types);
        debug_assert!(previous.is_none());
        for (key, bucket) in detached.buckets {
            for observer in bucket.values() {
                let previous = self
                    .observer_locations
                    .insert(observer.id, ObserverBucket::EntityEvent(key));
                debug_assert!(previous.is_none());
            }
            let previous = self.entity_event_buckets.insert(key, bucket);
            debug_assert!(previous.is_none());
        }
    }

    fn remove_lifecycle_observer(&mut self, key: LifecycleObserverKey, id: ObserverId) -> bool {
        let Some(bucket) = self.lifecycle_buckets.get_mut(&key) else {
            return false;
        };
        if !remove_observer_from_indexed_bucket(bucket, id) {
            return false;
        }
        if bucket.is_empty() {
            self.lifecycle_buckets.remove(&key);
        }
        true
    }

    fn remove_event_observer(&mut self, event_type: TypeId, id: ObserverId) -> bool {
        let Some(bucket) = self.event_buckets.get_mut(&event_type) else {
            return false;
        };
        if !remove_observer_from_indexed_bucket(bucket, id) {
            return false;
        }
        if bucket.is_empty() {
            self.event_buckets.remove(&event_type);
        }
        true
    }

    fn remove_entity_event_observer(
        &mut self,
        key: EntityEventObserverKey,
        id: ObserverId,
    ) -> bool {
        let Some(bucket) = self.entity_event_buckets.get_mut(&key) else {
            return false;
        };
        if !remove_observer_from_indexed_bucket(bucket, id) {
            return false;
        }
        if bucket.is_empty() {
            self.entity_event_buckets.remove(&key);
            self.remove_entity_event_type(key);
        }
        true
    }

    fn remove_entity_event_type(&mut self, key: EntityEventObserverKey) {
        let entity = key.1;
        let Some(event_types) = self.entity_event_types_by_entity.get_mut(&entity) else {
            return;
        };
        event_types.remove(&key.0);
        if event_types.is_empty() {
            self.entity_event_types_by_entity.remove(&entity);
        }
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
            .field("lifecycle_bucket_count", &self.lifecycle_buckets.len())
            .field("event_bucket_count", &self.event_buckets.len())
            .field(
                "entity_event_bucket_count",
                &self.entity_event_buckets.len(),
            )
            .field(
                "entity_event_target_count",
                &self.entity_event_types_by_entity.len(),
            )
            .field("observer_count", &self.observer_locations.len())
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
