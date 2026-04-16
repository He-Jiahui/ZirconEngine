use zircon_render_server::RenderPipelineHandle;

use super::renderer_asset::RendererAsset;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderPipelineAsset {
    pub handle: RenderPipelineHandle,
    pub name: String,
    pub renderer: RendererAsset,
}
