use std::collections::HashSet;

use zircon_runtime_interface::ui::event_ui::UiNodeId;

pub(super) const UI_DISPATCH_INLINE_VISITED_NODE_CAPACITY: usize = 16;

pub(super) struct UiDispatchVisitedNodeSet {
    inline: [UiNodeId; UI_DISPATCH_INLINE_VISITED_NODE_CAPACITY],
    inline_len: usize,
    expected_len: usize,
    overflow: Option<HashSet<UiNodeId>>,
}

impl UiDispatchVisitedNodeSet {
    pub(super) fn with_expected_len(expected_len: usize) -> Self {
        Self {
            inline: [UiNodeId::new(0); UI_DISPATCH_INLINE_VISITED_NODE_CAPACITY],
            inline_len: 0,
            expected_len,
            overflow: None,
        }
    }

    pub(super) fn insert(&mut self, node_id: UiNodeId) -> bool {
        if let Some(overflow) = self.overflow.as_mut() {
            return overflow.insert(node_id);
        }
        if self.inline[..self.inline_len].contains(&node_id) {
            return false;
        }
        if self.inline_len < UI_DISPATCH_INLINE_VISITED_NODE_CAPACITY {
            self.inline[self.inline_len] = node_id;
            self.inline_len += 1;
            return true;
        }

        let mut overflow = HashSet::with_capacity(
            self.expected_len
                .max(UI_DISPATCH_INLINE_VISITED_NODE_CAPACITY + 1),
        );
        overflow.extend(self.inline.iter().copied());
        let inserted = overflow.insert(node_id);
        self.overflow = Some(overflow);
        inserted
    }

    #[cfg(test)]
    fn uses_heap_storage(&self) -> bool {
        self.overflow.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime200_typical_ui_route_stays_inline_and_deduplicates() {
        let mut visited = UiDispatchVisitedNodeSet::with_expected_len(10);

        for value in 1..=10 {
            assert!(visited.insert(UiNodeId::new(value)));
        }
        assert!(!visited.insert(UiNodeId::new(4)));
        assert!(!visited.uses_heap_storage());
    }

    #[test]
    fn runtime200_deep_route_promotes_once_and_preserves_membership() {
        let mut visited = UiDispatchVisitedNodeSet::with_expected_len(100);

        for value in 1..=UI_DISPATCH_INLINE_VISITED_NODE_CAPACITY as u64 {
            assert!(visited.insert(UiNodeId::new(value)));
        }
        assert!(!visited.uses_heap_storage());
        assert!(visited.insert(UiNodeId::new(17)));
        assert!(visited.uses_heap_storage());
        assert!(!visited.insert(UiNodeId::new(4)));
        assert!(!visited.insert(UiNodeId::new(17)));
        assert!(visited.insert(UiNodeId::new(18)));
    }
}
