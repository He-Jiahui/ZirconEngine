use crate::core::framework::render::{CorePipelineKind, RenderPhase, RenderPipelineHandle};

use super::renderer_asset::RendererAsset;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderPipelineAsset {
    pub handle: RenderPipelineHandle,
    pub revision: u64,
    pub name: String,
    pub core_pipeline: CorePipelineKind,
    pub phase_mapping: Vec<RenderPhase>,
    pub renderer: RendererAsset,
}

impl RenderPipelineAsset {
    pub fn with_revision(mut self, revision: u64) -> Self {
        self.revision = revision;
        self
    }

    pub fn bump_revision(&mut self) {
        self.revision = self.revision.saturating_add(1).max(1);
    }
}
