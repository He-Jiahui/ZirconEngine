use crate::virtual_geometry::renderer::VirtualGeometryGpuReadback;

#[cfg(test)]
pub(crate) fn take_virtual_geometry_gpu_readback(
    readback: &mut Option<VirtualGeometryGpuReadback>,
) -> Option<VirtualGeometryGpuReadback> {
    readback.take()
}
