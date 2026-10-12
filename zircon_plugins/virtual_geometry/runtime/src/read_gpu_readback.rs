#[cfg(test)]
use super::super::VirtualGeometryGpuReadback;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometryHardwareRasterizationSource;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_gpu_readback_hardware_rasterization_source(
    readback: Option<&VirtualGeometryGpuReadback>,
) -> Option<RenderVirtualGeometryHardwareRasterizationSource> {
    readback.map(VirtualGeometryGpuReadback::hardware_rasterization_source)
}
