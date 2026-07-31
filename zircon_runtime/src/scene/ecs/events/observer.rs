use std::any::Any;
use std::marker::PhantomData;

use super::{Event, EventTypeId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct EventObserverId(u64);

impl EventObserverId {
    pub(crate) const fn new(raw: u64) -> Self {
        Self(raw)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Stable owner token for a send-boundary event observer registered in one [`EventStore`].
pub(crate) struct EventObserverHandle {
    event_type_id: EventTypeId,
    observer_id: EventObserverId,
}

impl EventObserverHandle {
    pub(crate) const fn new(event_type_id: EventTypeId, observer_id: EventObserverId) -> Self {
        Self {
            event_type_id,
            observer_id,
        }
    }

    pub(crate) const fn event_type_id(self) -> EventTypeId {
        self.event_type_id
    }

    pub(crate) const fn observer_id(self) -> EventObserverId {
        self.observer_id
    }
}

/// Type-erased synchronous observer. `false` rejects only that observer's delivery while the
/// typed ECS event still enters its ordinary queue, allowing the producer to surface pressure.
pub(super) trait ErasedEventObserver: Send + Sync {
    fn notify(&self, event: &dyn Any) -> bool;
}

pub(super) struct TypedEventObserver<T, F> {
    callback: F,
    _marker: PhantomData<fn() -> T>,
}

impl<T, F> TypedEventObserver<T, F> {
    pub(super) fn new(callback: F) -> Self {
        Self {
            callback,
            _marker: PhantomData,
        }
    }
}

impl<T, F> ErasedEventObserver for TypedEventObserver<T, F>
where
    T: Event,
    F: Fn(&T) -> bool + Send + Sync,
{
    fn notify(&self, event: &dyn Any) -> bool {
        event
            .downcast_ref::<T>()
            .is_some_and(|event| (self.callback)(event))
    }
}
