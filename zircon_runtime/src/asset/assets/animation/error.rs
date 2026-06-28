use thiserror::Error;

use crate::core::resource::ResourceLocatorError;

pub type AnimationAssetResult<T> = std::result::Result<T, AnimationAssetError>;

#[derive(Debug, Error)]
pub enum AnimationAssetError {
    #[error("animation {kind} binary serialization failed: {source}")]
    Serialize {
        kind: &'static str,
        #[source]
        source: bincode::Error,
    },
    #[error("animation {kind} document decode failed: {source}")]
    DocumentDeserialize {
        kind: &'static str,
        #[source]
        source: bincode::Error,
    },
    #[error("animation {kind} stream header decode failed: {source}")]
    StreamHeaderDeserialize {
        kind: &'static str,
        #[source]
        source: bincode::Error,
    },
    #[error("animation {kind} stream payload decode failed: {source}")]
    StreamPayloadDeserialize {
        kind: &'static str,
        #[source]
        source: bincode::Error,
    },
    #[error("animation {kind} document and stream decode failed: document: {document}; stream: {stream}")]
    DocumentAndStreamDecode {
        kind: &'static str,
        document: Box<AnimationAssetError>,
        stream: Box<AnimationAssetError>,
    },
    #[error("animation {kind} current and v1 payload decode failed: current: {current}; v1: {v1}")]
    CurrentAndV1PayloadDecode {
        kind: &'static str,
        current: Box<AnimationAssetError>,
        v1: Box<AnimationAssetError>,
    },
    #[error("invalid animation asset magic")]
    InvalidMagic,
    #[error("unsupported animation asset version {version}")]
    UnsupportedVersion { version: u32 },
    #[error("animation asset kind mismatch: expected {expected}, found {actual}")]
    KindMismatch {
        expected: &'static str,
        actual: &'static str,
    },
    #[error("animation asset reference uuid `{value}` is invalid: {source}")]
    InvalidReferenceUuid {
        value: String,
        #[source]
        source: uuid::Error,
    },
    #[error("animation asset reference locator `{value}` is invalid: {source}")]
    InvalidReferenceLocator {
        value: String,
        #[source]
        source: ResourceLocatorError,
    },
    #[error("animation graph clip node is missing clip reference")]
    MissingGraphClipReference,
    #[error("unknown animation channel value tag {tag}")]
    UnknownChannelValueTag { tag: u8 },
    #[error("unknown animation graph node tag {tag}")]
    UnknownGraphNodeTag { tag: u8 },
}
