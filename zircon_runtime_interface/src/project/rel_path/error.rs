use thiserror::Error;

/// Failure to construct a normalized, project-contained relative path.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RelPathError {
    #[error("relative path cannot be empty")]
    Empty,
    #[error("relative path {path:?} cannot be absolute or use a platform prefix")]
    AbsoluteOrPrefixed { path: String },
    #[error("relative path {path:?} cannot contain . or .. components")]
    DotComponent { path: String },
}
