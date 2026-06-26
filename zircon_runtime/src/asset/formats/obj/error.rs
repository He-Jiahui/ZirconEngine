use thiserror::Error;

pub(crate) type ObjDecodeResult<T> = std::result::Result<T, ObjDecodeError>;

#[derive(Debug, Error)]
pub(crate) enum ObjDecodeError {
    #[error("read mesh {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("missing {label} at {path}:{line}")]
    MissingScalar {
        path: String,
        line: usize,
        label: String,
    },
    #[error("invalid {label} '{value}' at {path}:{line}: {source}")]
    InvalidScalar {
        path: String,
        line: usize,
        label: String,
        value: String,
        #[source]
        source: std::num::ParseFloatError,
    },
    #[error("face with fewer than 3 vertices at {path}:{line}")]
    FaceVertexCount { path: String, line: usize },
    #[error("parse face vertex at {path}:{line}: {source}")]
    FaceVertex {
        path: String,
        line: usize,
        #[source]
        source: Box<ObjDecodeError>,
    },
    #[error("missing source data for {label}")]
    MissingSourceData { label: String },
    #[error("invalid {label} '{value}': {source}")]
    InvalidIndex {
        label: String,
        value: String,
        #[source]
        source: std::num::ParseIntError,
    },
    #[error("{label} cannot be zero")]
    ZeroIndex { label: String },
    #[error("{label} {value} is out of bounds")]
    IndexOutOfBounds { label: String, value: String },
    #[error("mesh {path} did not contain any vertex positions")]
    EmptyPositions { path: String },
    #[error("mesh {path} did not contain any faces")]
    EmptyFaces { path: String },
}
