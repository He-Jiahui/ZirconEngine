#[path = "dynamic_resolution.rs"]
mod dynamic_resolution;
#[path = "pipeline.rs"]
mod pipeline;
#[path = "resolution.rs"]
mod resolution;

pub use dynamic_resolution::{
    RenderDynamicResolutionController, RenderDynamicResolutionDecision,
    RenderDynamicResolutionDecisionReason, RenderDynamicResolutionGpuSample,
    RenderDynamicResolutionScope,
};
pub use pipeline::{RenderPipelinePhase, RenderViewFamilyPipeline};
pub use resolution::{
    RenderResolutionPlan, RenderResolutionPolicy, RenderTemporalHistoryKey, RenderUpscalerKind,
    RenderViewFamilyPhaseTargets, RenderViewFamilyTarget, MAX_RENDER_RESOLUTION_FRACTION,
    MIN_RENDER_RESOLUTION_FRACTION,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
