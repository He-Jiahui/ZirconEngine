use std::mem::MaybeUninit;

use crate::scene::ecs::QueryEntityError;
use crate::scene::EntityId;

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
