#[cfg(test)]
use super::super::VirtualGeometryGpuReadback;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_gpu_readback_visbuffer64_entry_count(
    readback: Option<&VirtualGeometryGpuReadback>,
) -> Option<u32> {
    readback.map(VirtualGeometryGpuReadback::visbuffer64_entry_count)
}
