//! 单次路由内的节点去重存储；阶段范围由创建实例的调用方决定。
//! 指针预览先独立执行，随后实例只覆盖目标与冒泡；导航实例覆盖所选候选列表。

use std::collections::HashSet;

use zircon_runtime_interface::ui::event_ui::UiNodeId;

/// 浅路由的内联槽位数；超过槽位后继续接受节点并升格为堆集合。
pub(super) const UI_DISPATCH_INLINE_VISITED_NODE_CAPACITY: usize = 16;

// 各路由阶段共用去重集合，确保同一节点不会因 preview/bubble 路径相交而重复调用；浅路由不分配堆空间。
// expected_len 只是深路由升格后的容量提示，不是合法节点数上限。
/// 保存一个调用范围内已访问节点；上方旧注释的阶段概括需以上述实例边界为准。
/// # 约束与生命周期
/// 指针预览不共享后建实例；导航只对冒泡列表或根回退列表中选定的一份去重。
/// 有效内联前缀由长度限定，升格后堆集合成为唯一查询与插入来源。
/// 预期长度只影响升格时的容量准备，不限制合法节点总量。
/// # 参考
/// 见同级指针与导航分发器的集合创建点；实例在一次派发结束后释放。
pub(super) struct UiDispatchVisitedNodeSet {
    // 仅长度以内的前缀有效，未用槽位中的零标识不代表已访问节点。
    inline: [UiNodeId; UI_DISPATCH_INLINE_VISITED_NODE_CAPACITY],
    inline_len: usize,
    expected_len: usize,
    // 一旦升格就保留到实例释放，后续插入不再查询内联槽位。
    overflow: Option<HashSet<UiNodeId>>,
}

impl UiDispatchVisitedNodeSet {
    /// 创建空集合并保存容量提示；此时只使用内联存储。
    pub(super) fn with_expected_len(expected_len: usize) -> Self {
        Self {
            inline: [UiNodeId::new(0); UI_DISPATCH_INLINE_VISITED_NODE_CAPACITY],
            inline_len: 0,
            expected_len,
            overflow: None,
        }
    }

    /// 记录节点并返回是否首次出现；重复项不改变存储阶段。
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

        // 容量耗尽后先迁移全部有效槽位，再插入当前节点，保持前后去重结果一致。
        let mut overflow = HashSet::with_capacity(
            self.expected_len
                .max(UI_DISPATCH_INLINE_VISITED_NODE_CAPACITY + 1),
        );
        overflow.extend(self.inline.iter().copied());
        let inserted = overflow.insert(node_id);
        self.overflow = Some(overflow);
        inserted
    }

    /// 仅供直接测试观察升格状态，不向产品调用方暴露存储策略。
    #[cfg(test)]
    fn uses_heap_storage(&self) -> bool {
        self.overflow.is_some()
    }
}

#[cfg(test)]
#[path = "tests/visited_node_set.rs"]
mod tests;
