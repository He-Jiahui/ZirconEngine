//! Typed failures for versioned animation resource serialization.

use thiserror::Error;

use crate::core::resource::ResourceLocatorError;

pub type AnimationAssetResult<T> = std::result::Result<T, AnimationAssetError>;

#[derive(Debug, Error)]
pub enum AnimationAssetError {
    #[error(
        "animation {kind} binary input is {actual_bytes} bytes, exceeding the {limit_bytes}-byte decode budget"
    )]
    InputTooLarge {
        kind: &'static str,
        actual_bytes: u64,
        limit_bytes: u64,
    },
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
    #[error(
        "animation {kind} document and stream decode failed: document: {document}; stream: {stream}"
    )]
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
    #[error(
        "animation {kind} current, v3, v2, and v1 payload decode failed: current: {current}; v3: {v3}; v2: {v2}; v1: {v1}"
    )]
    CurrentV3V2AndV1PayloadDecode {
        kind: &'static str,
        current: Box<AnimationAssetError>,
        v3: Box<AnimationAssetError>,
        v2: Box<AnimationAssetError>,
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

impl AnimationAssetError {
    /// Returns the binary envelope kind mismatch retained by a schema fallback chain.
    pub fn binary_kind_mismatch(&self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::KindMismatch { expected, actual } => Some((expected, actual)),
            Self::DocumentAndStreamDecode {
                document, stream, ..
            } => document
                .binary_kind_mismatch()
                .or_else(|| stream.binary_kind_mismatch()),
            Self::CurrentAndV1PayloadDecode { current, v1, .. } => current
                .binary_kind_mismatch()
                .or_else(|| v1.binary_kind_mismatch()),
            Self::CurrentV3V2AndV1PayloadDecode {
                current,
                v3,
                v2,
                v1,
                ..
            } => current
                .binary_kind_mismatch()
                .or_else(|| v3.binary_kind_mismatch())
                .or_else(|| v2.binary_kind_mismatch())
                .or_else(|| v1.binary_kind_mismatch()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AnimationAssetError;

    #[test]
    fn binary_kind_mismatch_survives_document_and_schema_fallback_errors() {
        let error = AnimationAssetError::CurrentAndV1PayloadDecode {
            kind: "sequence",
            current: Box::new(AnimationAssetError::DocumentAndStreamDecode {
                kind: "sequence",
                document: Box::new(AnimationAssetError::KindMismatch {
                    expected: "sequence",
                    actual: "graph",
                }),
                stream: Box::new(AnimationAssetError::InvalidMagic),
            }),
            v1: Box::new(AnimationAssetError::InvalidMagic),
        };

        assert_eq!(error.binary_kind_mismatch(), Some(("sequence", "graph")));
        assert_eq!(
            AnimationAssetError::InvalidMagic.binary_kind_mismatch(),
            None
        );
    }
}
