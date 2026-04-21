mod declarations;
mod extract_registration;
mod nanite;
mod normalized_page_table_entries;
mod pending_completion;
mod plan_ingestion;
mod prepare_frame;
mod residency_management;
mod snapshot;
#[cfg(test)]
mod test_accessors;

pub(crate) use declarations::VirtualGeometryPageRequest;
#[cfg(test)]
pub(crate) use declarations::VirtualGeometryPageResidencyState;
pub(crate) use declarations::VirtualGeometryRuntimeState;
pub(in crate::graphics::runtime::virtual_geometry) use declarations::HOT_FRONTIER_COOLING_FRAME_COUNT;
pub(crate) use nanite::build_virtual_geometry_automatic_extract_from_meshes;
pub(crate) use nanite::VirtualGeometryAutomaticExtractOutput;
#[cfg(test)]
pub(crate) use nanite::{
    build_virtual_geometry_automatic_extract, resolve_virtual_geometry_extract,
    VirtualGeometryAutomaticExtractInstance, VirtualGeometryCpuReferenceConfig,
    VirtualGeometryCpuReferenceFrame, VirtualGeometryDebugConfig, VirtualGeometryExecutionMode,
};
pub(crate) use normalized_page_table_entries::normalized_page_table_entries;
