use crate::scene::ecs::{Component, StorageType};

#[derive(Debug)]
pub(super) struct DynamicComponentPresence;

impl Component for DynamicComponentPresence {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}
