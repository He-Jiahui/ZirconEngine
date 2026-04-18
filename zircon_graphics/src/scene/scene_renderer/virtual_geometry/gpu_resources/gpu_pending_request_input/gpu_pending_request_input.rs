use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(in crate::scene::scene_renderer::virtual_geometry::gpu_resources) struct GpuPendingRequestInput
{
    pub(in crate::scene::scene_renderer::virtual_geometry::gpu_resources) page_id: u32,
    pub(in crate::scene::scene_renderer::virtual_geometry::gpu_resources) size_bytes: u32,
}
