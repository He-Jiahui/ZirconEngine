use std::any::Any;

use crate::scene::ecs::ComponentTicks;

pub(super) struct StoredResource {
    pub(super) value: Box<dyn Any + Send + Sync>,
    pub(super) type_name: &'static str,
    pub(super) ticks: ComponentTicks,
}
