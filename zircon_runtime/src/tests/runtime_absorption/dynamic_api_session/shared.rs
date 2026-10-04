//! 动态会话的二进制接口、宿主请求和诊断路由需与共享契约同步。集中挂载下级测试；所有行为断言留在被挂载模块。
#[path = "shared/abi.rs"]
pub(super) mod abi;
#[path = "shared/behavior.rs"]
pub(super) mod behavior;
#[path = "shared/diagnostics.rs"]
pub(super) mod diagnostics;
#[path = "shared/docs.rs"]
pub(super) mod docs;
#[path = "shared/host_requests.rs"]
pub(super) mod host_requests;
#[path = "shared/slices.rs"]
pub(super) mod slices;
#[path = "shared/source_inventory.rs"]
pub(super) mod source_inventory;
#[path = "shared/split_layout.rs"]
mod split_layout;
