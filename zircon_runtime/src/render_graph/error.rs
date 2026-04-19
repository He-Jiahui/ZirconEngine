use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RenderGraphError {
    #[error("render graph pass `{pass}` is unknown")]
    UnknownPass { pass: usize },
    #[error("render graph `{graph_name}` contains a dependency cycle")]
    CycleDetected { graph_name: String },
}
