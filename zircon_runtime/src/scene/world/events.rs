use crate::scene::ecs::{EventStore, Events};

use super::World;

impl World {
    pub fn send_event<T>(&mut self, event: T)
    where
        T: 'static + Send + Sync,
    {
        self.events.send(event);
    }

    pub fn update_events<T>(&mut self)
    where
        T: 'static + Send + Sync,
    {
        self.events.update::<T>();
    }

    pub fn clear_events<T>(&mut self)
    where
        T: 'static + Send + Sync,
    {
        self.events.events_mut::<T>().clear();
    }

    pub fn events<T>(&self) -> Option<&Events<T>>
    where
        T: 'static + Send + Sync,
    {
        self.events.events::<T>()
    }

    pub(crate) fn event_store_mut(&mut self) -> &mut EventStore {
        &mut self.events
    }
}
