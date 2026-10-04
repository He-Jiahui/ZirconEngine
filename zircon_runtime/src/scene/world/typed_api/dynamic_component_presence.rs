use crate::scene::ecs::{Component, StorageType};

#[derive(Debug)]
// 动态组件的稀疏 ECS 存在性标记；真实字段仍由 World 的 JSON 组件表和类型目录管理。
pub(super) struct DynamicComponentPresence;

impl Component for DynamicComponentPresence {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;
}
