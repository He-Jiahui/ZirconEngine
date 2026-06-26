use thiserror::Error;

pub type AnimationResult<T> = std::result::Result<T, AnimationError>;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum AnimationError {
    #[error("non-finite skeleton bind {field} for bone `{bone}`")]
    NonFiniteSkeletonBind { bone: String, field: &'static str },
    #[error("zero-length skeleton bind rotation for bone `{bone}`")]
    ZeroLengthSkeletonBindRotation { bone: String },
    #[error("expected {expected} animation sample, found {actual}")]
    SampleTypeMismatch {
        expected: &'static str,
        actual: &'static str,
    },
    #[error("non-finite {sample_kind} animation sample")]
    NonFiniteSample { sample_kind: &'static str },
    #[error("zero-length quaternion animation sample")]
    ZeroLengthQuaternionSample,
    #[error("non-finite animation channel sample `{sample_kind}`")]
    NonFiniteChannelSample { sample_kind: &'static str },
    #[error("zero-length quaternion animation channel sample")]
    ZeroLengthQuaternionChannelSample,
}
