#[cfg(test)]
use super::super::VirtualGeometryGpuReadback;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometrySelectedCluster;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_gpu_readback_selected_clusters(
    readback: Option<&VirtualGeometryGpuReadback>,
) -> Option<Vec<RenderVirtualGeometrySelectedCluster>> {
    readback.map(|readback| readback.selected_clusters().to_vec())
}
