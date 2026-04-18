mod virtual_geometry_page_request;
#[cfg(test)]
mod virtual_geometry_page_residency_state;
mod virtual_geometry_runtime_snapshot;
mod virtual_geometry_runtime_state;

pub(crate) use virtual_geometry_page_request::VirtualGeometryPageRequest;
#[cfg(test)]
pub(crate) use virtual_geometry_page_residency_state::VirtualGeometryPageResidencyState;
pub(super) use virtual_geometry_runtime_snapshot::VirtualGeometryRuntimeSnapshot;
pub(crate) use virtual_geometry_runtime_state::VirtualGeometryRuntimeState;
