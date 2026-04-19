use std::sync::Arc;

use super::super::GpuTextureResource;

pub(in crate::graphics::scene::resources) struct PreparedTexture {
    pub(in crate::graphics::scene::resources) revision: u64,
    pub(in crate::graphics::scene::resources) resource: Arc<GpuTextureResource>,
}
