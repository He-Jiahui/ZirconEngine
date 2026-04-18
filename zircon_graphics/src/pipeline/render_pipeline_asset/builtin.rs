use zircon_framework::render::RenderPipelineHandle;

use crate::pipeline::declarations::RenderPipelineAsset;

impl RenderPipelineAsset {
    pub fn builtin(handle: RenderPipelineHandle) -> Option<Self> {
        match handle.raw() {
            1 => Some(Self::default_forward_plus()),
            2 => Some(Self::default_deferred()),
            _ => None,
        }
    }
}
