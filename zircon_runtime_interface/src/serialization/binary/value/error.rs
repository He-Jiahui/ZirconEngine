use thiserror::Error;

#[derive(Debug, Error)]
pub(in crate::serialization) enum BinaryValueError {
    #[error("floating-point value {value} is not finite")]
    NonFiniteFloat { value: f64 },
    #[error("JSON number {value} cannot be represented by the binary v1 numeric domain")]
    InvalidJsonNumber { value: String },
    #[error("object contains duplicate key {key:?}")]
    DuplicateObjectKey { key: String },
    #[error("binary value stream is empty")]
    EmptyValue,
    #[error("binary value stream contains more than one root value")]
    MultipleRootValues,
    #[error("binary value stream ended before a container was complete")]
    IncompleteContainer,
    #[error("object value is missing its preceding key")]
    MissingObjectKey,
    #[error("object key {key:?} appears outside an object key position")]
    UnexpectedObjectKey { key: String },
    #[error("{kind} node reached the primitive value decoder")]
    UnexpectedNodeKind { kind: &'static str },
    #[error("binary value nesting depth {found} exceeds maximum {max}")]
    DepthLimitExceeded { max: usize, found: usize },
    #[error("binary value node count {found} exceeds maximum {max}")]
    NodeLimitExceeded { max: usize, found: usize },
    #[error("container entry count {found} exceeds maximum {max}")]
    ContainerLimitExceeded { max: usize, found: usize },
    #[error("string byte length {found} exceeds maximum {max}")]
    StringLimitExceeded { max: usize, found: usize },
}
