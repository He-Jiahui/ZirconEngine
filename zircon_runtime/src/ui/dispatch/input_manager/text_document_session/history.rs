use std::{collections::VecDeque, ops::Range};

use zircon_runtime_interface::ui::{
    surface::{UiEditableTextState, UiTextCaret, UiTextCaretAffinity, UiTextSelection},
    text::UiTextEditKind,
};

use crate::ui::text::{CommittedTextEditIntent, TextEditStateTransition};

pub(super) const MVP_TEXT_HISTORY_MAX_ENTRIES: usize = 100;
pub(super) const MVP_TEXT_HISTORY_MAX_DELTA_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui) enum UiTextHistoryDirection {
    Undo,
    Redo,
}

// 历史状态只在文档/属性事务成功后推进；Barrier 结束可撤销链，安全输入和外部替换不保留旧载荷。
pub(in crate::ui) enum UiTextHistoryCommit {
    Record(UiTextHistoryEntry),
    Barrier,
    Undo,
    Redo,
}

// 保留一次已提交替换的差量与交互状态，不保存整个编辑文档或暂态 IME 组合。
pub(in crate::ui) struct UiTextHistoryEntry {
    old: Range<usize>,
    new: Range<usize>,
    removed: String,
    inserted: String,
    before: UiTextInteractionState,
    after: UiTextInteractionState,
}

impl UiTextHistoryEntry {
    pub(super) fn new(
        intent: &CommittedTextEditIntent,
        removed: String,
        inserted: String,
        current: &UiEditableTextState,
        next: &UiEditableTextState,
    ) -> Self {
        let before = if current.composition.is_some() {
            UiTextInteractionState::for_composition_source(&intent.old)
        } else {
            UiTextInteractionState::from_state(current)
        };
        Self {
            old: intent.old.clone(),
            new: intent.new.clone(),
            removed,
            inserted,
            before,
            after: UiTextInteractionState::from_state(next),
        }
    }

    pub(super) fn retained_bytes(&self) -> usize {
        self.removed.len().saturating_add(self.inserted.len())
    }

    pub(super) fn expected_range(&self, direction: UiTextHistoryDirection) -> Range<usize> {
        match direction {
            UiTextHistoryDirection::Undo => self.new.clone(),
            UiTextHistoryDirection::Redo => self.old.clone(),
        }
    }

    pub(super) fn expected_text(&self, direction: UiTextHistoryDirection) -> &str {
        match direction {
            UiTextHistoryDirection::Undo => &self.inserted,
            UiTextHistoryDirection::Redo => &self.removed,
        }
    }

    // 先匹配当前范围的预期文本再构造逆向操作，拒绝把已失效历史强行套到外部更新后的内容。
    pub(super) fn transition(
        &self,
        mut current: UiEditableTextState,
        direction: UiTextHistoryDirection,
    ) -> Option<TextEditStateTransition> {
        let range = self.expected_range(direction);
        if current.text.get(range.clone()) != Some(self.expected_text(direction)) {
            return None;
        }
        let (replacement, interaction, kind) = match direction {
            UiTextHistoryDirection::Undo => (&self.removed, &self.before, UiTextEditKind::Undo),
            UiTextHistoryDirection::Redo => (&self.inserted, &self.after, UiTextEditKind::Redo),
        };
        let new_end = range.start.checked_add(replacement.len())?;
        current.text.replace_range(range.clone(), replacement);
        interaction.apply_to(&mut current);
        Some(TextEditStateTransition {
            state: current,
            committed: Some(CommittedTextEditIntent {
                old: range.clone(),
                new: range.start..new_end,
                kind,
            }),
        })
    }
}

struct UiTextInteractionState {
    caret: UiTextCaret,
    selection: Option<UiTextSelection>,
}

impl UiTextInteractionState {
    fn from_state(state: &UiEditableTextState) -> Self {
        Self {
            caret: state.caret.clone(),
            selection: state.selection.clone(),
        }
    }

    fn for_composition_source(range: &Range<usize>) -> Self {
        let selection = (!range.is_empty()).then(|| UiTextSelection {
            anchor: range.start,
            focus: range.end,
        });
        Self {
            caret: UiTextCaret {
                offset: range.end,
                affinity: UiTextCaretAffinity::Downstream,
            },
            selection,
        }
    }

    fn apply_to(&self, state: &mut UiEditableTextState) {
        state.caret = self.caret.clone();
        state.selection = self.selection.clone();
        state.composition = None;
    }
}

#[derive(Default)]
pub(super) struct UiTextDocumentHistory {
    undo: VecDeque<UiTextHistoryEntry>,
    redo: VecDeque<UiTextHistoryEntry>,
    retained_bytes: usize,
}

impl UiTextDocumentHistory {
    pub(super) fn latest(&self, direction: UiTextHistoryDirection) -> Option<&UiTextHistoryEntry> {
        match direction {
            UiTextHistoryDirection::Undo => self.undo.back(),
            UiTextHistoryDirection::Redo => self.redo.back(),
        }
    }

    pub(super) fn commit(&mut self, commit: UiTextHistoryCommit) {
        match commit {
            UiTextHistoryCommit::Record(entry) => self.record(entry),
            UiTextHistoryCommit::Barrier => self.clear(),
            UiTextHistoryCommit::Undo => {
                if let Some(entry) = self.undo.pop_back() {
                    self.redo.push_back(entry);
                }
            }
            UiTextHistoryCommit::Redo => {
                if let Some(entry) = self.redo.pop_back() {
                    self.undo.push_back(entry);
                }
            }
        }
    }

    pub(super) fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.retained_bytes = 0;
    }

    fn record(&mut self, entry: UiTextHistoryEntry) {
        self.clear_redo();
        self.retained_bytes = self.retained_bytes.saturating_add(entry.retained_bytes());
        self.undo.push_back(entry);
        while self.undo.len() > MVP_TEXT_HISTORY_MAX_ENTRIES
            || self.retained_bytes > MVP_TEXT_HISTORY_MAX_DELTA_BYTES
        {
            let Some(removed) = self.undo.pop_front() else {
                break;
            };
            self.retained_bytes = self.retained_bytes.saturating_sub(removed.retained_bytes());
        }
    }

    fn clear_redo(&mut self) {
        for entry in self.redo.drain(..) {
            self.retained_bytes = self.retained_bytes.saturating_sub(entry.retained_bytes());
        }
    }
}

#[cfg(test)]
#[path = "tests/history.rs"]
mod tests;
