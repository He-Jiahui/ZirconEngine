use crate::core::framework::text::TextLayoutError;
use crate::text::model::{TextShapingFailureReceipt, TextShapingRequestDiagnostics};
use crate::text::ShapedGlyphRun;
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
/// 由 Result 转换时，字体 generation 变化保留为 Deferred；发布者据此区分重试与失败策略。
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
#[path = "tests/outcome.rs"]
mod tests;
