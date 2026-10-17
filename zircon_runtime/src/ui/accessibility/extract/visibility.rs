//! 为一次快照构建缓存有效隐藏状态，先沿根的子链传播，再补齐断开根遍历的父链。
//! 索引只拥有节点 ID 与布尔值；构建中的临时路径/访问集合在各离散分量间复用。

use std::collections::{BTreeMap, HashSet};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    tree::{UiTree, UiTreeNode},
};

use super::is_hidden;

/// 当前树的一次性隐藏状态索引，不借用树；祖先隐藏与节点自身隐藏在构建时合成。
pub(super) struct EffectiveHiddenIndex {
    hidden_by_node: BTreeMap<UiNodeId, bool>,
}

impl EffectiveHiddenIndex {
    /// 从只读树构建索引，逐步调用可失败的预算检查。
    ///
    /// # 返回与约束
    /// 检查错误立即向上传递，不返回半成品索引；父链中的重复访问会结束该次回溯。
    /// 索引反映本次读取的树状态，树改变后由下一次快照重新创建。
    pub(super) fn build<E>(
        tree: &UiTree,
        mut check_deadline: impl FnMut() -> Result<(), E>,
    ) -> Result<Self, E> {
        let mut hidden_by_node = BTreeMap::new();
        let mut stack = tree
            .roots
            .iter()
            .rev()
            .copied()
            .map(|node_id| (node_id, false))
            .collect::<Vec<_>>();
        while let Some((node_id, parent_hidden)) = stack.pop() {
            check_deadline()?;
            if hidden_by_node.contains_key(&node_id) {
                continue;
            }
            let Some(node) = tree.nodes.get(&node_id) else {
                continue;
            };
            let effectively_hidden = parent_hidden || is_hidden(node);
            hidden_by_node.insert(node_id, effectively_hidden);
            stack.extend(
                node.children
                    .iter()
                    .rev()
                    .copied()
                    .map(|child_id| (child_id, effectively_hidden)),
            );
        }

        // Detached components are resolved one at a time. Keep their path and cycle table
        // alive across components so a fragmented tree does not allocate two new collections
        // for every detached node. `clear` preserves the HashSet bucket table for the next path.
        let mut detached_path = Vec::new();
        let mut detached_visited = HashSet::new();
        for node_id in tree.nodes.keys().copied() {
            check_deadline()?;
            if !hidden_by_node.contains_key(&node_id) {
                resolve_detached_node(
                    tree,
                    node_id,
                    &mut hidden_by_node,
                    &mut detached_path,
                    &mut detached_visited,
                    &mut check_deadline,
                )?;
            }
        }
        Ok(Self { hidden_by_node })
    }

    /// 查询已合成状态；索引中不存在的 ID 按未隐藏处理，不表示该节点一定存在。
    pub(super) fn is_hidden(&self, node_id: UiNodeId) -> bool {
        self.hidden_by_node.get(&node_id).copied().unwrap_or(false)
    }
}

fn resolve_detached_node<E>(
    tree: &UiTree,
    node_id: UiNodeId,
    hidden_by_node: &mut BTreeMap<UiNodeId, bool>,
    path: &mut Vec<UiNodeId>,
    visited: &mut HashSet<UiNodeId>,
    check_deadline: &mut impl FnMut() -> Result<(), E>,
) -> Result<(), E> {
    path.clear();
    visited.clear();
    let mut cursor = Some(node_id);
    let inherited_hidden = loop {
        check_deadline()?;
        let Some(current_id) = cursor else {
            break false;
        };
        if let Some(hidden) = hidden_by_node.get(&current_id) {
            break *hidden;
        }
        if !visited.insert(current_id) {
            break false;
        }
        let Some(node) = tree.nodes.get(&current_id) else {
            break false;
        };
        path.push(current_id);
        cursor = node.parent;
    };

    let mut effectively_hidden = inherited_hidden;
    for &current_id in path.iter().rev() {
        check_deadline()?;
        let Some(node) = tree.nodes.get(&current_id) else {
            continue;
        };
        effectively_hidden |= is_hidden(node);
        hidden_by_node.insert(current_id, effectively_hidden);
    }
    Ok(())
}

// BUG: [CR-ZCA-R050-0001] tools/tests/test_runtime_ui_accessibility_extraction_performance_contract.py
// 的隐藏索引源码断言在执行时会失败：它读取本文件，却要求已迁入子测试文件的用例名仍在本文件中。
#[cfg(test)]
#[path = "tests/visibility.rs"]
mod tests;
