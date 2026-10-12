//! 在线资源权威的操作入口；目录发布经批次提交，租约另行管理载荷驻留，两者均在权威锁内刷新派生投影。
//! 调用端通过快照读取一致状态，并用事件跟踪目录变化；载荷驻留变化由就绪视图表达。

#[path = "commit.rs"]
mod commit;
#[path = "lazy_registration.rs"]
mod lazy_registration;
#[path = "lease_ops.rs"]
mod lease_ops;
#[path = "management_projection.rs"]
mod management_projection;
#[path = "payload_ops.rs"]
mod payload_ops;
#[path = "readiness_projection.rs"]
mod readiness_projection;
#[path = "registry_export.rs"]
mod registry_export;
#[path = "registry_ops.rs"]
mod registry_ops;
#[path = "resource_manager.rs"]
mod resource_manager;
#[path = "revision.rs"]
mod revision;
#[path = "runtime_slot.rs"]
mod runtime_slot;

#[cfg(test)]
#[path = "tests/mod.rs"]
mod tests;

pub use commit::PreparedResourceMutation;
pub use resource_manager::{
    ResourceManager, ResourceProjectionSnapshot, ResourceRegistryReadGuard,
};
