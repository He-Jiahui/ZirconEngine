use crate::virtual_geometry::renderer::{
    VirtualGeometryGpuReadback, VirtualGeometryGpuReadbackCompletionParts,
};

// 一次性取得完成记录，后续读取同一 Option 将返回 None。
pub(crate) fn take_virtual_geometry_gpu_completion_parts(
    readback: &mut Option<VirtualGeometryGpuReadback>,
) -> Option<VirtualGeometryGpuReadbackCompletionParts> {
    readback
        .take()
        .map(VirtualGeometryGpuReadback::into_completion_parts)
}
