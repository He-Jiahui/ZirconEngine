mod dynamic_resolution;
mod pipeline;
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
mod tests;
