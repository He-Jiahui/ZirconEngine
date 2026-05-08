use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt;

use crate::scene::ecs::{ChangeTick, ComponentTicks};

struct StoredResource {
    value: Box<dyn Any + Send + Sync>,
    type_name: &'static str,
    ticks: ComponentTicks,
}

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
        let ticks = self
            .resources
            .get(&type_id)
            .map(|stored| {
                let mut ticks = stored.ticks;
                ticks.set_changed(tick);
                ticks
            })
            .unwrap_or_else(|| ComponentTicks::new(tick));
        self.resources
            .insert(
                type_id,
                StoredResource {
                    value: Box::new(resource),
                    type_name: type_name::<T>(),
                    ticks,
                },
            )
            .and_then(|stored| stored.value.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    pub fn get<T: 'static + Send + Sync>(&self) -> Option<&T> {
        self.resources
            .get(&TypeId::of::<T>())
            .and_then(|stored| stored.value.downcast_ref::<T>())
    }

    pub fn get_mut<T: 'static + Send + Sync>(&mut self) -> Option<&mut T> {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .and_then(|stored| stored.value.downcast_mut::<T>())
    }

    pub fn get_mut_at_tick_with_ticks<T: 'static + Send + Sync>(
        &mut self,
        tick: ChangeTick,
    ) -> Option<(&mut T, ComponentTicks)> {
        let stored = self.resources.get_mut(&TypeId::of::<T>())?;
        stored.ticks.set_changed(tick);
        let ticks = stored.ticks;
        stored.value.downcast_mut::<T>().map(|value| (value, ticks))
    }

    pub fn remove<T: 'static + Send + Sync>(&mut self) -> Option<T> {
        self.resources
            .remove(&TypeId::of::<T>())
            .and_then(|stored| stored.value.downcast::<T>().ok())
            .map(|boxed| *boxed)
    }

    pub fn contains<T: 'static + Send + Sync>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    pub fn ticks<T: 'static + Send + Sync>(&self) -> Option<ComponentTicks> {
        self.resources
            .get(&TypeId::of::<T>())
            .map(|stored| stored.ticks)
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
        let mut names = self
            .resources
            .values()
            .map(|stored| stored.type_name)
            .collect::<Vec<_>>();
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
