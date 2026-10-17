//! 保留式界面树的数据、基础修改入口和序列化契约。
//! 布局执行、命中路由与表面发布由 Runtime 的对应扩展承担。

#[path = "node/mod.rs"]
mod node;

pub use node::{
    UiCursor, UiDirtyFlags, UiInputPolicy, UiLayoutCache, UiPointerEvents, UiTemplateNodeMetadata,
    UiTree, UiTreeError, UiTreeNode, UiTreeNodes, UiVisibility,
};
