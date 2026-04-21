mod automatic_extract;
#[allow(dead_code)]
mod cpu_reference;
#[allow(dead_code)]
mod execution_mode;

#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use automatic_extract::resolve_virtual_geometry_extract;
#[allow(unused_imports)]
pub(crate) use automatic_extract::{
    build_virtual_geometry_automatic_extract, build_virtual_geometry_automatic_extract_from_meshes,
    VirtualGeometryAutomaticExtractInstance, VirtualGeometryAutomaticExtractOutput,
};
#[allow(unused_imports)]
pub(crate) use cpu_reference::{
    VirtualGeometryCpuReferenceConfig, VirtualGeometryCpuReferenceFrame,
    VirtualGeometryCpuReferenceLeafCluster, VirtualGeometryCpuReferenceNodeVisit,
    VirtualGeometryDebugConfig,
};
#[allow(unused_imports)]
pub(crate) use execution_mode::VirtualGeometryExecutionMode;
