use crate::scene::ecs::InternalEntity;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComponentRemoveResult<T> {
    pub value: T,
    pub swapped_entity: Option<InternalEntity>,
}
