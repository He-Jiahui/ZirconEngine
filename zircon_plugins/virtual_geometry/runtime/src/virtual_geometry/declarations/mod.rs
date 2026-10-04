//! 定义页请求、依赖、驻留状态和统计快照；状态迁移由同级模块实现。
mod virtual_geometry_page_request;
mod virtual_geometry_runtime_snapshot;
mod virtual_geometry_runtime_state;

pub(crate) use virtual_geometry_page_request::VirtualGeometryPageRequest;
pub(super) use virtual_geometry_runtime_snapshot::VirtualGeometryRuntimeSnapshot;
pub(crate) use virtual_geometry_runtime_state::VirtualGeometryRuntimeState;
pub(in crate::virtual_geometry) use virtual_geometry_runtime_state::HOT_FRONTIER_COOLING_FRAME_COUNT;
