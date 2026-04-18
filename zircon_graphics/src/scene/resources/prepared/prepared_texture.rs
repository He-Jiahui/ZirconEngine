use std::sync::Arc;

use super::super::GpuTextureResource;

pub(in crate::scene::resources) struct PreparedTexture {
    pub(in crate::scene::resources) revision: u64,
    pub(in crate::scene::resources) resource: Arc<GpuTextureResource>,
}
