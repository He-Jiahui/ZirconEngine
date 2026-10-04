use std::marker::PhantomData;

use super::query_state::{project_entity_from_plans, CachedArchetypePlan};
use crate::scene::ecs::{
    ChangeTickWindow, ComponentStorageLocation, QueryEntityItem, QueryFilter, QueryMutData,
};
use crate::scene::{EntityId, World};

/// 对 `UniqueEntityArray` 或等价唯一输入逐个返回可变查询项。
///
/// 输入必须唯一；返回项的借用覆盖整个 world 借用，可跨连续 next 调用保留。
pub struct QueryManyUniqueMutIter<'world, 'state, D, F = (), I = std::vec::IntoIter<EntityId>>
where
    D: QueryMutData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    world: *mut World,
    plans: &'state [CachedArchetypePlan],
    component_locations: Vec<ComponentStorageLocation>,
    entities: I,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world mut World, fn() -> (D, F))>,
}

impl<'world, 'state, D, F, I> QueryManyUniqueMutIter<'world, 'state, D, F, I>
where
    D: QueryMutData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    pub(crate) unsafe fn new<EntityList>(
        world: *mut World,
        plans: &'state [CachedArchetypePlan],
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> Self
    where
        EntityList: IntoIterator<IntoIter = I>,
        EntityList::Item: QueryEntityItem,
    {
        unsafe {
            Self {
                world,
                plans,
                component_locations: Vec::new(),
                entities: entities.into_iter(),
                ticks,
                _marker: PhantomData,
            }
        }
    }
}

impl<'world, 'state, D, F, I> Iterator for QueryManyUniqueMutIter<'world, 'state, D, F, I>
where
    D: QueryMutData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entity_item) = self.entities.next() {
            let entity = entity_item.entity_id();
            let matches = {
                // SAFETY: only current projection and contracted candidate-local
                // matching use the shared reference, before the raw leaf fetch.
                let world = unsafe { &*self.world };
                project_entity_from_plans(self.plans, world, entity, &mut self.component_locations)
                    .is_some()
                    && F::matches_component_locations(
                        world,
                        entity,
                        &self.component_locations,
                        self.ticks,
                    )
            };
            if !matches {
                continue;
            }
            // Unique input and the unsafe data/filter contracts require
            // compatible items across distinct yielded entities.
            return unsafe {
                D::fetch_mut_with_component_locations(
                    self.world,
                    entity,
                    &self.component_locations,
                    self.ticks,
                )
            };
        }
        None
    }
}
