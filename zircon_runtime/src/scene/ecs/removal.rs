use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::scene::EntityId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemovedComponentEvent {
    entity: EntityId,
}

impl RemovedComponentEvent {
    pub const fn new(entity: EntityId) -> Self {
        Self { entity }
    }

    pub const fn entity(self) -> EntityId {
        self.entity
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RemovedComponentEvents {
    events: HashMap<TypeId, Vec<RemovedComponentEvent>>,
    type_names: HashMap<TypeId, String>,
}

impl RemovedComponentEvents {
    pub fn push<T>(&mut self, entity: EntityId)
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.type_names
            .entry(type_id)
            .or_insert_with(|| type_name::<T>().to_string());
        self.events
            .entry(type_id)
            .or_default()
            .push(RemovedComponentEvent::new(entity));
    }

    pub(crate) fn push_type_id(
        &mut self,
        type_id: TypeId,
        type_name: impl Into<String>,
        entity: EntityId,
    ) {
        self.type_names
            .entry(type_id)
            .or_insert_with(|| type_name.into());
        self.events
            .entry(type_id)
            .or_default()
            .push(RemovedComponentEvent::new(entity));
    }

    pub fn events<T>(&self) -> &[RemovedComponentEvent]
    where
        T: 'static,
    {
        self.events
            .get(&TypeId::of::<T>())
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn registered_type_names(&self) -> Vec<&str> {
        let mut names = self
            .type_names
            .values()
            .map(String::as_str)
            .collect::<Vec<_>>();
        names.sort_unstable();
        names
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemovedComponentReader<T> {
    cursor: usize,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Default for RemovedComponentReader<T> {
    fn default() -> Self {
        Self {
            cursor: 0,
            _marker: PhantomData,
        }
    }
}

impl<T> RemovedComponentReader<T> {
    pub fn read(&mut self, events: &RemovedComponentEvents) -> Vec<EntityId>
    where
        T: 'static,
    {
        let all = events.events::<T>();
        let start = self.cursor.min(all.len());
        self.cursor = all.len();
        all[start..].iter().map(|event| event.entity()).collect()
    }

    pub fn len(&self, events: &RemovedComponentEvents) -> usize
    where
        T: 'static,
    {
        events.events::<T>().len().saturating_sub(self.cursor)
    }

    pub fn is_empty(&self, events: &RemovedComponentEvents) -> bool
    where
        T: 'static,
    {
        self.len(events) == 0
    }

    pub fn clear(&mut self, events: &RemovedComponentEvents)
    where
        T: 'static,
    {
        self.cursor = events.events::<T>().len();
    }
}
