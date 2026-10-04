use std::collections::BTreeSet;

use zircon_runtime_interface::ui::event_ui::UiNodeId;

const MAX_PENDING_TEXT_FOCUS_LOSS_OWNERS: usize = 1_024;

/// 收集本轮派发失焦的文本 owner，供输入管理器结束模型回写并丢弃文档历史。
/// 溢出时放弃部分清单并要求全量清理，避免未记录的 owner 继续保留旧编辑会话。
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct UiPendingTextFocusLoss {
    owners: BTreeSet<UiNodeId>,
    overflowed: bool,
}

pub(crate) struct UiPendingTextFocusLossOwners {
    pub(crate) owners: BTreeSet<UiNodeId>,
    pub(crate) overflowed: bool,
}

impl UiPendingTextFocusLoss {
    pub(super) fn record(&mut self, owner: UiNodeId) {
        if self.overflowed || self.owners.contains(&owner) {
            return;
        }
        if self.owners.len() >= MAX_PENDING_TEXT_FOCUS_LOSS_OWNERS {
            self.owners.clear();
            self.overflowed = true;
            return;
        }
        self.owners.insert(owner);
    }

    pub(super) fn take(&mut self) -> UiPendingTextFocusLossOwners {
        UiPendingTextFocusLossOwners {
            owners: std::mem::take(&mut self.owners),
            overflowed: std::mem::take(&mut self.overflowed),
        }
    }
}

#[cfg(test)]
#[path = "tests/focus_loss.rs"]
mod tests;
