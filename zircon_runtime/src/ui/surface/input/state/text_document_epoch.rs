use std::collections::BTreeMap;

use zircon_runtime_interface::ui::event_ui::UiNodeId;

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
mod tests {
    use super::*;

    #[test]
    fn exhausted_epoch_never_aliases_an_earlier_document_source() {
        let owner = UiNodeId::new(7);
        let mut epochs = UiTextDocumentEpochs::default();
        epochs.revisions.insert(owner, Some(u64::MAX));

        assert_eq!(epochs.advance(owner), None);
        assert_eq!(epochs.current(owner), None);
        assert_eq!(epochs.advance(owner), None);
    }
}
