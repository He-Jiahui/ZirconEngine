use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{event_ui::UiNodeId, surface::UiArrangedTree};

const VISIBILITY_UNKNOWN: u8 = 0;
const VISIBILITY_VISITING: u8 = 1;
const VISIBILITY_HIDDEN: u8 = 2;
const VISIBILITY_VISIBLE: u8 = 3;
const VISIBILITY_WORD_BITS: usize = u64::BITS as usize;

/// Compact inherited render visibility published with the arranged-node index.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct UiArrangedVisibilityIndex {
    node_ids: Vec<UiNodeId>,
    render_visible_words: Vec<u64>,
}

impl UiArrangedVisibilityIndex {
    pub(crate) fn from_arranged(
        arranged_tree: &UiArrangedTree,
        node_indices: &BTreeMap<UiNodeId, usize>,
    ) -> Self {
        let mut index = Self::default();
        index.rebuild(arranged_tree, node_indices);
        index
    }

    pub(crate) fn rebuild(
        &mut self,
        arranged_tree: &UiArrangedTree,
        node_indices: &BTreeMap<UiNodeId, usize>,
    ) {
        let mut states = vec![VISIBILITY_UNKNOWN; arranged_tree.nodes.len()];
        let mut path = Vec::new();

        for start_index in 0..arranged_tree.nodes.len() {
            if is_resolved(states[start_index]) {
                continue;
            }
            path.clear();
            let mut current_index = start_index;
            let mut parent_visible;

            loop {
                match states[current_index] {
                    VISIBILITY_VISIBLE => {
                        parent_visible = true;
                        break;
                    }
                    VISIBILITY_HIDDEN => {
                        parent_visible = false;
                        break;
                    }
                    VISIBILITY_VISITING => {
                        // A malformed parent cycle must never leave descendants render-visible.
                        parent_visible = false;
                        break;
                    }
                    VISIBILITY_UNKNOWN => {}
                    _ => unreachable!("arranged visibility state is bounded"),
                }

                states[current_index] = VISIBILITY_VISITING;
                path.push(current_index);
                let current = &arranged_tree.nodes[current_index];
                let Some(parent_id) = current.parent else {
                    parent_visible = true;
                    break;
                };
                let Some(next_index) = node_indices.get(&parent_id).copied() else {
                    parent_visible = false;
                    break;
                };
                if arranged_tree
                    .nodes
                    .get(next_index)
                    .is_none_or(|node| node.node_id != parent_id)
                {
                    parent_visible = false;
                    break;
                }
                current_index = next_index;
            }

            while let Some(index) = path.pop() {
                parent_visible = parent_visible && arranged_tree.nodes[index].is_render_visible();
                states[index] = if parent_visible {
                    VISIBILITY_VISIBLE
                } else {
                    VISIBILITY_HIDDEN
                };
            }
        }

        self.node_ids.clear();
        self.node_ids.extend(node_indices.keys().copied());
        self.render_visible_words.clear();
        self.render_visible_words.resize(
            self.node_ids.len().saturating_add(VISIBILITY_WORD_BITS - 1) / VISIBILITY_WORD_BITS,
            0,
        );
        for (sorted_index, node_id) in self.node_ids.iter().copied().enumerate() {
            let visible = node_indices
                .get(&node_id)
                .and_then(|arranged_index| states.get(*arranged_index))
                .is_some_and(|state| *state == VISIBILITY_VISIBLE);
            if visible {
                self.render_visible_words[sorted_index / VISIBILITY_WORD_BITS] |=
                    1_u64 << (sorted_index % VISIBILITY_WORD_BITS);
            }
        }
    }

    pub(crate) fn is_render_visible(&self, node_id: UiNodeId) -> bool {
        let Ok(index) = self.node_ids.binary_search(&node_id) else {
            return false;
        };
        self.render_visible_words
            .get(index / VISIBILITY_WORD_BITS)
            .is_some_and(|word| word & (1_u64 << (index % VISIBILITY_WORD_BITS)) != 0)
    }
}

fn is_resolved(state: u8) -> bool {
    matches!(state, VISIBILITY_HIDDEN | VISIBILITY_VISIBLE)
}

#[cfg(test)]
mod tests {
    use super::UiArrangedVisibilityIndex;
    use crate::ui::surface::arranged_node_indices;
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        layout::UiFrame,
        surface::{UiArrangedNode, UiArrangedTree},
        tree::{UiInputPolicy, UiVisibility},
    };

    #[test]
    fn hidden_ancestor_hides_visible_descendants() {
        let root = UiNodeId::new(10);
        let child = UiNodeId::new(2);
        let tree = arranged_tree(vec![
            arranged_node(child, Some(root), UiVisibility::Visible),
            arranged_node(root, None, UiVisibility::Hidden),
        ]);
        let index = visibility_index(&tree);

        assert!(!index.is_render_visible(root));
        assert!(!index.is_render_visible(child));
    }

    #[test]
    fn self_hit_test_invisible_ancestor_remains_render_visible() {
        let root = UiNodeId::new(1);
        let child = UiNodeId::new(2);
        let tree = arranged_tree(vec![
            arranged_node(child, Some(root), UiVisibility::Visible),
            arranged_node(root, None, UiVisibility::SelfHitTestInvisible),
        ]);
        let index = visibility_index(&tree);

        assert!(index.is_render_visible(root));
        assert!(index.is_render_visible(child));
    }

    #[test]
    fn missing_parent_fails_closed() {
        let node_id = UiNodeId::new(1);
        let tree = arranged_tree(vec![arranged_node(
            node_id,
            Some(UiNodeId::new(99)),
            UiVisibility::Visible,
        )]);

        assert!(!visibility_index(&tree).is_render_visible(node_id));
    }

    #[test]
    fn parent_cycle_fails_closed() {
        let first = UiNodeId::new(1);
        let second = UiNodeId::new(2);
        let tree = arranged_tree(vec![
            arranged_node(first, Some(second), UiVisibility::Visible),
            arranged_node(second, Some(first), UiVisibility::Visible),
        ]);
        let index = visibility_index(&tree);

        assert!(!index.is_render_visible(first));
        assert!(!index.is_render_visible(second));
    }

    fn visibility_index(tree: &UiArrangedTree) -> UiArrangedVisibilityIndex {
        UiArrangedVisibilityIndex::from_arranged(tree, &arranged_node_indices(tree))
    }

    fn arranged_tree(nodes: Vec<UiArrangedNode>) -> UiArrangedTree {
        UiArrangedTree {
            tree_id: UiTreeId::new("arranged.visibility.index"),
            draw_order: nodes
                .iter()
                .map(|node| node.node_id)
                .collect::<Vec<_>>()
                .into(),
            nodes: nodes.into(),
            ..UiArrangedTree::default()
        }
    }

    fn arranged_node(
        node_id: UiNodeId,
        parent: Option<UiNodeId>,
        visibility: UiVisibility,
    ) -> UiArrangedNode {
        UiArrangedNode {
            node_id,
            node_path: UiNodePath::new(format!("node/{}", node_id.0)),
            parent,
            children: Vec::new(),
            frame: UiFrame::new(0.0, 0.0, 10.0, 10.0),
            clip_frame: UiFrame::new(0.0, 0.0, 10.0, 10.0),
            z_index: 0,
            paint_order: node_id.0,
            visibility,
            input_policy: UiInputPolicy::Receive,
            pointer_events: Default::default(),
            enabled: true,
            clickable: false,
            hoverable: false,
            focusable: false,
            clip_to_bounds: false,
            control_id: None,
            slot: None,
        }
    }
}
