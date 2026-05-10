use crate::core::framework::render::{CorePipelineKind, RenderPhase, RenderPipelineHandle};

use super::renderer_asset::RendererAsset;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderPipelineAsset {
    pub handle: RenderPipelineHandle,
    pub name: String,
    pub core_pipeline: CorePipelineKind,
    pub phase_mapping: Vec<RenderPhase>,
    pub renderer: RendererAsset,
}
