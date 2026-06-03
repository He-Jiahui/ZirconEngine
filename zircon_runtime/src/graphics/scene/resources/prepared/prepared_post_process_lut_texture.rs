use std::sync::Arc;

use super::super::PostProcessLutTextureResource;

pub(in crate::graphics::scene::resources) struct PreparedPostProcessLutTexture {
    pub(in crate::graphics::scene::resources) revision: u64,
    pub(in crate::graphics::scene::resources) resource: Arc<PostProcessLutTextureResource>,
}
