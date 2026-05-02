#[cfg(test)]
use super::super::VirtualGeometryGpuReadback;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_gpu_readback_selected_cluster_count(
    readback: Option<&VirtualGeometryGpuReadback>,
) -> Option<u32> {
    readback.map(VirtualGeometryGpuReadback::selected_cluster_count)
}
