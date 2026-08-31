use crate::text::{
    RichTextAuthoringDiagnostic, RichTextAuthoringDiagnosticCode,
    RichTextAuthoringDiagnosticSeverity, RichTextAuthoringRecovery,
};

use super::{RichParseBuilder, RichTextContentTrust, RichTextParseError, markup_source_range};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BidiControlKind {
    Mark,
    EmbeddingOpen,
    EmbeddingPop,
    OverrideOpen,
    IsolateOpen,
    IsolatePop,
}

impl BidiControlKind {
    const fn diagnostic_code(self) -> RichTextAuthoringDiagnosticCode {
        match self {
            Self::Mark => RichTextAuthoringDiagnosticCode::BidirectionalMark,
            Self::EmbeddingOpen | Self::EmbeddingPop => {
                RichTextAuthoringDiagnosticCode::BidirectionalEmbedding
            }
            Self::OverrideOpen => RichTextAuthoringDiagnosticCode::BidirectionalOverride,
            Self::IsolateOpen | Self::IsolatePop => {
                RichTextAuthoringDiagnosticCode::BidirectionalIsolate
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BidiControlFrameKind {
    EmbeddingOrOverride,
    Isolate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BidiControlFrame {
    kind: BidiControlFrameKind,
    code: RichTextAuthoringDiagnosticCode,
    source_range: (u32, u32),
}

pub(super) struct BidiControlAdmission {
    content_trust: RichTextContentTrust,
    max_depth: usize,
    stack: Vec<BidiControlFrame>,
}

impl BidiControlAdmission {
    pub(super) const fn new(content_trust: RichTextContentTrust, max_depth: usize) -> Self {
        Self {
            content_trust,
            max_depth,
            stack: Vec::new(),
        }
    }

    pub(super) fn observe(
        &mut self,
        kind: BidiControlKind,
        source_range: (u32, u32),
    ) -> Result<(), RichTextParseError> {
        let code = kind.diagnostic_code();
        if self.content_trust == RichTextContentTrust::Untrusted
            && matches!(
                kind,
                BidiControlKind::EmbeddingOpen
                    | BidiControlKind::EmbeddingPop
                    | BidiControlKind::OverrideOpen
            )
        {
            return Err(RichTextParseError::BidiControlNotAllowed { code, source_range });
        }
        match kind {
            BidiControlKind::Mark => Ok(()),
            BidiControlKind::EmbeddingOpen | BidiControlKind::OverrideOpen => self.push_frame(
                BidiControlFrameKind::EmbeddingOrOverride,
                code,
                source_range,
            ),
            BidiControlKind::EmbeddingPop => match self.stack.last() {
                Some(frame) if frame.kind == BidiControlFrameKind::EmbeddingOrOverride => {
                    self.stack.pop();
                    Ok(())
                }
                _ => Err(RichTextParseError::UnbalancedBidiControl { code, source_range }),
            },
            BidiControlKind::IsolateOpen => {
                self.push_frame(BidiControlFrameKind::Isolate, code, source_range)
            }
            BidiControlKind::IsolatePop => {
                let Some(isolate_index) = self
                    .stack
                    .iter()
                    .rposition(|frame| frame.kind == BidiControlFrameKind::Isolate)
                else {
                    return Err(RichTextParseError::UnbalancedBidiControl { code, source_range });
                };
                self.stack.truncate(isolate_index);
                Ok(())
            }
        }
    }

    fn push_frame(
        &mut self,
        kind: BidiControlFrameKind,
        code: RichTextAuthoringDiagnosticCode,
        source_range: (u32, u32),
    ) -> Result<(), RichTextParseError> {
        let attempted_depth = self.stack.len().checked_add(1).unwrap_or(usize::MAX);
        if attempted_depth > self.max_depth {
            return Err(RichTextParseError::BidiControlDepthExceeded {
                attempted_depth,
                max_depth: self.max_depth,
                source_range,
            });
        }
        self.stack.push(BidiControlFrame {
            kind,
            code,
            source_range,
        });
        Ok(())
    }

    pub(super) fn finish(&self) -> Result<(), RichTextParseError> {
        let Some(frame) = self.stack.last() else {
            return Ok(());
        };
        Err(RichTextParseError::UnbalancedBidiControl {
            code: frame.code,
            source_range: frame.source_range,
        })
    }
}

pub(super) fn push_source_bidi_control_diagnostics(
    result: &mut RichParseBuilder,
    source_text: &str,
    source_start: usize,
) -> Result<(), RichTextParseError> {
    for (offset, character) in source_text.char_indices() {
        let Some(kind) = bidi_control_kind(character) else {
            continue;
        };
        let absolute_start = source_start.checked_add(offset).unwrap_or(usize::MAX);
        let source_range = markup_source_range(absolute_start, character.len_utf8())?;
        push_bidi_control_diagnostic(result, kind, source_range)?;
    }
    Ok(())
}

pub(super) fn push_literal_bidi_control_diagnostic(
    result: &mut RichParseBuilder,
    literal: &str,
    source_range: (u32, u32),
) -> Result<(), RichTextParseError> {
    for character in literal.chars() {
        if let Some(kind) = bidi_control_kind(character) {
            push_bidi_control_diagnostic(result, kind, source_range)?;
        }
    }
    Ok(())
}

fn push_bidi_control_diagnostic(
    result: &mut RichParseBuilder,
    kind: BidiControlKind,
    source_range: (u32, u32),
) -> Result<(), RichTextParseError> {
    result.admit_bidi_control(kind, source_range)?;
    result.push_authoring_diagnostic(RichTextAuthoringDiagnostic {
        severity: RichTextAuthoringDiagnosticSeverity::Warning,
        code: kind.diagnostic_code(),
        source_range,
        recovery: RichTextAuthoringRecovery::PreservedAsText,
    });
    Ok(())
}

const fn bidi_control_kind(character: char) -> Option<BidiControlKind> {
    match character {
        '\u{061c}' | '\u{200e}' | '\u{200f}' => Some(BidiControlKind::Mark),
        '\u{202a}' | '\u{202b}' => Some(BidiControlKind::EmbeddingOpen),
        '\u{202c}' => Some(BidiControlKind::EmbeddingPop),
        '\u{202d}' | '\u{202e}' => Some(BidiControlKind::OverrideOpen),
        '\u{2066}' | '\u{2067}' | '\u{2068}' => Some(BidiControlKind::IsolateOpen),
        '\u{2069}' => Some(BidiControlKind::IsolatePop),
        _ => None,
    }
}
