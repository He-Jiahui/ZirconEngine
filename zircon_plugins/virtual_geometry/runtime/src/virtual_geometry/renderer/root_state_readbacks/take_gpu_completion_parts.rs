use crate::virtual_geometry::renderer::{
    VirtualGeometryGpuReadback, VirtualGeometryGpuReadbackCompletionParts,
};

pub(crate) fn take_virtual_geometry_gpu_completion_parts(
    readback: &mut Option<VirtualGeometryGpuReadback>,
) -> Option<VirtualGeometryGpuReadbackCompletionParts> {
    readback
        .take()
        .map(VirtualGeometryGpuReadback::into_completion_parts)
}
