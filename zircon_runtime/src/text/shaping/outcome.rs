use crate::core::framework::text::TextLayoutError;
use crate::text::ShapedGlyphRun;
use crate::text::model::{TextShapingFailureReceipt, TextShapingRequestDiagnostics};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextShapingFailure {
    error: TextLayoutError,
    receipt: Option<TextShapingFailureReceipt>,
    request_diagnostics: TextShapingRequestDiagnostics,
}

impl TextShapingFailure {
    pub(crate) const fn with_receipt(
        error: TextLayoutError,
        receipt: TextShapingFailureReceipt,
    ) -> Self {
        Self {
            error,
            receipt: Some(receipt),
            request_diagnostics: TextShapingRequestDiagnostics::EMPTY,
        }
    }

    pub(crate) const fn with_optional_receipt(
        error: TextLayoutError,
        receipt: Option<TextShapingFailureReceipt>,
    ) -> Self {
        Self {
            error,
            receipt,
            request_diagnostics: TextShapingRequestDiagnostics::EMPTY,
        }
    }

    pub(crate) const fn error(&self) -> &TextLayoutError {
        &self.error
    }

    pub(crate) const fn receipt(&self) -> Option<TextShapingFailureReceipt> {
        self.receipt
    }

    pub(crate) const fn request_diagnostics(&self) -> TextShapingRequestDiagnostics {
        self.request_diagnostics
    }

    pub(crate) fn with_request_diagnostics(
        mut self,
        diagnostics: TextShapingRequestDiagnostics,
    ) -> Self {
        self.request_diagnostics.merge(diagnostics);
        self
    }

    pub(crate) fn replace_request_diagnostics(
        mut self,
        diagnostics: TextShapingRequestDiagnostics,
    ) -> Self {
        self.request_diagnostics = diagnostics;
        self
    }

    pub(crate) fn into_error(self) -> TextLayoutError {
        self.error
    }

    fn ensure_generation_receipt(self) -> Self {
        if self.error == TextLayoutError::FontGenerationChanged && self.receipt.is_none() {
            return Self::font_generation_changed()
                .with_request_diagnostics(self.request_diagnostics);
        }
        self
    }
}

impl From<TextLayoutError> for TextShapingFailure {
    fn from(error: TextLayoutError) -> Self {
        Self {
            error,
            receipt: None,
            request_diagnostics: TextShapingRequestDiagnostics::EMPTY,
        }
    }
}

/// A publishable value plus request-owned diagnostics that must not enter the artifact cache.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TextShapingCompletion<T> {
    value: T,
    diagnostics: TextShapingRequestDiagnostics,
}

impl<T> TextShapingCompletion<T> {
    pub(crate) const fn new(value: T, diagnostics: TextShapingRequestDiagnostics) -> Self {
        Self { value, diagnostics }
    }

    pub(crate) fn into_parts(self) -> (T, TextShapingRequestDiagnostics) {
        (self.value, self.diagnostics)
    }
}

impl Deref for TextShapingFailure {
    type Target = TextLayoutError;

    fn deref(&self) -> &Self::Target {
        self.error()
    }
}

/// Typed internal handoff from a fallible text stage to its owner.
///
/// Only `Ready` contains a publishable value. Error outcomes retain their disposition until the
/// sole publication owner applies an explicit fallback policy; lower stages must not manufacture
/// zero geometry or empty glyph runs.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TextShapingOutcome<T = Arc<ShapedGlyphRun>> {
    Ready(T),
    Deferred(TextShapingFailure),
    Failed(TextShapingFailure),
}

impl<T> TextShapingOutcome<T> {
    pub(crate) fn deferred(error: TextLayoutError) -> Self {
        let failure = TextShapingFailure::from(error).ensure_generation_receipt();
        Self::Deferred(failure)
    }

    pub(crate) fn failed(error: TextLayoutError) -> Self {
        Self::Failed(error.into())
    }

    pub(crate) fn failed_with_receipt(
        error: TextLayoutError,
        receipt: TextShapingFailureReceipt,
    ) -> Self {
        Self::Failed(TextShapingFailure::with_receipt(error, receipt))
    }

    pub(crate) fn from_result(result: Result<T, TextLayoutError>) -> Self {
        match result {
            Ok(value) => Self::Ready(value),
            Err(error @ TextLayoutError::FontGenerationChanged) => Self::deferred(error),
            Err(error) => Self::failed(error),
        }
    }

    pub(crate) fn from_shape_result(result: Result<T, TextShapingFailure>) -> Self {
        match result {
            Ok(value) => Self::Ready(value),
            Err(failure) if failure.error() == &TextLayoutError::FontGenerationChanged => {
                Self::Deferred(failure.ensure_generation_receipt())
            }
            Err(failure) => Self::Failed(failure),
        }
    }

