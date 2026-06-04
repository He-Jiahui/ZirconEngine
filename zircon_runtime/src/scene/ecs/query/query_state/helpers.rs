use std::mem::MaybeUninit;

use crate::scene::ecs::{QueryEntityError, QueryEntityItem};
use crate::scene::EntityId;

use super::super::cached_query_iter::cached_query_entity_index;

pub(super) fn cached_many_entities<EntityList>(
    cached_entity_indices: &[(EntityId, usize)],
    entities: EntityList,
) -> Vec<EntityId>
where
    EntityList: IntoIterator,
    EntityList::Item: QueryEntityItem,
{
    entities
        .into_iter()
        .map(QueryEntityItem::entity_id)
        .filter(|entity| cached_query_entity_index(cached_entity_indices, *entity).is_some())
        .collect()
}

pub(super) fn collect_many_query_items<Item, const N: usize>(
    entities: [EntityId; N],
    mut get_item: impl FnMut(EntityId) -> Result<Item, QueryEntityError>,
) -> Result<[Item; N], QueryEntityError> {
    let mut values: [MaybeUninit<Item>; N] = std::array::from_fn(|_| MaybeUninit::uninit());
    let mut initialized = 0;

    for (slot, entity) in values.iter_mut().zip(entities) {
        match get_item(entity) {
            Ok(item) => {
                slot.write(item);
                initialized += 1;
            }
            Err(error) => {
                for value in &mut values[..initialized] {
                    // Only slots written before the error contain initialized values.
                    unsafe {
                        value.assume_init_drop();
                    }
                }
                return Err(error);
            }
        }
    }

    Ok(values.map(|value| {
        // Every slot was written by the loop above.
        unsafe { value.assume_init() }
    }))
}
