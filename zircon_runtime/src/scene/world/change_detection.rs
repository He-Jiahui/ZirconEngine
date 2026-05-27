use crate::scene::ecs::{ChangeTick, Component, ComponentTicks, RemovedComponentEvents, Resource};
use crate::scene::{EntityId, World};

impl World {
    pub fn read_change_tick(&self) -> ChangeTick {
        self.change_tick
    }

    pub fn last_change_tick(&self) -> ChangeTick {
        self.last_change_tick
    }

    pub fn clear_trackers(&mut self) {
        self.last_change_tick = self.change_tick;
    }

    pub(crate) fn advance_change_tick(&mut self) -> ChangeTick {
        self.change_tick = self.change_tick.next();
        self.change_tick
    }

    pub(crate) fn replace_active_change_tick(
        &mut self,
        tick: Option<ChangeTick>,
    ) -> Option<ChangeTick> {
        std::mem::replace(&mut self.active_change_tick, tick)
    }

    pub(crate) fn mutation_change_tick(&mut self) -> ChangeTick {
        self.active_change_tick
            .unwrap_or_else(|| self.advance_change_tick())
    }

    pub fn component_change_ticks<T>(&self, entity: EntityId) -> Option<ComponentTicks>
    where
        T: Component,
    {
        let component_id = self.registered_component_id::<T>()?;
        let internal = self.internal_entity(entity)?;
        self.component_storage.ticks(component_id, internal)
    }

    pub(crate) fn mark_component_changed_at_tick<T>(&mut self, entity: EntityId, tick: ChangeTick)
    where
        T: Component,
    {
        let Some(component_id) = self.registered_component_id::<T>() else {
            return;
        };
        let Some(internal) = self.internal_entity(entity) else {
            return;
        };
        self.component_storage
            .mark_changed(component_id, internal, tick);
    }

    pub fn resource_change_ticks<T>(&self) -> Option<ComponentTicks>
    where
        T: Resource,
    {
        self.resources.ticks::<T>()
    }

    pub(crate) fn resource_mut_with_ticks<T>(&mut self) -> Option<(&mut T, ComponentTicks)>
    where
        T: Resource,
    {
        let tick = self.mutation_change_tick();
        self.resources.get_mut_at_tick_with_ticks::<T>(tick)
    }

    pub(crate) fn record_removed_component<T>(&mut self, entity: EntityId)
    where
        T: Component,
    {
        self.removed_component_events.push::<T>(entity);
    }

    pub(crate) fn removed_component_events(&self) -> &RemovedComponentEvents {
        &self.removed_component_events
    }
}
