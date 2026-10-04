use crate::scene::ecs::{
    ChangeTick, Component, ComponentMutationRecorder, ComponentTicks, RemovedComponentEvents,
    RemovedComponentRetention, RemovedComponentRetentionMetrics, Resource, StorageType,
};
use crate::scene::{EntityId, World};

impl World {
    pub fn read_change_tick(&self) -> ChangeTick {
        self.change_tick
    }

    pub fn last_change_tick(&self) -> ChangeTick {
        self.last_change_tick
    }

    /// 结束一次观察窗口并推进移除事件保留期；调度器应在本帧消费者读完事件后调用。
    pub fn clear_trackers(&mut self) {
        self.advance_removed_component_events();
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
        // The ordinary entry owns every World field exclusively.
        unsafe { Self::query_mutation_change_tick(self) }
    }

    /// Advances only the clock leaf while disjoint query rows remain borrowed.
    ///
    /// # Safety
    /// The caller must hold the World query loan, serialize clock mutation, and keep
    /// live items disjoint from both clock fields. No shared clock/World reference may
    /// overlap this write; returned query items must not retain either metadata field.
    pub(super) unsafe fn query_mutation_change_tick(world: *mut Self) -> ChangeTick {
        unsafe {
            if let Some(tick) = std::ptr::addr_of!((*world).active_change_tick).read() {
                return tick;
            }
            let clock = &mut *std::ptr::addr_of_mut!((*world).change_tick);
            *clock = clock.next();
            *clock
        }
    }

    pub fn component_change_ticks<T>(&self, entity: EntityId) -> Option<ComponentTicks>
    where
        T: Component,
    {
        let component_id = self.registered_component_id::<T>()?;
        let internal = self.internal_entity(entity)?;
        match T::STORAGE_TYPE {
            StorageType::Table => {
                let location = self
                    .entity_registry
                    .location_for_internal(internal)
                    .ok()?
                    .location;
                self.archetype_index.component_ticks(
                    location.archetype_id,
                    location.table_row,
                    component_id,
                )
            }
            StorageType::SparseSet => self.component_storage.ticks(component_id, internal),
        }
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
        match T::STORAGE_TYPE {
            StorageType::Table => {
                let Ok(stable_location) = self.entity_registry.location_for_internal(internal)
                else {
                    return;
                };
                let location = stable_location.location;
                let _ = self.archetype_index.get_mut_at_tick::<T>(
                    location.archetype_id,
                    location.table_row,
                    component_id,
                    tick,
                );
            }
            StorageType::SparseSet => {
                self.component_storage
                    .mark_changed(component_id, internal, tick);
            }
        }
    }

    pub fn resource_change_ticks<T>(&self) -> Option<ComponentTicks>
    where
        T: Resource,
    {
        self.resources.ticks::<T>()
    }

    pub(crate) fn component_mut_with_ticks<T>(
        &mut self,
        entity: EntityId,
    ) -> Option<(
        &mut T,
        &mut ComponentTicks,
        ChangeTick,
        ComponentMutationRecorder<'_>,
    )>
    where
        T: Component,
    {
        if Self::protected_derived_component_name::<T>().is_some()
            || Self::protected_authored_component_name::<T>().is_some()
        {
            return None;
        }
        self.component_mut_with_ticks_unrestricted(entity)
    }

    #[cfg(test)]
    pub(super) fn component_mut_with_ticks_prevalidated_authored<T>(
        &mut self,
        entity: EntityId,
    ) -> Option<(
        &mut T,
        &mut ComponentTicks,
        ChangeTick,
        ComponentMutationRecorder<'_>,
    )>
    where
        T: Component,
    {
        debug_assert!(Self::protected_authored_component_name::<T>().is_some());
        self.component_mut_with_ticks_unrestricted(entity)
    }

    fn component_mut_with_ticks_unrestricted<T>(
        &mut self,
        entity: EntityId,
    ) -> Option<(
        &mut T,
        &mut ComponentTicks,
        ChangeTick,
        ComponentMutationRecorder<'_>,
    )>
    where
        T: Component,
    {
        let component_id = self.registered_component_id::<T>()?;
        let internal = self.internal_entity(entity)?;
        let tick = self.mutation_change_tick();
        let mutation_recorder = self
            .derived_state_dirty
            .component_mutation_recorder::<T>(entity);
        let (value, ticks) = match T::STORAGE_TYPE {
            StorageType::Table => {
                let location = self
                    .entity_registry
                    .location_for_internal(internal)
                    .ok()?
                    .location;
                self.archetype_index.get_mut_with_ticks(
                    location.archetype_id,
                    location.table_row,
                    component_id,
                )?
            }
            StorageType::SparseSet => self
                .component_storage
                .get_mut_with_ticks(component_id, internal)?,
        };
        Some((value, ticks, tick, mutation_recorder))
    }

    pub(crate) fn resource_mut_with_ticks<T>(
        &mut self,
    ) -> Option<(&mut T, &mut ComponentTicks, ChangeTick)>
    where
        T: Resource,
    {
        let tick = self.mutation_change_tick();
        let (value, ticks) = self.resources.get_mut_with_ticks::<T>()?;
        Some((value, ticks, tick))
    }

    pub(crate) fn record_removed_component<T>(&mut self, entity: EntityId)
    where
        T: Component,
    {
        self.removed_component_events.push::<T>(entity);
    }

    pub fn configure_removed_component_retention<T>(&mut self, retention: RemovedComponentRetention)
    where
        T: Component,
    {
        self.removed_component_events
            .configure_retention::<T>(retention);
    }

    pub fn removed_component_retention<T>(&self) -> Option<RemovedComponentRetention>
    where
        T: Component,
    {
        self.removed_component_events.retention::<T>()
    }

    pub fn removed_component_retention_metrics<T>(&self) -> Option<RemovedComponentRetentionMetrics>
    where
        T: Component,
    {
        self.removed_component_events.retention_metrics::<T>()
    }

    pub fn clear_removed_component_events<T>(&mut self)
    where
        T: Component,
    {
        self.removed_component_events.clear::<T>();
        if super::render_component_changes::is_render_component_change_source::<T>() {
            self.derived_state_dirty.mark_render_dirty();
        }
    }

    pub fn last_removed_component_advance_channel_visits(&self) -> usize {
        self.removed_component_events.last_advance_channel_visits()
    }

    pub(crate) fn advance_removed_component_events(&mut self) {
        self.removed_component_events.advance_frame();
    }

    pub(crate) fn removed_component_events(&self) -> &RemovedComponentEvents {
        &self.removed_component_events
    }
}
