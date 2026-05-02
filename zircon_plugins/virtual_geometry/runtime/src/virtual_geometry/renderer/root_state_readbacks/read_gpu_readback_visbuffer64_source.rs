#[cfg(test)]
use super::super::VirtualGeometryGpuReadback;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometryVisBuffer64Source;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_gpu_readback_visbuffer64_source(
    readback: Option<&VirtualGeometryGpuReadback>,
) -> Option<RenderVirtualGeometryVisBuffer64Source> {
    readback.map(VirtualGeometryGpuReadback::visbuffer64_source)
}
