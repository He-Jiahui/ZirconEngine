use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryVisBuffer64Entry, RenderVirtualGeometryVisBuffer64Source,
};

pub(in crate::graphics::scene::scene_renderer::core) struct VirtualGeometryVisBuffer64PassStoreParts
{
    pub(in crate::graphics::scene::scene_renderer::core) clear_value: u64,
    pub(in crate::graphics::scene::scene_renderer::core) entries:
        Vec<RenderVirtualGeometryVisBuffer64Entry>,
    pub(in crate::graphics::scene::scene_renderer::core) source:
        RenderVirtualGeometryVisBuffer64Source,
    pub(in crate::graphics::scene::scene_renderer::core) entry_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) buffer: Option<Arc<wgpu::Buffer>>,
}
