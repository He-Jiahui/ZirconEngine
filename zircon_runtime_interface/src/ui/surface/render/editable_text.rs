use core::fmt;

use serde::{Deserialize, Serialize};

use super::UiTextRange;

/// UTF-8 byte offsets into text, matching Rust string slicing units.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiTextByteRange {
    pub start_byte: u32,
    pub end_byte: u32,
}

impl UiTextByteRange {
    pub const fn new(start_byte: u32, end_byte: u32) -> Self {
        Self {
            start_byte,
            end_byte,
        }
    }
}

/// Styling state for a platform-provided range within IME preedit text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextPreeditClauseKind {
    Input,
    Converted,
    TargetConverted,
    TargetNotConverted,
}

impl UiTextPreeditClauseKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Converted => "converted",
            Self::TargetConverted => "target_converted",
            Self::TargetNotConverted => "target_not_converted",
        }
    }

    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "input" => Some(Self::Input),
            "converted" => Some(Self::Converted),
            "target_converted" => Some(Self::TargetConverted),
            "target_not_converted" => Some(Self::TargetNotConverted),
            _ => None,
        }
    }
}

/// A UTF-8 byte range in preedit text with platform conversion styling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiTextPreeditClause {
    pub range: UiTextByteRange,
    pub kind: UiTextPreeditClauseKind,
}

impl UiTextPreeditClause {
    pub const fn new(range: UiTextByteRange, kind: UiTextPreeditClauseKind) -> Self {
        Self { range, kind }
    }

    /// Validates preedit-local byte ranges and their non-overlapping text order.
    pub fn validate_preedit_payload(
        text: &str,
        cursor_range: Option<UiTextByteRange>,
        preedit_clauses: &[Self],
    ) -> Result<(), UiTextPreeditClauseError> {
        if let Some(cursor_range) = cursor_range {
            validate_preedit_range(cursor_range, text)?;
        }

        let mut previous_start = None;
        let mut previous_end = 0;
        for clause in preedit_clauses {
            validate_preedit_range(clause.range, text)?;
            if previous_start.is_some_and(|start| clause.range.start_byte < start) {
                return Err(UiTextPreeditClauseError::RangeSequenceOutOfOrder);
            }
            if clause.range.start_byte < previous_end {
                return Err(UiTextPreeditClauseError::RangeOverlapsPrevious);
            }
            previous_start = Some(clause.range.start_byte);
            previous_end = clause.range.end_byte;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextPreeditClauseError {
    ClausesRequirePreedit,
    RangeOutOfOrder,
    RangeSequenceOutOfOrder,
    RangeOutsideText,
    RangeNotUtf8Boundary,
    RangeOverlapsPrevious,
}

impl fmt::Display for UiTextPreeditClauseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClausesRequirePreedit => {
                formatter.write_str("preedit clauses require a preedit IME event")
            }
            Self::RangeOutOfOrder => formatter.write_str("preedit range is out of order"),
            Self::RangeSequenceOutOfOrder => {
                formatter.write_str("preedit clause ranges are not in text order")
            }
            Self::RangeOutsideText => formatter.write_str("preedit range exceeds text"),
            Self::RangeNotUtf8Boundary => {
                formatter.write_str("preedit range is not a UTF-8 byte range")
            }
            Self::RangeOverlapsPrevious => {
                formatter.write_str("preedit clause range overlaps a preceding clause")
            }
        }
    }
}

impl std::error::Error for UiTextPreeditClauseError {}

fn validate_preedit_range(
    range: UiTextByteRange,
    text: &str,
) -> Result<(), UiTextPreeditClauseError> {
    let start = range.start_byte as usize;
    let end = range.end_byte as usize;
    if start > end {
        return Err(UiTextPreeditClauseError::RangeOutOfOrder);
    }
    if end > text.len() {
        return Err(UiTextPreeditClauseError::RangeOutsideText);
    }
    if !text.is_char_boundary(start) || !text.is_char_boundary(end) {
        return Err(UiTextPreeditClauseError::RangeNotUtf8Boundary);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextCaretAffinity {
    #[default]
    Downstream,
    Upstream,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextCaret {
    pub offset: usize,
    pub affinity: UiTextCaretAffinity,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextSelection {
    pub anchor: usize,
    pub focus: usize,
}

impl UiTextSelection {
    pub fn collapsed(offset: usize) -> Self {
        Self {
            anchor: offset,
            focus: offset,
        }
    }

    pub fn range(&self) -> UiTextRange {
        UiTextRange {
            start: self.anchor.min(self.focus),
            end: self.anchor.max(self.focus),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextComposition {
    pub range: UiTextRange,
    pub text: String,
    /// Platform-provided clause spans within `text`, retained for marked-text painting.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preedit_clauses: Vec<UiTextPreeditClause>,
    /// Text that occupied `range` before visible preedit replacement; absent for paint-only snapshots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restore_text: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiEditableTextState {
    pub text: String,
    pub caret: UiTextCaret,
    pub selection: Option<UiTextSelection>,
    pub composition: Option<UiTextComposition>,
    pub read_only: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum UiTextEditAction {
    Insert {
        text: String,
    },
    Backspace,
    Delete,
    MoveCaret {
        offset: usize,
        extend_selection: bool,
    },
    SetSelection {
        anchor: usize,
        focus: usize,
    },
    SetComposition {
        range: UiTextRange,
        text: String,
    },
    CommitComposition,
    CancelComposition,
}
