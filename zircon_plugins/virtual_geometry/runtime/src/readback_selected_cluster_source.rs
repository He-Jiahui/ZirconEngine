#[cfg(test)]
use super::super::VirtualGeometryGpuReadback;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometrySelectedClusterSource;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_gpu_readback_selected_cluster_source(
    readback: Option<&VirtualGeometryGpuReadback>,
) -> Option<RenderVirtualGeometrySelectedClusterSource> {
    readback.map(VirtualGeometryGpuReadback::selected_cluster_source)
}
