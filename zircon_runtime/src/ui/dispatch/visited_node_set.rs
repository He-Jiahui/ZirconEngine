use std::collections::HashSet;

use zircon_runtime_interface::ui::event_ui::UiNodeId;

pub(super) const UI_DISPATCH_INLINE_VISITED_NODE_CAPACITY: usize = 16;

// 各路由阶段共用去重集合，确保同一节点不会因 preview/bubble 路径相交而重复调用；浅路由不分配堆空间。
// expected_len 只是深路由升格后的容量提示，不是合法节点数上限。
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
#[path = "tests/visited_node_set.rs"]
mod tests;
