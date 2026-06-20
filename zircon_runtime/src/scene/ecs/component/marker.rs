use crate::scene::ecs::StorageType;

pub trait Component: 'static + Send + Sync {
    const STORAGE_TYPE: StorageType = StorageType::Table;
}
