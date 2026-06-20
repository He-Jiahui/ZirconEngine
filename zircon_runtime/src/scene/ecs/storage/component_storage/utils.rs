use crate::scene::ecs::ComponentId;

use super::super::StorageError;
use super::entry::StoredComponent;

pub(in crate::scene::ecs::storage) fn sort_component_ids_if_needed(
    component_ids: &mut [ComponentId],
) {
    if component_ids.len() > 1 {
        component_ids.sort_unstable();
    }
}

pub(in crate::scene::ecs::storage) fn downcast_component<T>(
    component_id: ComponentId,
    value: StoredComponent,
) -> Result<T, StorageError>
where
    T: 'static + Send + Sync,
{
    match value.downcast::<T>() {
        Ok(value) => Ok(*value),
        Err(_) => Err(StorageError::ComponentTypeMismatch { component_id }),
    }
}
