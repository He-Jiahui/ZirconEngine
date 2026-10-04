//! 管线资产是 renderer data 与运行时编译图之间可修订的契约，作者内容变化要推进修订。
use crate::core::framework::render::{CorePipelineKind, RenderPhase, RenderPipelineHandle};

use super::renderer_asset::RendererAsset;

/// 注册到框架的作者资产；修订号参与编译图缓存键。
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
