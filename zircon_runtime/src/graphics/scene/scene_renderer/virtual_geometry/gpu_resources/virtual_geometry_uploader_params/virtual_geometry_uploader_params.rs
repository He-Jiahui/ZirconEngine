use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) struct VirtualGeometryUploaderParams
{
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) pending_count: u32,
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) available_slot_count: u32,
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) evictable_count: u32,
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) page_budget: u32,
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) streaming_budget_bytes:
        u32,
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) reclaimable_bytes: u32,
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) resident_count: u32,
    pub(in crate::graphics::scene::scene_renderer::virtual_geometry::gpu_resources) _padding1: u32,
}
