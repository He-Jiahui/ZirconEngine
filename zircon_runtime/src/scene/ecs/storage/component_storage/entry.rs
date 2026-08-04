use std::any::Any;

use crate::scene::ecs::InternalEntity;

pub(crate) type StoredComponent = Box<dyn Any + Send + Sync>;

pub(in crate::scene::ecs::storage) struct RawRemoveResult {
    pub(super) value: StoredComponent,
    pub(super) swapped_entity: Option<InternalEntity>,
}
