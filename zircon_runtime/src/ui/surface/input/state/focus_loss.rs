use std::collections::BTreeSet;

use zircon_runtime_interface::ui::event_ui::UiNodeId;

const MAX_PENDING_TEXT_FOCUS_LOSS_OWNERS: usize = 1_024;

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
mod tests {
    use super::*;

    #[test]
    fn owners_are_deduplicated() {
        let mut pending = UiPendingTextFocusLoss::default();
        pending.record(UiNodeId::new(1));
        pending.record(UiNodeId::new(1));

        let taken = pending.take();

        assert_eq!(taken.owners.len(), 1);
        assert!(!taken.overflowed);
    }

    #[test]
    fn overflow_discards_partial_owners_and_requests_fail_closed_clear() {
        let mut pending = UiPendingTextFocusLoss::default();
        for raw in 1..=MAX_PENDING_TEXT_FOCUS_LOSS_OWNERS as u64 {
            pending.record(UiNodeId::new(raw));
        }
        pending.record(UiNodeId::new(MAX_PENDING_TEXT_FOCUS_LOSS_OWNERS as u64 + 1));

        let taken = pending.take();

        assert!(taken.owners.is_empty());
        assert!(taken.overflowed);
        let reset = pending.take();
        assert!(reset.owners.is_empty());
        assert!(!reset.overflowed);
    }
}
