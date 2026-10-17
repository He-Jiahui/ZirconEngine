use std::any::TypeId;
use std::sync::Arc;

use crate::scene::ecs::{ComponentId, ComponentLifecycleEvent, LifecycleEventKind};
use crate::scene::{EntityId, World};

use super::callback_registry::IndexedObserver;
use super::ObserverId;
use super::{EntityEventCallback, EventCallback, LifecycleCallback};

pub(super) type LifecycleObserverKey = (LifecycleEventKind, ComponentId);
pub(super) type EntityEventObserverKey = (TypeId, EntityId);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum ObserverBucket {
    Lifecycle(LifecycleObserverKey),
    Event(TypeId),
    EntityEvent(EntityEventObserverKey),
}

#[derive(Clone)]
pub(super) struct LifecycleObserver {
    pub(super) id: ObserverId,
    pub(super) callback: LifecycleCallback,
}

impl IndexedObserver for LifecycleObserver {
    fn observer_id(&self) -> ObserverId {
        self.id
    }
}

#[derive(Clone)]
pub(super) struct EventObserver {
    pub(super) id: ObserverId,
    pub(super) callback: EventCallback,
}

impl IndexedObserver for EventObserver {
    fn observer_id(&self) -> ObserverId {
        self.id
    }
}

#[derive(Clone)]
pub(super) struct EntityEventObserver {
    pub(super) id: ObserverId,
    pub(super) callback: EntityEventCallback,
}

impl IndexedObserver for EntityEventObserver {
    fn observer_id(&self) -> ObserverId {
        self.id
    }
}

pub(crate) struct LifecycleCallbackBucket {
    observers: Arc<Vec<LifecycleObserver>>,
}

impl LifecycleCallbackBucket {
    pub(super) fn new(observers: Arc<Vec<LifecycleObserver>>) -> Self {
        Self { observers }
    }

    pub(crate) fn dispatch(&self, world: &mut World, event: ComponentLifecycleEvent) {
        let owner = world.owner_identity().clone();
        let execution = world.begin_schedule_execution();
        let plan_owner = execution.plan();
        for observer in self.observers.iter() {
            if !owner.is_same_world(world.owner_identity())
                || !world.schedule().owns_stage_plan(plan_owner)
            {
                break;
            }
            observer.callback.dispatch(world, &event);
        }
    }
}

pub(crate) struct EventCallbackBucket {
    observers: Arc<Vec<EventObserver>>,
}

impl EventCallbackBucket {
    pub(super) fn new(observers: Arc<Vec<EventObserver>>) -> Self {
        Self { observers }
    }

    pub(crate) fn dispatch<E>(&self, world: &mut World, event: &E)
    where
        E: 'static,
    {
        let owner = world.owner_identity().clone();
        let execution = world.begin_schedule_execution();
        let plan_owner = execution.plan();
        for observer in self.observers.iter() {
            if !owner.is_same_world(world.owner_identity())
                || !world.schedule().owns_stage_plan(plan_owner)
            {
                break;
            }
            (observer.callback)(world, event);
        }
    }
}

pub(crate) struct EntityEventCallbackBucket {
    observers: Arc<Vec<EntityEventObserver>>,
}

impl EntityEventCallbackBucket {
    pub(super) fn new(observers: Arc<Vec<EntityEventObserver>>) -> Self {
        Self { observers }
    }

    pub(crate) fn dispatch<E>(&self, world: &mut World, entity: EntityId, event: &E)
    where
        E: 'static,
    {
        let owner = world.owner_identity().clone();
        let execution = world.begin_schedule_execution();
        let plan_owner = execution.plan();
        for observer in self.observers.iter() {
            if !owner.is_same_world(world.owner_identity())
                || !world.schedule().owns_stage_plan(plan_owner)
            {
                break;
            }
            observer.callback.dispatch(world, entity, event);
        }
    }
}

impl zircon_runtime_interface::retained_memory::RetainedOwnerV1 for EntityEventObserver {
    fn retained_bound_v1(
        &self,
        m: &mut zircon_runtime_interface::retained_memory::RetainedBoundBuilderV1,
    ) -> Result<(), zircon_runtime_interface::retained_memory::RetentionContractError> {
        m.arc_payload(self.callback.as_ref())
    }
}
unsafe impl zircon_runtime_interface::retained_memory::ImmutableRetainedOwnerV1
    for EntityEventObserver
{
}
