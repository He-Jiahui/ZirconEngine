use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) struct GpuPendingRequestInput
{
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) page_id: u32,
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) size_bytes: u32,
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) frontier_rank: u32,
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) assigned_slot: u32,
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) recycled_page_id: u32,
}
