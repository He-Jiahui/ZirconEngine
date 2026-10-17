#[path = "declarations/mod.rs"]
mod declarations;
#[path = "extract_registration.rs"]
mod extract_registration;
#[cfg(test)]
#[path = "../tests/geometry_execution_stats.rs"]
mod geometry_execution_stats;
#[path = "nanite/mod.rs"]
mod nanite;
#[path = "../normalized_page_table_entries.rs"]
mod normalized_page_table_entries;
#[path = "pending_completion/mod.rs"]
mod pending_completion;
#[path = "plan_ingestion.rs"]
mod plan_ingestion;
#[path = "prepare_frame/mod.rs"]
mod prepare_frame;
#[cfg(test)]
#[path = "../tests/product_order.rs"]
mod product_order;
#[path = "renderer/mod.rs"]
mod renderer;
#[path = "residency_management/mod.rs"]
mod residency_management;
#[path = "snapshot.rs"]
mod snapshot;
#[path = "types/mod.rs"]
mod types;
#[cfg(test)]
#[path = "../tests/virtual_geometry_imported.rs"]
mod virtual_geometry_imported_extract;
#[cfg(test)]
#[path = "../tests/virtual_geometry_nanite_cpu.rs"]
mod virtual_geometry_nanite_cpu;
#[cfg(test)]
#[path = "../tests/virtual_geometry_node_and.rs"]
mod virtual_geometry_node_and;
#[cfg(test)]
#[path = "../tests/virtual_geometry_render.rs"]
mod virtual_geometry_render_framework_stats;
#[cfg(test)]
#[path = "../tests/unified_indirect_cpu.rs"]
mod virtual_geometry_unified_indirect_cpu;

// Broad moved renderer snapshots stay unwired until their old runtime-owner
// imports are migrated to plugin-local types and public neutral runtime seams.

pub(crate) use declarations::VirtualGeometryPageRequest;
pub(crate) use declarations::VirtualGeometryRuntimeState;
pub(in crate::virtual_geometry) use declarations::HOT_FRONTIER_COOLING_FRAME_COUNT;
#[cfg(test)]
pub(crate) use nanite::{
    build_virtual_geometry_automatic_extract, build_virtual_geometry_automatic_extract_from_meshes,
    resolve_virtual_geometry_extract, VirtualGeometryAutomaticExtractInstance,
    VirtualGeometryCpuReferenceConfig, VirtualGeometryCpuReferenceFrame,
    VirtualGeometryDebugConfig,
};
#[allow(unused_imports)]
pub(crate) use nanite::{
    build_virtual_geometry_automatic_extract_from_admitted_sources,
    build_virtual_geometry_automatic_extract_from_meshes_with_debug,
    VirtualGeometryAutomaticExtractOutput,
};
pub(super) use normalized_page_table_entries::normalized_page_table_entries;
pub(super) use renderer::runtime_prepare_renderer_outputs;
#[cfg(test)]
pub(crate) use renderer::{
    uploader_test_collector_registration, VirtualGeometryGpuReadback,
    VirtualGeometryUploaderTestCollector,
};
pub(crate) use types::*;

pub(crate) use renderer::prepare_virtual_geometry_node_and_cluster_cull;

pub(crate) use renderer::materialize_prepared_cull;
