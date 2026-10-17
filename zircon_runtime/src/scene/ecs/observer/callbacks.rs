use std::any::Any;
use std::sync::Arc;

use crate::scene::ecs::ComponentLifecycleEvent;
use crate::scene::{EntityId, World};

pub(crate) type LifecycleCallback =
    Arc<dyn crate::core::framework::retained_memory::LifecycleEffectOwnerV1>;
pub(crate) type EventCallback = Arc<dyn Fn(&mut World, &dyn Any) + Send + Sync>;
pub(crate) type EntityEventCallback =
    Arc<dyn crate::core::framework::retained_memory::EntityObserverOwnerV1>;
