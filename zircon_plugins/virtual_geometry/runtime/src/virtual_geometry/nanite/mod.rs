mod automatic_extract;
#[allow(dead_code)]
mod cpu_reference;
mod page_payload;

#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use automatic_extract::resolve_virtual_geometry_extract;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use automatic_extract::{
    build_virtual_geometry_automatic_extract, build_virtual_geometry_automatic_extract_from_meshes,
    VirtualGeometryAutomaticExtractInstance,
};
#[allow(unused_imports)]
pub(crate) use automatic_extract::{
    build_virtual_geometry_automatic_extract_from_meshes_with_debug,
    VirtualGeometryAutomaticExtractOutput,
};
#[allow(unused_imports)]
pub(crate) use cpu_reference::{
    VirtualGeometryCpuReferenceConfig, VirtualGeometryCpuReferenceFrame,
    VirtualGeometryCpuReferenceLeafCluster, VirtualGeometryCpuReferenceNodeVisit,
    VirtualGeometryDebugConfig,
};
