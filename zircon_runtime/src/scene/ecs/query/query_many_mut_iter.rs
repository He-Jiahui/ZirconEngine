use std::marker::PhantomData;

use super::query_state::{project_entity_from_plans, CachedArchetypePlan};
use crate::scene::ecs::{
    ChangeTickWindow, ComponentStorageLocation, QueryEntityItem, QueryFilter, QueryMutData,
};
use crate::scene::{EntityId, World};

/// 使用已编译列位置逐个返回可变查询项。
///
/// 每次返回项只借用本次 fetch_next 的可变接收者；释放该项后可以继续处理重复实体。
/// 需要同时保留多个返回项时，使用要求唯一实体的迭代入口。
pub struct QueryManyMutIter<'world, 'state, D, F = (), I = std::vec::IntoIter<EntityId>>
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

impl<'world, 'state, D, F, I> QueryManyMutIter<'world, 'state, D, F, I>
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

    pub fn fetch_next(&mut self) -> Option<D::Item<'_>> {
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
