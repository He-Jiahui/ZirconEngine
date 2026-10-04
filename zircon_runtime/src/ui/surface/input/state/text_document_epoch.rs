use std::collections::BTreeMap;

use zircon_runtime_interface::ui::event_ui::UiNodeId;

/// 连接 surface 属性投影与宿主文档版本，防止准备好的旧编辑覆盖后续文本变化。
/// u64 用尽后保持 None 并拒绝继续编辑，不回绕到旧 source 的版本。
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct UiTextDocumentEpochs {
    revisions: BTreeMap<UiNodeId, Option<u64>>,
}

impl UiTextDocumentEpochs {
    pub(super) fn current(&self, owner: UiNodeId) -> Option<u64> {
        self.revisions.get(&owner).copied().unwrap_or(Some(0))
    }

    pub(super) fn advance(&mut self, owner: UiNodeId) -> Option<u64> {
        let next = self
            .current(owner)
            .and_then(|revision| revision.checked_add(1));
        self.revisions.insert(owner, next);
        next
    }

    pub(super) fn drop_owner(&mut self, owner: UiNodeId) {
        self.revisions.remove(&owner);
    }
}

#[cfg(test)]
#[path = "tests/text_document_epoch.rs"]
mod tests;
