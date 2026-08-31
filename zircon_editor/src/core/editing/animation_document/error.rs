use thiserror::Error;

use super::{AnimationAuthoringDocumentKind, AnimationDocumentRevision};

#[derive(Debug, Error)]
pub(crate) enum AnimationAuthoringDocumentError {
    #[error("animation authoring document {document} is already attached")]
    DuplicateDocument { document: u64 },
    #[error("animation authoring document {document} is missing")]
    MissingDocument { document: u64 },
    #[error("animation authoring document revision space is exhausted")]
    RevisionExhausted,
    #[error(
        "animation authoring document {document} revision is stale: expected {expected}, current {actual}"
    )]
    StaleRevision {
        document: u64,
        expected: u64,
        actual: u64,
    },
    #[error("animation authoring document kind mismatch: expected {expected}, actual {actual}")]
    WrongKind {
        expected: &'static str,
        actual: &'static str,
    },
    #[error("animation document source serialization failed: {message}")]
    Serialization { message: String },
}

impl AnimationAuthoringDocumentError {
    pub(crate) fn wrong_kind(
        expected: AnimationAuthoringDocumentKind,
        actual: AnimationAuthoringDocumentKind,
    ) -> Self {
        Self::WrongKind {
            expected: expected.label(),
            actual: actual.label(),
        }
    }

    pub(crate) fn stale_revision(
        document: u64,
        expected: AnimationDocumentRevision,
        actual: AnimationDocumentRevision,
    ) -> Self {
        Self::StaleRevision {
            document,
            expected: expected.value(),
            actual: actual.value(),
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum AnimationDocumentMutationError {
    #[error(transparent)]
    Document(#[from] AnimationAuthoringDocumentError),
    #[error("invalid animation track path: {message}")]
    InvalidTrackPath { message: String },
    #[error("invalid animation graph locator: {message}")]
    InvalidGraphLocator { message: String },
}
