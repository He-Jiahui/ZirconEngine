//! 为类型化错误审查挂载专属检查与共享清单；此入口只划分职责，实际断言由子模块及测试框架承担。
#[path = "metadata/child_inventory_paths.rs"]
mod child_inventory_paths;
#[path = "metadata/delegation_paths.rs"]
mod delegation_paths;
#[path = "metadata/review_guard_paths.rs"]
mod review_guard_paths;
#[path = "metadata/root_paths.rs"]
mod root_paths;

pub(super) use child_inventory_paths::*;
pub(super) use delegation_paths::*;
pub(super) use review_guard_paths::*;
pub(super) use root_paths::*;
