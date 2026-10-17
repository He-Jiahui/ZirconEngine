use thiserror::Error;

/// Failure to construct a normalized, project-contained relative path.
/// 错误只描述相对路径的词法形状；它不证明目标在实际项目根目录内。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum RelPathError {
    #[error("relative path cannot be empty")]
    Empty,
    #[error("relative path {path:?} cannot be absolute or use a platform prefix")]
    AbsoluteOrPrefixed { path: String },
    #[error("relative path {path:?} cannot contain . or .. components")]
    DotComponent { path: String },
}
