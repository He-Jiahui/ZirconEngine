use std::any::{TypeId, type_name};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;

use crate::scene::ecs::{ChangeTick, ComponentTicks};

use super::stored_resource::StoredResource;

#[derive(Default)]
pub struct ResourceStore {
    resources: HashMap<TypeId, StoredResource>,
}

impl ResourceStore {
    pub fn insert<T: 'static + Send + Sync>(&mut self, resource: T) -> Option<T> {
        self.insert_at_tick(resource, ChangeTick::INITIAL)
    }

    pub fn insert_at_tick<T: 'static + Send + Sync>(
        &mut self,
        resource: T,
        tick: ChangeTick,
    ) -> Option<T> {
        let type_id = TypeId::of::<T>();
        match self.resources.entry(type_id) {
            Entry::Occupied(mut occupied) => {
                let stored = occupied.get_mut();
                stored.ticks.set_changed(tick);
                let previous = std::mem::replace(&mut stored.value, Box::new(resource));
                let Ok(boxed) = previous.downcast::<T>() else {
                    return None;
                };
                Some(*boxed)
            }
            Entry::Vacant(vacant) => {
                vacant.insert(StoredResource {
                    value: Box::new(resource),
                    type_name: type_name::<T>(),
                    ticks: ComponentTicks::new(tick),
                });
                None
            }
        }
    }

    pub fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        let stored = self.resources.get(&TypeId::of::<T>())?;
        stored.value.downcast_ref::<T>()
    }

    pub fn get_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        let stored = self.resources.get_mut(&TypeId::of::<T>())?;
        stored.value.downcast_mut::<T>()
    }

    pub fn get_mut_at_tick_with_ticks<T: 'static + Send + Sync>(
        &mut self,
        tick: ChangeTick,
    ) -> Option<(&mut T, ComponentTicks)> {
        let stored = self.resources.get_mut(&TypeId::of::<T>())?;
        stored.ticks.set_changed(tick);
        let ticks = stored.ticks;
        let Some(value) = stored.value.downcast_mut::<T>() else {
            return None;
        };
        Some((value, ticks))
    }

    pub fn remove<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        let stored = self.resources.remove(&TypeId::of::<T>())?;
        let Ok(boxed) = stored.value.downcast::<T>() else {
            return None;
        };
        Some(*boxed)
    }

    pub fn contains<T: 'static + Send + Sync>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    pub fn ticks<T: 'static + Send + Sync>(&self) -> Option<ComponentTicks> {
        let stored = self.resources.get(&TypeId::of::<T>())?;
        Some(stored.ticks)
    }

    pub fn len(&self) -> usize {
        self.resources.len()
    }

    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    pub fn clear(&mut self) {
        self.resources.clear();
    }

    pub fn type_names(&self) -> Vec<&'static str> {
        let mut names = Vec::with_capacity(self.resources.len());
        for stored in self.resources.values() {
            names.push(stored.type_name);
        }
        names.sort_unstable();
        names
    }
}

impl fmt::Debug for ResourceStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ResourceStore")
            .field("type_names", &self.type_names())
            .finish()
    }
}

impl Clone for ResourceStore {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for ResourceStore {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
