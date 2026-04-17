mod extract_registration;
mod pending_completion;
mod plan_ingestion;
mod prepare_frame;
mod residency_management;
mod snapshot;
#[cfg(test)]
mod test_accessors;
mod virtual_geometry_page_request;
#[cfg(test)]
mod virtual_geometry_page_residency_state;
mod virtual_geometry_runtime_snapshot;
mod virtual_geometry_runtime_state;

#[cfg(test)]
pub(crate) use virtual_geometry_page_request::VirtualGeometryPageRequest;
#[cfg(test)]
pub(crate) use virtual_geometry_page_residency_state::VirtualGeometryPageResidencyState;
pub(crate) use virtual_geometry_runtime_state::VirtualGeometryRuntimeState;
