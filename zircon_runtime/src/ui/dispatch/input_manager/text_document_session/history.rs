//! 保存每个 owner 的已提交替换差量及撤销/重做交互状态。
//! 准备历史只产生拥有文本差量的值；队列推进留到文档与属性事务成功之后。
//! 记录新编辑先丢弃重做分支，再按条目数与差量字节预算淘汰最旧记录。

use std::{collections::VecDeque, ops::Range};

use zircon_runtime_interface::ui::{
    surface::{UiEditableTextState, UiTextCaret, UiTextCaretAffinity, UiTextSelection},
    text::UiTextEditKind,
};

use crate::ui::text::{CommittedTextEditIntent, TextEditStateTransition};

pub(super) const MVP_TEXT_HISTORY_MAX_ENTRIES: usize = 100;
pub(super) const MVP_TEXT_HISTORY_MAX_DELTA_BYTES: usize = 1024 * 1024;

/// 选择读取撤销链或重做链，不会自行移动历史条目。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui) enum UiTextHistoryDirection {
    Undo,
    Redo,
}

// 历史状态只在文档/属性事务成功后推进；Barrier 结束可撤销链，安全输入和外部替换不保留旧载荷。
/// 在事务准备阶段决定成功提交后的历史动作。
/// `Record` 拥有差量，`Barrier` 清除链，另两项只移动已有条目；
/// 会话仅在文档提交结果为 Changed 时应用该动作，准备值本身不推进历史。
pub(in crate::ui) enum UiTextHistoryCommit {
    Record(UiTextHistoryEntry),
    Barrier,
    Undo,
    Redo,
}

// 保留一次已提交替换的差量与交互状态，不保存整个编辑文档或暂态 IME 组合。
/// 拥有一次替换的前后 UTF-8 字节范围、删入文本和两侧光标/选区。
///
/// # 约束与生命周期
/// 范围由已同步的编辑意图提供，正文只保留本次差量；条目不借用完整文档。
/// 构造时若处于预编辑，以旧源范围还原提交前选区，而不保留临时组合状态。
pub(in crate::ui) struct UiTextHistoryEntry {
    old: Range<usize>,
    new: Range<usize>,
    removed: String,
    inserted: String,
    before: UiTextInteractionState,
    after: UiTextInteractionState,
}

impl UiTextHistoryEntry {
    /// 从已确认的替换意图及两侧交互状态构造拥有数据的历史条目。
    ///
    /// # 参数与约束
    /// `intent` 的两段范围分别属于提交前后源；`removed`、`inserted` 移入本次差量。
    /// `current`、`next` 仅用于提取光标与选区，不在条目中保留借用。
    /// 调用方须提供与范围相符的正文；此构造器不自行查询存储或验证对应关系。
    ///
    /// # 返回
    /// 返回尚未入队的条目；仅在实际事务成功后交给历史提交动作。
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
    /// 在拥有的当前状态上准备一次撤销或重做，不移动历史队列。
    ///
    /// # 参数
    /// `current` 被消费；`direction` 选择应匹配的范围、正文及恢复的交互状态。
    ///
    /// # 返回与约束
    /// 当前 UTF-8 范围内容不符或新终点加法溢出时返回 None；成功返回下一状态与提交意图。
    /// 成功状态恢复对应光标/选区并清除组合；文档及队列仍由调用方后续事务推进。
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

/// 仅保留可恢复的光标与选区；应用到状态时清除组合，不保存预编辑正文。
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

/// 拥有单个 owner 的撤销/重做队列与两队列合计的差量字节数。
///
/// # 生命周期与预算
/// 新记录清空 redo，再从 undo 头部淘汰至条目和字节均不超限；
/// 撤销/重做只在两队列之间搬移条目，不改变保留字节数。
/// 屏障、失焦、源重绑或 owner 关闭会释放相应历史。
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

    /// 消费已完成文档事务对应的历史动作。
    /// `Record` 触发预算处理，`Barrier` 清空两链；Undo/Redo 在来源队列为空时保持不变。
    /// 本方法不验证文档内容；调用方应先完成匹配检查与文档/属性提交。
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

    /// 先释放 redo 的正文与计数，再纳入新差量并从最旧 undo 开始淘汰。
    /// 单条差量超预算时也会被淘汰；这里限制的是保留历史，不拒绝已经完成的文档编辑。
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
#[path = "../tests/history.rs"]
mod tests;
