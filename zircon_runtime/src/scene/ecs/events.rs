use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Events<T> {
    current: Vec<T>,
    next: Vec<T>,
}

impl<T> Default for Events<T> {
    fn default() -> Self {
        Self {
            current: Vec::new(),
            next: Vec::new(),
        }
    }
}

impl<T> Events<T> {
    pub fn send(&mut self, event: T) {
        self.next.push(event);
    }

    pub fn update(&mut self) {
        self.current.clear();
        std::mem::swap(&mut self.current, &mut self.next);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.current.iter()
    }

    pub fn drain(&mut self) -> Vec<T> {
        self.current.drain(..).collect()
    }

    pub fn clear(&mut self) {
        self.current.clear();
        self.next.clear();
    }

    pub fn len(&self) -> usize {
        self.current.len()
    }

    pub fn is_empty(&self) -> bool {
        self.current.is_empty()
    }
}

#[derive(Default)]
pub struct EventStore {
    stores: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    type_names: HashMap<TypeId, &'static str>,
}

impl EventStore {
    pub fn events<T: 'static + Send + Sync>(&self) -> Option<&Events<T>> {
        self.stores
            .get(&TypeId::of::<T>())
            .and_then(|store| store.downcast_ref::<Events<T>>())
    }

    pub fn events_mut<T: 'static + Send + Sync>(&mut self) -> &mut Events<T> {
        let type_id = TypeId::of::<T>();
        self.type_names.entry(type_id).or_insert(type_name::<T>());
        self.stores
            .entry(type_id)
            .or_insert_with(|| Box::<Events<T>>::default())
            .downcast_mut::<Events<T>>()
            .expect("event store type id must match event queue type")
    }

    pub fn send<T: 'static + Send + Sync>(&mut self, event: T) {
        self.events_mut::<T>().send(event);
    }

    pub fn update<T: 'static + Send + Sync>(&mut self) {
        self.events_mut::<T>().update();
    }

    pub fn drain<T: 'static + Send + Sync>(&mut self) -> Vec<T> {
        self.events_mut::<T>().drain()
    }

    pub fn registered_type_names(&self) -> Vec<&'static str> {
        let mut names = self.type_names.values().copied().collect::<Vec<_>>();
        names.sort_unstable();
        names
    }
}

impl fmt::Debug for EventStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventStore")
            .field("registered_type_names", &self.registered_type_names())
            .finish()
    }
}
