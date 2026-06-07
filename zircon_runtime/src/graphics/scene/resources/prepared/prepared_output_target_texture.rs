use std::sync::Arc;

use super::super::OutputTargetTextureResource;

pub(in crate::graphics::scene::resources) struct PreparedOutputTargetTexture {
    pub(in crate::graphics::scene::resources) revision: u64,
    #[allow(dead_code)]
    pub(in crate::graphics::scene::resources) resource: Arc<OutputTargetTextureResource>,
}
