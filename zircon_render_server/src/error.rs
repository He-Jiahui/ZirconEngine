use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum RenderServerError {
    #[error("render viewport `{viewport}` does not exist")]
    UnknownViewport { viewport: u64 },
    #[error("render pipeline `{pipeline}` does not exist")]
    UnknownPipeline { pipeline: u64 },
    #[error("render backend error: {0}")]
    Backend(String),
}
