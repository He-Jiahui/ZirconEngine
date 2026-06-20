use std::any::Any;
use std::sync::Arc;

use crate::scene::ecs::ComponentLifecycleEvent;
use crate::scene::{EntityId, World};

pub(crate) type LifecycleCallback = Arc<dyn Fn(&mut World, ComponentLifecycleEvent) + Send + Sync>;
pub(crate) type EventCallback = Arc<dyn Fn(&mut World, &dyn Any) + Send + Sync>;
pub(crate) type EntityEventCallback = Arc<dyn Fn(&mut World, EntityId, &dyn Any) + Send + Sync>;
