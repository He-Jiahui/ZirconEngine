use serde::{Deserialize, Serialize};

use super::UiTextRange;

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
