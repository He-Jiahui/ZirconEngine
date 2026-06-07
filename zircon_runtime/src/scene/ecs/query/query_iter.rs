use std::marker::PhantomData;

use crate::scene::ecs::{
    ChangeTickWindow, ComponentStorageLocation, QueryData, QueryFilter, StableEntityLocation,
};
use crate::scene::{EntityId, World};

use super::cached_query_iter::cached_query_component_locations;

pub struct QueryIter<'world, 'entities, D, F = ()>
where
    D: QueryData,
    F: QueryFilter,
{
    world: &'world World,
    entities: &'entities [EntityId],
    locations: Option<&'entities [StableEntityLocation]>,
    component_locations: Option<&'entities [ComponentStorageLocation]>,
    component_location_offsets: Option<&'entities [usize]>,
    index: usize,
    ticks: ChangeTickWindow,
    _marker: PhantomData<fn() -> (D, F)>,
}

impl<'world, 'entities, D, F> QueryIter<'world, 'entities, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    pub(crate) fn new(
        world: &'world World,
        entities: &'entities [EntityId],
        ticks: ChangeTickWindow,
    ) -> Self {
        Self {
            world,
            entities,
            locations: None,
            component_locations: None,
            component_location_offsets: None,
            index: 0,
            ticks,
            _marker: PhantomData,
        }
    }

    pub(crate) fn new_cached_locations(
        world: &'world World,
        entities: &'entities [EntityId],
        locations: &'entities [StableEntityLocation],
        component_locations: &'entities [ComponentStorageLocation],
        component_location_offsets: &'entities [usize],
        ticks: ChangeTickWindow,
    ) -> Self {
        Self {
            world,
            entities,
            locations: Some(locations),
            component_locations: Some(component_locations),
            component_location_offsets: Some(component_location_offsets),
            index: 0,
            ticks,
            _marker: PhantomData,
        }
    }

    #[cfg(test)]
    pub(crate) fn uses_cached_component_locations(&self) -> bool {
        self.locations.is_some()
            && self.component_locations.is_some()
            && self.component_location_offsets.is_some()
    }
}

impl<'world, 'entities, D, F> Iterator for QueryIter<'world, 'entities, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entity) = self.entities.get(self.index).copied() {
            let index = self.index;
            self.index += 1;
            if let (Some(locations), Some(component_locations), Some(component_location_offsets)) = (
                self.locations,
                self.component_locations,
                self.component_location_offsets,
            ) {
                let stable_location = locations.get(index).copied()?;
                let component_locations = cached_query_component_locations(
                    component_locations,
                    component_location_offsets,
                    index,
                )?;
                if F::matches_component_locations(
                    self.world,
                    entity,
                    component_locations,
                    self.ticks,
                ) {
                    if let Some(item) = D::fetch_with_component_locations(
                        self.world,
                        entity,
                        stable_location,
                        component_locations,
                        self.ticks,
                    ) {
                        return Some(item);
                    }
                }
                continue;
            }
            if F::matches(self.world, entity, self.ticks) && D::matches_data(self.world, entity) {
                if let Some(item) = D::fetch_with_ticks(self.world, entity, self.ticks) {
                    return Some(item);
                }
            }
        }
        None
    }
}
