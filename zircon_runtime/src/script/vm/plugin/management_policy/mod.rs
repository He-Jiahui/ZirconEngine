//! 聚合 VM 包管理策略类型与校验错误；发现入口先验证组合策略，热重载和 GC 协调器随后读取对应子策略。

#[path = "error.rs"]
mod error;
#[path = "garbage_collection.rs"]
mod garbage_collection;
#[path = "hot_reload.rs"]
mod hot_reload;
#[path = "memory.rs"]
mod memory;
#[path = "policy.rs"]
mod policy;

pub use error::{VmPluginManagementPolicyError, VmPluginManagementPolicyResult};
pub use garbage_collection::{VmPluginGarbageCollectionMode, VmPluginGarbageCollectionPolicy};
pub use hot_reload::VmPluginHotReloadPolicy;
pub use memory::VmPluginMemoryPolicy;
pub use policy::VmPluginManagementPolicy;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
