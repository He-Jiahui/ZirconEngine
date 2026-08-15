use std::collections::BTreeMap;
use std::sync::Arc;

use super::ObserverId;

pub(super) fn insert_observer_into_bucket<T>(
    bucket: &mut Arc<BTreeMap<ObserverId, T>>,
    id: ObserverId,
    observer: T,
) where
    T: Clone,
{
    let replaced = Arc::make_mut(bucket).insert(id, observer);
    debug_assert!(replaced.is_none(), "observer ids must be unique");
}

pub(super) fn remove_observer_from_indexed_bucket<T>(
    bucket: &mut Arc<BTreeMap<ObserverId, T>>,
    id: ObserverId,
) -> bool
where
    T: Clone,
{
    Arc::make_mut(bucket).remove(&id).is_some()
}