    pub(crate) const fn failure_receipt(&self) -> Option<TextShapingFailureReceipt> {
        match self {
            Self::Deferred(failure) | Self::Failed(failure) => failure.receipt(),
            Self::Ready(_) => None,
        }
    }

    pub(crate) fn map<U>(self, map: impl FnOnce(T) -> U) -> TextShapingOutcome<U> {
        match self {
            Self::Ready(value) => TextShapingOutcome::Ready(map(value)),
            Self::Deferred(error) => TextShapingOutcome::Deferred(error),
            Self::Failed(error) => TextShapingOutcome::Failed(error),
        }
    }

    pub(crate) fn and_then<U>(
        self,
        map: impl FnOnce(T) -> TextShapingOutcome<U>,
    ) -> TextShapingOutcome<U> {
        match self {
            Self::Ready(value) => map(value),
            Self::Deferred(error) => TextShapingOutcome::Deferred(error),
            Self::Failed(error) => TextShapingOutcome::Failed(error),
        }
    }

    pub(crate) fn into_result(self) -> Result<T, TextLayoutError> {
        match self {
            Self::Ready(value) => Ok(value),
            Self::Deferred(failure) | Self::Failed(failure) => Err(failure.into_error()),
        }
    }
}

/// Generic name used by measure, breaking, and publication owners during the M2b hard cut.
pub(crate) type TextLayoutOutcome<T> = TextShapingOutcome<T>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::shaping::{
        TextShapingFailureCode, TextShapingFailureDependency, TextShapingFailureDisposition,
        TextShapingFailurePhase, TextShapingFailureReceipt,
    };
    use crate::text::{FontFaceId, TextRange};

    #[test]
    fn request_diagnostics_are_fixed_and_stay_outside_the_shaped_cache_artifact() {
        assert!(
            std::mem::size_of::<TextShapingRequestDiagnostics>() <= 160,
            "request diagnostics must remain a small fixed-cardinality transient envelope"
        );
        for source in [
            include_str!("../model/shaped_run.rs"),
            include_str!("../cache/shaped_cache/memory.rs"),
        ] {
            assert!(!source.contains("TextShapingRequestDiagnostics"));
            assert!(!source.contains("TextFontResolutionReport"));
        }
    }

    #[test]
    fn classifies_generation_changes_as_deferred_and_other_errors_as_failed() {
        assert!(matches!(
            TextShapingOutcome::<()>::from_result(Err(TextLayoutError::FontGenerationChanged)),
            TextShapingOutcome::Deferred(failure)
                if failure.error() == &TextLayoutError::FontGenerationChanged
                    && failure.receipt().is_some_and(|receipt|
                        receipt.code == TextShapingFailureCode::FontGenerationChanged
                            && receipt.phase == TextShapingFailurePhase::FontResolution
                            && receipt.dependency == TextShapingFailureDependency::FontDatabase
                            && receipt.disposition == TextShapingFailureDisposition::Deferred)
        ));
        assert!(matches!(
            TextShapingOutcome::<()>::from_result(Err(TextLayoutError::BidiInvariant)),
            TextShapingOutcome::Failed(failure)
                if failure.error() == &TextLayoutError::BidiInvariant
        ));
    }

    #[test]
    fn maps_only_ready_values_without_losing_error_disposition() {
        assert_eq!(
            TextShapingOutcome::from_result(Ok(2_u32)).map(|value| value * 2),
            TextShapingOutcome::Ready(4)
        );
        assert_eq!(
            TextShapingOutcome::<u32>::from_result(Err(TextLayoutError::FontGenerationChanged))
                .map(|value| value * 2),
            TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged)
        );
    }

    #[test]
    fn request_failure_receipt_survives_outcome_transforms() {
        let receipt = TextShapingFailureReceipt {
            code: TextShapingFailureCode::BackendFaceParse,
            phase: TextShapingFailurePhase::FontLoad,
            source_range: Some(TextRange { start: 4, end: 9 }),
            face: Some(FontFaceId(7)),
            dependency: TextShapingFailureDependency::FontFace,
            disposition: TextShapingFailureDisposition::AlternateBackend,
            budget: None,
        };
        let outcome =
            TextShapingOutcome::<u32>::failed_with_receipt(TextLayoutError::ShapingFailed, receipt)
                .map(|value| value * 2)
                .and_then(|value| TextShapingOutcome::Ready(value + 1));

        assert_eq!(outcome.failure_receipt(), Some(receipt));
        assert_eq!(outcome.into_result(), Err(TextLayoutError::ShapingFailed));
    }

    #[test]
    fn neutral_layout_failure_does_not_fabricate_a_shaping_receipt() {
        let outcome = TextShapingOutcome::<()>::failed(TextLayoutError::InvalidFontSize);

        assert_eq!(outcome.failure_receipt(), None);
        assert_eq!(outcome.into_result(), Err(TextLayoutError::InvalidFontSize));
    }
}
