use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    tree::{UiTree, UiTreeNode},
};

use super::is_hidden;

pub(super) struct EffectiveHiddenIndex {
    hidden_by_node: BTreeMap<UiNodeId, bool>,
}

impl EffectiveHiddenIndex {
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

        for node_id in tree.nodes.keys().copied() {
            check_deadline()?;
            if !hidden_by_node.contains_key(&node_id) {
                resolve_detached_node(tree, node_id, &mut hidden_by_node, &mut check_deadline)?;
            }
        }
        Ok(Self { hidden_by_node })
    }

    pub(super) fn is_hidden(&self, node_id: UiNodeId) -> bool {
        self.hidden_by_node.get(&node_id).copied().unwrap_or(false)
    }
}

fn resolve_detached_node<E>(
    tree: &UiTree,
    node_id: UiNodeId,
    hidden_by_node: &mut BTreeMap<UiNodeId, bool>,
    check_deadline: &mut impl FnMut() -> Result<(), E>,
) -> Result<(), E> {
    let mut path = Vec::new();
    let mut visited = BTreeSet::new();
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
    for current_id in path.into_iter().rev() {
        check_deadline()?;
        let Some(node) = tree.nodes.get(&current_id) else {
            continue;
        };
        effectively_hidden |= is_hidden(node);
        hidden_by_node.insert(current_id, effectively_hidden);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        tree::{UiTree, UiTreeNode, UiVisibility},
    };

    use super::EffectiveHiddenIndex;

    #[test]
    fn effective_hidden_index_propagates_hidden_ancestors() {
        let mut tree = UiTree::new(UiTreeId::new("a11y.effective-hidden-index"));
        tree.insert_root(UiTreeNode::new(id(1), UiNodePath::new("root")));
        tree.insert_child(
            id(1),
            UiTreeNode::new(id(2), UiNodePath::new("root/hidden"))
                .with_visibility(UiVisibility::Collapsed),
        )
        .unwrap();
        tree.insert_child(
            id(2),
            UiTreeNode::new(id(3), UiNodePath::new("root/hidden/child")),
        )
        .unwrap();

        let index = EffectiveHiddenIndex::build(&tree, || Ok::<_, ()>(())).unwrap();

        assert!(!index.is_hidden(id(1)));
        assert!(index.is_hidden(id(2)));
        assert!(index.is_hidden(id(3)));
    }

    fn id(value: u64) -> UiNodeId {
        UiNodeId::new(value)
    }
}
