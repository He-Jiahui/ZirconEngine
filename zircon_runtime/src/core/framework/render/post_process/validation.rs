use thiserror::Error;

use super::super::RenderPipelinePhase;
use super::PostProcessEffectKind;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum PostProcessGraphValidationError {
    #[error("post-process node `{node}` requires missing resource `{resource}`")]
    MissingRequiredInput { node: String, resource: String },
    #[error("post-process node `{node}` produces duplicate resource `{resource}`")]
    DuplicateOutputResource { node: String, resource: String },
    #[error("post-process node `{node}` depends on disabled or missing effect `{dependency}`")]
    MissingDependency {
        node: String,
        dependency: PostProcessEffectKind,
    },
    #[error("post-process node `{node}` requires unavailable view-family phase `{phase:?}`")]
    UnavailableViewFamilyPhase {
        node: String,
        phase: RenderPipelinePhase,
    },
    #[error("resolved view family requires post-process phase `{phase:?}`, but the stack has no node for it")]
    MissingRequiredViewFamilyPhase { phase: RenderPipelinePhase },
    #[error("post-process pass graph contains a dependency cycle")]
    CycleDetected,
}
