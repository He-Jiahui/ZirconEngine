use std::collections::BTreeMap;

use zircon_runtime_interface::ui::{event_ui::UiNodeId, surface::UiArrangedTree};

const VISIBILITY_UNKNOWN: u8 = 0;
const VISIBILITY_VISITING: u8 = 1;
const VISIBILITY_HIDDEN: u8 = 2;
const VISIBILITY_VISIBLE: u8 = 3;
const VISIBILITY_WORD_BITS: usize = u64::BITS as usize;
const RESOLUTION_RETAINED_SCALE: usize = 2;

/// Compact inherited render visibility published with the arranged-node index.
#[derive(Debug, Default)]
pub(crate) struct UiArrangedVisibilityIndex {
    node_ids: Vec<UiNodeId>,
    render_visible_words: Vec<u64>,
    // Resolution storage is retained between arranged-tree rebuilds. It is not
    // part of the published visibility authority and is bounded to a small
    // multiple of the current node count before each rebuild.
    resolution_states: Vec<u8>,
    resolution_path: Vec<usize>,
}

impl Clone for UiArrangedVisibilityIndex {
    fn clone(&self) -> Self {
        Self {
            node_ids: self.node_ids.clone(),
            render_visible_words: self.render_visible_words.clone(),
            // Resolver scratch is deliberately not copied into a cloned Surface.
            resolution_states: Vec::new(),
            resolution_path: Vec::new(),
        }
    }
}

impl PartialEq for UiArrangedVisibilityIndex {
    fn eq(&self, other: &Self) -> bool {
        self.node_ids == other.node_ids && self.render_visible_words == other.render_visible_words
    }
}

impl Eq for UiArrangedVisibilityIndex {}

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
        self.prepare_resolution_scratch(arranged_tree.nodes.len());
        let states = &mut self.resolution_states;
        let path = &mut self.resolution_path;

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
        let node_count = node_indices.len();
        self.node_ids.reserve(node_count);
        self.render_visible_words.clear();
        self.render_visible_words.resize(
            node_count.saturating_add(VISIBILITY_WORD_BITS - 1) / VISIBILITY_WORD_BITS,
            0,
        );
        let resolution_states = &self.resolution_states;
        let node_ids = &mut self.node_ids;
        let render_visible_words = &mut self.render_visible_words;
        for (sorted_index, (node_id, arranged_index)) in node_indices.iter().enumerate() {
            node_ids.push(*node_id);
            let visible = resolution_states
                .get(*arranged_index)
                .is_some_and(|state| *state == VISIBILITY_VISIBLE);
            if visible {
                render_visible_words[sorted_index / VISIBILITY_WORD_BITS] |=
                    1_u64 << (sorted_index % VISIBILITY_WORD_BITS);
            }
        }
    }

    fn prepare_resolution_scratch(&mut self, node_count: usize) {
        let retained_capacity_budget = resolution_retained_capacity_budget(node_count);
        if self.resolution_states.capacity() > retained_capacity_budget {
            self.resolution_states = Vec::with_capacity(node_count);
        }
        self.resolution_states.clear();
        self.resolution_states
            .resize(node_count, VISIBILITY_UNKNOWN);

        if self.resolution_path.capacity() > retained_capacity_budget {
            self.resolution_path = Vec::with_capacity(node_count);
        } else {
            self.resolution_path.clear();
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

fn resolution_retained_capacity_budget(node_count: usize) -> usize {
    node_count.saturating_mul(RESOLUTION_RETAINED_SCALE)
}

fn is_resolved(state: u8) -> bool {
    matches!(state, VISIBILITY_HIDDEN | VISIBILITY_VISIBLE)
}

#[cfg(test)]
#[path = "tests/arranged_visibility.rs"]
mod tests;
