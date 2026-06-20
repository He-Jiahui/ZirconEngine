use std::any::TypeId;

use crate::scene::ecs::{ComponentId, LifecycleEventKind};
use crate::scene::EntityId;

use super::entry::{EntityEventObserver, EventObserver, LifecycleObserver};
use super::ObserverId;

pub(super) fn lifecycle_callback_count(
    observers: &[LifecycleObserver],
    kind: LifecycleEventKind,
    component_id: ComponentId,
) -> usize {
    let mut count = 0_usize;
    for observer in observers {
        if observer.kind == kind && observer.component_id == component_id {
            count += 1;
        }
    }
    count
}

pub(super) fn event_callback_count(observers: &[EventObserver], event_type: TypeId) -> usize {
    let mut count = 0_usize;
    for observer in observers {
        if observer.event_type == event_type {
            count += 1;
        }
    }
    count
}

pub(super) fn entity_event_callback_count(
    observers: &[EntityEventObserver],
    event_type: TypeId,
    entity: EntityId,
) -> usize {
    let mut count = 0_usize;
    for observer in observers {
        if observer.event_type == event_type && observer.entity == entity {
            count += 1;
        }
    }
    count
}

pub(super) fn remove_observer_by_id<T>(
    observers: &mut Vec<T>,
    id: ObserverId,
    observer_id: impl Fn(&T) -> ObserverId,
) -> bool {
    let mut index = 0_usize;
    while index < observers.len() {
        if observer_id(&observers[index]) == id {
            observers.remove(index);
            return true;
        }
        index += 1;
    }
    false
}
