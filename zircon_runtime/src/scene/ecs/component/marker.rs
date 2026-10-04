use crate::scene::ecs::StorageType;

/// 可进入 World 的静态 Rust 组件；存储类型由类型声明决定，并参与原型签名与查询访问规划。
pub trait Component: 'static + Send + Sync {
    const STORAGE_TYPE: StorageType = StorageType::Table;
}
