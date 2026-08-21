use std::path::PathBuf;

use crate::asset::assets::{FontFaceExtractionError, FontSourceDecodeError};
use crate::text::font::instance::FontInstanceError;
use crate::text::FontFaceId;

#[derive(Debug, thiserror::Error)]
pub(crate) enum FontDatabaseError {
    #[error("font family is empty")]
    EmptyFamily,
    #[error("font source contains no bytes")]
    EmptyBytes,
    #[error("font source {path} could not be read: {source}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("font source {path} could not be decoded: {source}")]
    SourceDecode {
        path: PathBuf,
        #[source]
        source: FontSourceDecodeError,
    },
    #[error("font face {face_index} could not be materialized: {source}")]
    FaceExtraction {
        face_index: u32,
        #[source]
        source: FontFaceExtractionError,
    },
    #[error("font face bytes are unavailable for {0:?}")]
    FaceBytesUnavailable(FontFaceId),
    #[error("font face is unknown: {0:?}")]
    UnknownFace(FontFaceId),
    #[error("font face has no shaping-backend identity: {0:?}")]
    BackendFaceUnavailable(FontFaceId),
    #[error(transparent)]
    FontInstance(#[from] FontInstanceError),
}
