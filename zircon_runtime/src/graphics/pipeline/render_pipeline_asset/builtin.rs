use crate::core::framework::render::RenderPipelineHandle;

use crate::graphics::pipeline::declarations::RenderPipelineAsset;

impl RenderPipelineAsset {
    pub fn builtin(handle: RenderPipelineHandle) -> Option<Self> {
        match handle.raw() {
            1 => Some(Self::default_forward_plus()),
            2 => Some(Self::default_deferred()),
            3 => Some(Self::default_core2d()),
            _ => None,
        }
    }
}
