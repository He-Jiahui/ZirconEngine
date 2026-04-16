use std::sync::Arc;

use super::gpu_texture_resource::GpuTextureResource;

pub(super) struct PreparedTexture {
    pub(super) revision: u64,
    pub(super) resource: Arc<GpuTextureResource>,
}
