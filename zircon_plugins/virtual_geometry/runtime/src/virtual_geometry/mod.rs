mod declarations;
mod extract_registration;
mod nanite;
mod normalized_page_table_entries;
mod pending_completion;
mod plan_ingestion;
mod prepare_frame;
mod renderer;
mod residency_management;
mod snapshot;
mod types;
#[cfg(test)]
#[path = "test_sources/virtual_geometry_imported_extract.rs"]
mod virtual_geometry_imported_extract;
#[cfg(test)]
#[path = "test_sources/virtual_geometry_nanite_cpu.rs"]
mod virtual_geometry_nanite_cpu;

pub(crate) use declarations::VirtualGeometryPageRequest;
pub(crate) use declarations::VirtualGeometryRuntimeState;
pub(in crate::virtual_geometry) use declarations::HOT_FRONTIER_COOLING_FRAME_COUNT;
#[cfg(test)]
pub(crate) use nanite::{
    build_virtual_geometry_automatic_extract, build_virtual_geometry_automatic_extract_from_meshes,
    resolve_virtual_geometry_extract, VirtualGeometryAutomaticExtractInstance,
    VirtualGeometryCpuReferenceConfig, VirtualGeometryCpuReferenceFrame,
    VirtualGeometryDebugConfig, VirtualGeometryExecutionMode,
};
#[allow(unused_imports)]
pub(crate) use nanite::{
    build_virtual_geometry_automatic_extract_from_meshes_with_debug,
    VirtualGeometryAutomaticExtractOutput,
};
pub(super) use normalized_page_table_entries::normalized_page_table_entries;
pub(crate) use types::*;
