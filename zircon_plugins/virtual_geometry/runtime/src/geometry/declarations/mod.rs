//! 定义页请求、依赖、驻留状态和统计快照；状态迁移由同级模块实现。
#[path = "../../virtual_geometry_page_request.rs"]
mod virtual_geometry_page_request;
#[path = "../../virtual_geometry_runtime_snapshot.rs"]
mod virtual_geometry_runtime_snapshot;
#[path = "state/mod.rs"]
mod virtual_geometry_runtime_state;

pub(crate) use virtual_geometry_page_request::VirtualGeometryPageRequest;
pub(super) use virtual_geometry_runtime_snapshot::VirtualGeometryRuntimeSnapshot;
pub(crate) use virtual_geometry_runtime_state::VirtualGeometryRuntimeState;
pub(in crate::virtual_geometry) use virtual_geometry_runtime_state::HOT_FRONTIER_COOLING_FRAME_COUNT;
