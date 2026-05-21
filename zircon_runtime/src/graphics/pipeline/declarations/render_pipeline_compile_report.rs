use super::{CompiledRenderPipeline, RendererFeatureContractDiagnostic};

/// Compiled render graph plus authoring diagnostics collected from feature assets.
#[derive(Clone, Debug, PartialEq)]
pub struct RenderPipelineCompileReport {
    pub pipeline: CompiledRenderPipeline,
    pub diagnostics: Vec<RendererFeatureContractDiagnostic>,
}
