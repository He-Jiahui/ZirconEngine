mod build_virtual_geometry_prepare;
mod build_virtual_geometry_runtime;
mod collect_virtual_geometry_evictable_page_ids;

pub(in crate::runtime::server::submit_frame_extract::prepare_runtime_submission) use build_virtual_geometry_prepare::build_virtual_geometry_prepare;
pub(in crate::runtime::server::submit_frame_extract::prepare_runtime_submission) use build_virtual_geometry_runtime::build_virtual_geometry_runtime;
pub(in crate::runtime::server::submit_frame_extract::prepare_runtime_submission) use collect_virtual_geometry_evictable_page_ids::collect_virtual_geometry_evictable_page_ids;
