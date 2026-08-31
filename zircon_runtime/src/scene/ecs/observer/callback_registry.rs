use std::sync::Arc;

use super::ObserverId;

#[cfg(test)]
#[path = "callback_registry/vector_bucket_tests.rs"]
mod vector_bucket_tests;

pub(super) trait IndexedObserver {
    fn observer_id(&self) -> ObserverId;
}

pub(super) fn insert_observer_into_bucket<T>(bucket: &mut Arc<Vec<T>>, id: ObserverId, observer: T)
where
    T: Clone + IndexedObserver,
{
    let observers = Arc::make_mut(bucket);
    debug_assert!(
        observers
            .last()
            .is_none_or(|registered| registered.observer_id() < id),
        "observer ids must be unique and monotonically allocated"
    );
    observers.push(observer);
}

pub(super) fn remove_observer_from_indexed_bucket<T>(
    bucket: &mut Arc<Vec<T>>,
    id: ObserverId,
) -> bool
where
    T: Clone + IndexedObserver,
{
    let observers = Arc::make_mut(bucket);
    let Some(index) = observers
        .iter()
        .position(|observer| observer.observer_id() == id)
    else {
        return false;
    };
    observers.remove(index);
    true
}
