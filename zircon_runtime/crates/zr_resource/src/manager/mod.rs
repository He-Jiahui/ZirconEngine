//! 在线资源权威的操作入口；目录发布经批次提交，租约另行管理载荷驻留，两者均在权威锁内刷新派生投影。
//! 调用端通过快照读取一致状态，并用事件跟踪目录变化；载荷驻留变化由就绪视图表达。

mod commit;
mod lazy_registration;
mod lease_ops;
mod management_projection;
mod payload_ops;
mod readiness_projection;
mod registry_export;
mod registry_ops;
mod resource_manager;
mod revision;
mod runtime_slot;

#[cfg(test)]
mod tests;

pub use commit::PreparedResourceMutation;
pub use resource_manager::{
    ResourceManager, ResourceProjectionSnapshot, ResourceRegistryReadGuard,
};
