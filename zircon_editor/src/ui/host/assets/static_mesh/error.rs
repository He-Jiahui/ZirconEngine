use zircon_runtime::asset::{AssetReference, AssetUri};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum StaticMeshSourceError {
    #[error("model/instance source mismatch: expected {expected}, got {actual}")]
    ModelMismatch {
        expected: AssetUri,
        actual: AssetUri,
    },
    #[error("base primitive binding count differs: expected {expected}, got {actual}")]
    BindingCountMismatch { expected: usize, actual: usize },
    #[error("source primitive {primitive_index} has no exact mesh reference")]
    MissingMeshReference { primitive_index: usize },
    #[error("source primitive {primitive_index} has a different mesh reference")]
    MeshReferenceMismatch {
        primitive_index: usize,
        expected: Box<AssetReference>,
        actual: Box<AssetReference>,
    },
}
