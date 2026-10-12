#[cfg(test)]
use super::super::VirtualGeometryGpuReadback;
#[cfg(test)]
use zircon_runtime::core::framework::render::RenderVirtualGeometryVisBuffer64Entry;

#[cfg(test)]
pub(crate) fn read_virtual_geometry_gpu_readback_visbuffer64(
    readback: Option<&VirtualGeometryGpuReadback>,
) -> Option<(u64, Vec<RenderVirtualGeometryVisBuffer64Entry>)> {
    readback.map(|readback| {
        (
            readback.visbuffer64_clear_value(),
            readback.visbuffer64_entries().to_vec(),
        )
    })
}
