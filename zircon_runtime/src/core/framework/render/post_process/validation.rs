use thiserror::Error;

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
    #[error("post-process pass graph contains a dependency cycle")]
    CycleDetected,
}
