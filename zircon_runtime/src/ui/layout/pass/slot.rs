use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{StretchMode, UiContainerKind, UiMargin, UiSlot},
    tree::UiTree,
};

use super::responsive_mui::MuiResponsiveCandidates;
use super::virtual_list_layout::UiMaterializedVirtualListLayoutIndex;
use super::workspace::{UiArrangeChildScratch, UiLayoutPassWorkspace};

#[derive(Debug, Default)]
pub(crate) struct UiLayoutSlotIndex {
    state: RefCell<UiLayoutSlotIndexState>,
    workspace: RefCell<UiLayoutPassWorkspace>,
    empty_ordered_children: Arc<[UiNodeId]>,
    pub(super) virtual_lists: RefCell<UiMaterializedVirtualListLayoutIndex>,
}

impl Clone for UiLayoutSlotIndex {
    fn clone(&self) -> Self {
        Self {
            state: RefCell::new(self.state.borrow().clone()),
            workspace: RefCell::default(),
            empty_ordered_children: Arc::clone(&self.empty_ordered_children),
            virtual_lists: RefCell::new(self.virtual_lists.borrow().clone()),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct UiLayoutSlotIndexState {
    initialized: bool,
    node_count: usize,
    slot_count: usize,
    layout_order_generation: u64,
    parent_by_child: BTreeMap<UiNodeId, UiNodeId>,
    ordered_children_by_parent: BTreeMap<UiNodeId, UiOrderedChildren>,
    parent_size_dependencies_by_parent: BTreeMap<UiNodeId, UiParentSizeDependencies>,
    responsive_candidates: MuiResponsiveCandidates,
    #[cfg(test)]
    parent_size_dependency_evaluations: usize,
}

#[derive(Clone, Debug)]
struct UiOrderedChildren {
    container: UiContainerKind,
    tree_children: Arc<[UiNodeId]>,
    ordered_children: Arc<[UiNodeId]>,
}

#[derive(Clone, Debug)]
struct UiParentSizeDependencies {
    container: UiContainerKind,
    tree_children: Arc<[UiNodeId]>,
    position_by_child: BTreeMap<UiNodeId, usize>,
    dependent_children_by_position: BTreeMap<usize, UiNodeId>,
}

impl UiLayoutSlotIndex {
    pub(super) fn for_tree(tree: &UiTree) -> Self {
        let index = Self::default();
        index.refresh_for_tree(tree);
        index
    }

    pub(crate) fn refresh_for_tree(&self, tree: &UiTree) {
        self.state.borrow_mut().rebuild(tree);
    }

    pub(super) fn ensure_initialized(&self, tree: &UiTree) {
        let (needs_rebuild, needs_order_patch) = {
            let state = self.state.borrow();
            let shape_changed = state.node_count != tree.nodes.len()
                || state.slot_count != tree.layout_slots().len();
            let generation_changed =
                state.layout_order_generation != tree.layout_order_generation();
            let has_patch_authority = !tree.pending_layout_order_parent_ids().is_empty();
            (
                !state.initialized
                    || ((shape_changed || generation_changed) && !has_patch_authority),
                state.initialized && (shape_changed || generation_changed) && has_patch_authority,
            )
        };
        if needs_rebuild {
            self.refresh_for_tree(tree);
        } else if needs_order_patch {
            let parent_ids = tree.pending_layout_order_parent_ids().clone();
            self.state
                .borrow_mut()
                .patch_layout_order_parents(tree, &parent_ids);
        }
    }

    pub(super) fn synchronize_ordered_children(
        &self,
        tree: &UiTree,
        node_ids: &BTreeSet<UiNodeId>,
    ) {
        self.ensure_initialized(tree);
        let parent_ids = {
            let state = self.state.borrow();
            node_ids
                .iter()
                .copied()
                .filter(|node_id| {
                    let Some(node) = tree.node(*node_id) else {
                        return state.ordered_children_by_parent.contains_key(node_id);
                    };
                    state
                        .ordered_children_by_parent
                        .get(node_id)
                        .is_none_or(|ordered| {
                            ordered.container != node.container
                                || ordered.tree_children.as_ref() != node.children.as_slice()
                        })
                })
                .collect::<BTreeSet<_>>()
        };
        if !parent_ids.is_empty() {
            self.state
                .borrow_mut()
                .patch_layout_order_parents(tree, &parent_ids);
        }
    }

    pub(super) fn ordered_children_for_container(
        &self,
        tree: &UiTree,
        parent_id: UiNodeId,
        container: UiContainerKind,
    ) -> Arc<[UiNodeId]> {
        self.ensure_initialized(tree);
        if tree
            .node(parent_id)
            .is_none_or(|parent| parent.children.is_empty())
        {
            return Arc::clone(&self.empty_ordered_children);
        }
        let needs_refresh = self
            .state
            .borrow()
            .ordered_children_by_parent
            .get(&parent_id)
            .is_none_or(|ordered| ordered.container != container);
        if needs_refresh {
            self.state
                .borrow_mut()
                .patch_layout_order_parents(tree, &BTreeSet::from([parent_id]));
        }
        self.state
            .borrow()
            .ordered_children_by_parent
            .get(&parent_id)
            .map(|ordered| Arc::clone(&ordered.ordered_children))
            .unwrap_or_else(|| Arc::clone(&self.empty_ordered_children))
    }

    pub(super) fn synchronize_responsive_candidates(
        &self,
        tree: &UiTree,
        node_ids: &std::collections::BTreeSet<UiNodeId>,
    ) {
        self.ensure_initialized(tree);
        self.state
            .borrow_mut()
            .responsive_candidates
            .patch_nodes(tree, node_ids);
    }

    pub(super) fn responsive_layout_may_change(&self, width: f32) -> bool {
        self.state
            .borrow_mut()
            .responsive_candidates
            .responsive_layout_may_change(width)
    }

    pub(super) fn synchronize_parent_size_dependencies(
        &self,
        tree: &UiTree,
        node_ids: &BTreeSet<UiNodeId>,
    ) {
        self.ensure_initialized(tree);
        self.state
            .borrow_mut()
            .patch_parent_size_dependencies(tree, node_ids);
    }

    pub(super) fn copy_parent_size_dependent_children(
        &self,
        tree: &UiTree,
        parent_id: UiNodeId,
        output: &mut Vec<UiNodeId>,
    ) {
        self.ensure_initialized(tree);
        output.clear();
        let state = self.state.borrow();
        if let Some(dependencies) = state.parent_size_dependencies_by_parent.get(&parent_id) {
            output.extend(
                dependencies
                    .dependent_children_by_position
                    .values()
                    .copied(),
            );
        } else if let Some(parent) = tree.node(parent_id) {
            output.extend_from_slice(&parent.children);
        }
    }

    #[cfg(test)]
    fn parent_size_dependency_evaluations(&self) -> usize {
        self.state.borrow().parent_size_dependency_evaluations
    }

    pub(super) fn with_responsive_candidates<T>(
        &self,
        read: impl FnOnce(&MuiResponsiveCandidates) -> T,
    ) -> T {
        read(&self.state.borrow().responsive_candidates)
    }

    pub(super) fn with_measure_workspace<T>(
        &self,
        measure: impl FnOnce(&mut UiLayoutPassWorkspace) -> T,
    ) -> T {
        let mut workspace = self.workspace.borrow_mut();
        measure(&mut workspace)
    }

    pub(super) fn take_arrange_child_scratch(&self) -> UiArrangeChildScratch {
        self.workspace
            .borrow_mut()
            .arrange_child_pool
            .pop()
            .unwrap_or_default()
    }

    pub(super) fn recycle_arrange_child_scratch(&self, mut scratch: UiArrangeChildScratch) {
        scratch.children.clear();
        scratch.linear.constraints.clear();
        scratch.linear.resolved.clear();
        scratch.linear.priorities.clear();
        scratch.linear.active_indices.clear();
        scratch.wrap_row_items.clear();
        scratch.wrap_content_desired.clear();
        scratch.masonry.column_heights.clear();
        scratch.masonry.column_counts.clear();
        self.workspace.borrow_mut().arrange_child_pool.push(scratch);
    }

    pub(super) fn take_hidden_subtree_stack(&self) -> Vec<UiNodeId> {
        std::mem::take(&mut self.workspace.borrow_mut().hidden_subtree_stack)
    }

    pub(super) fn recycle_hidden_subtree_stack(&self, mut stack: Vec<UiNodeId>) {
        stack.clear();
        self.workspace.borrow_mut().hidden_subtree_stack = stack;
    }

    pub(super) fn first_index_for_edge(
        &self,
        tree: &UiTree,
        parent_id: UiNodeId,
        child_id: UiNodeId,
    ) -> Option<usize> {
        tree.first_layout_slot_index_for_edge(parent_id, child_id)
    }

    pub(crate) fn index_for_kind(
        &self,
        tree: &UiTree,
        parent_id: UiNodeId,
        child_id: UiNodeId,
        kind: zircon_runtime_interface::ui::layout::UiSlotKind,
    ) -> Option<usize> {
        tree.layout_slot_index_for_edge_kind(parent_id, child_id, kind)
    }
}

impl UiLayoutSlotIndexState {
    fn rebuild(&mut self, tree: &UiTree) {
        let mut parent_by_child = BTreeMap::new();
        for (parent_id, parent) in &tree.nodes {
            for child_id in &parent.children {
                parent_by_child.insert(*child_id, *parent_id);
            }
        }
        self.parent_by_child = parent_by_child;
        self.ordered_children_by_parent.clear();
        for parent_id in tree.nodes.keys().copied() {
            self.rebuild_ordered_children(tree, parent_id);
        }
        self.parent_size_dependencies_by_parent.clear();
        #[cfg(test)]
        {
            self.parent_size_dependency_evaluations = 0;
        }
        for parent_id in tree.nodes.keys().copied() {
            self.rebuild_parent_size_dependencies(tree, parent_id);
        }
        self.responsive_candidates = MuiResponsiveCandidates::for_tree(tree);
        self.node_count = tree.nodes.len();
        self.slot_count = tree.layout_slots().len();
        self.layout_order_generation = tree.layout_order_generation();
        self.initialized = true;
    }

    fn patch_layout_order_parents(&mut self, tree: &UiTree, parent_ids: &BTreeSet<UiNodeId>) {
        if parent_ids.is_empty() {
            return;
        }
        for parent_id in parent_ids.iter().copied() {
            if let Some(previous) = self.ordered_children_by_parent.remove(&parent_id) {
                for child_id in previous.tree_children.iter().copied() {
                    if self.parent_by_child.get(&child_id) == Some(&parent_id) {
                        self.parent_by_child.remove(&child_id);
                    }
                }
            }
            if let Some(parent) = tree.node(parent_id) {
                for child_id in parent.children.iter().copied() {
                    self.parent_by_child.insert(child_id, parent_id);
                }
            }
        }
        for parent_id in parent_ids.iter().copied() {
            self.rebuild_ordered_children(tree, parent_id);
        }
        self.node_count = tree.nodes.len();
        self.slot_count = tree.layout_slots().len();
        self.layout_order_generation = tree.layout_order_generation();
    }

    fn rebuild_ordered_children(&mut self, tree: &UiTree, parent_id: UiNodeId) {
        let Some(parent) = tree.node(parent_id) else {
            self.ordered_children_by_parent.remove(&parent_id);
            return;
        };
        if parent.children.is_empty() {
            self.ordered_children_by_parent.remove(&parent_id);
            return;
        }
        let container = parent.container;
        let tree_children = Arc::<[UiNodeId]>::from(parent.children.clone());
        let ordered_children = if container_uses_slot_order(container) {
            let mut entries = parent
                .children
                .iter()
                .copied()
                .enumerate()
                .map(|(index, child_id)| {
                    let order =
                        indexed_slot_for_container_child(tree, parent_id, child_id, container)
                            .map(|slot| slot.order)
                            .unwrap_or_default();
                    (order, index, child_id)
                })
                .collect::<Vec<_>>();
            entries.sort_unstable_by_key(|(order, index, _)| (*order, *index));
            Arc::from(
                entries
                    .into_iter()
                    .map(|(_, _, child_id)| child_id)
                    .collect::<Vec<_>>(),
            )
        } else {
            Arc::clone(&tree_children)
        };
        self.ordered_children_by_parent.insert(
            parent_id,
            UiOrderedChildren {
                container,
                tree_children,
                ordered_children,
            },
        );
    }

    fn patch_parent_size_dependencies(&mut self, tree: &UiTree, node_ids: &BTreeSet<UiNodeId>) {
        let mut rebuild_parent_ids = tree.pending_layout_order_parent_ids().clone();
        for node_id in node_ids.iter().copied() {
            let previous_parent_id = self.parent_by_child.get(&node_id).copied();
            let current_parent_id = tree.node(node_id).and_then(|node| node.parent);
            if previous_parent_id != current_parent_id {
                rebuild_parent_ids.extend(previous_parent_id);
                rebuild_parent_ids.extend(current_parent_id);
            }
            match current_parent_id {
                Some(parent_id) => {
                    self.parent_by_child.insert(node_id, parent_id);
                }
                None => {
                    self.parent_by_child.remove(&node_id);
                }
            }
            if self.parent_size_dependencies_need_rebuild(tree, node_id) {
                rebuild_parent_ids.insert(node_id);
            }
        }

        for parent_id in rebuild_parent_ids.iter().copied() {
            self.rebuild_parent_size_dependencies(tree, parent_id);
        }

        for child_id in node_ids.iter().copied() {
            let Some(parent_id) = tree.node(child_id).and_then(|node| node.parent) else {
                continue;
            };
            if rebuild_parent_ids.contains(&parent_id) {
                continue;
            }
            if !self.patch_parent_size_dependency_child(tree, parent_id, child_id) {
                self.rebuild_parent_size_dependencies(tree, parent_id);
                rebuild_parent_ids.insert(parent_id);
            }
        }
    }

    fn parent_size_dependencies_need_rebuild(&self, tree: &UiTree, parent_id: UiNodeId) -> bool {
        match (
            tree.node(parent_id),
            self.parent_size_dependencies_by_parent.get(&parent_id),
        ) {
            (Some(parent), Some(dependencies)) => {
                dependencies.container != parent.container
                    || dependencies.tree_children.as_ref() != parent.children.as_slice()
            }
            (Some(parent), None) => !parent.children.is_empty(),
            (None, Some(_)) => true,
            (None, None) => false,
        }
    }

    fn patch_parent_size_dependency_child(
        &mut self,
        tree: &UiTree,
        parent_id: UiNodeId,
        child_id: UiNodeId,
    ) -> bool {
        let Some(parent) = tree.node(parent_id) else {
            return false;
        };
        let Some(position) = self
            .parent_size_dependencies_by_parent
            .get(&parent_id)
            .filter(|dependencies| {
                dependencies.container == parent.container
                    && dependencies.tree_children.as_ref() == parent.children.as_slice()
            })
            .and_then(|dependencies| dependencies.position_by_child.get(&child_id))
            .copied()
        else {
            return false;
        };

        #[cfg(test)]
        {
            self.parent_size_dependency_evaluations =
                self.parent_size_dependency_evaluations.saturating_add(1);
        }
        let depends_on_parent =
            free_child_depends_on_parent_size(tree, parent_id, child_id, parent.container);
        let dependencies = self
            .parent_size_dependencies_by_parent
            .get_mut(&parent_id)
            .expect("validated parent dependency record");
        dependencies
            .dependent_children_by_position
            .remove(&position);
        if depends_on_parent {
            dependencies
                .dependent_children_by_position
                .insert(position, child_id);
        }
        true
    }

    fn rebuild_parent_size_dependencies(&mut self, tree: &UiTree, parent_id: UiNodeId) {
        let Some(parent) = tree.node(parent_id) else {
            self.parent_size_dependencies_by_parent.remove(&parent_id);
            return;
        };
        let mut position_by_child = BTreeMap::new();
        let mut dependent_children_by_position = BTreeMap::new();
        for (position, child_id) in parent.children.iter().copied().enumerate() {
            position_by_child.insert(child_id, position);
            #[cfg(test)]
            {
                self.parent_size_dependency_evaluations =
                    self.parent_size_dependency_evaluations.saturating_add(1);
            }
            if free_child_depends_on_parent_size(tree, parent_id, child_id, parent.container) {
                dependent_children_by_position.insert(position, child_id);
            }
        }
        self.parent_size_dependencies_by_parent.insert(
            parent_id,
            UiParentSizeDependencies {
                container: parent.container,
                tree_children: Arc::from(parent.children.clone()),
                position_by_child,
                dependent_children_by_position,
            },
        );
    }
}

fn free_child_depends_on_parent_size(
    tree: &UiTree,
    parent_id: UiNodeId,
    child_id: UiNodeId,
    container: UiContainerKind,
) -> bool {
    let Some(child) = tree.node(child_id) else {
        return true;
    };
    let slot = container.child_slot_kind().and_then(|kind| {
        tree.layout_slot_index_for_edge_kind(parent_id, child_id, kind)
            .and_then(|slot_index| tree.layout_slot(slot_index))
    });
    if slot.is_some_and(|slot| slot.canvas_placement.is_some() || has_slot_frame_policy(Some(slot)))
    {
        return true;
    }
    child.anchor.x != 0.0
        || child.anchor.y != 0.0
        || child.constraints.width.stretch_mode == StretchMode::Stretch
        || child.constraints.height.stretch_mode == StretchMode::Stretch
}

// Parent-local layout projections are derived caches and do not contribute to surface identity.
impl PartialEq for UiLayoutSlotIndex {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

pub(super) fn slot_for_container_child<'a>(
    tree: &'a UiTree,
    slot_index: &UiLayoutSlotIndex,
    parent_id: UiNodeId,
    child_id: UiNodeId,
    container: UiContainerKind,
) -> Option<&'a UiSlot> {
    let slot_kind = slot_kind_for_container(container)?;
    tree.layout_slot(slot_index.index_for_kind(tree, parent_id, child_id, slot_kind)?)
}

fn container_uses_slot_order(container: UiContainerKind) -> bool {
    matches!(
        container,
        UiContainerKind::Free
            | UiContainerKind::Canvas
            | UiContainerKind::Container
            | UiContainerKind::Overlay
            | UiContainerKind::BlockBox
            | UiContainerKind::SizeBox(_)
            | UiContainerKind::HorizontalBox(_)
            | UiContainerKind::VerticalBox(_)
            | UiContainerKind::WrapBox(_)
            | UiContainerKind::GridBox(_)
            | UiContainerKind::MasonryBox(_)
    )
}

pub(super) fn has_slot_frame_policy(slot: Option<&UiSlot>) -> bool {
    slot.is_some_and(|slot| {
        slot.padding != UiMargin::default() || slot.alignment != Default::default()
    })
}

pub(super) fn slot_padding(slot: Option<&UiSlot>) -> UiMargin {
    slot.filter(|slot| slot.padding != UiMargin::default())
        .map(|slot| slot.padding)
        .unwrap_or_default()
}

fn slot_kind_for_container(
    container: UiContainerKind,
) -> Option<zircon_runtime_interface::ui::layout::UiSlotKind> {
    container.child_slot_kind()
}

fn indexed_slot_for_container_child<'a>(
    tree: &'a UiTree,
    parent_id: UiNodeId,
    child_id: UiNodeId,
    container: UiContainerKind,
) -> Option<&'a UiSlot> {
    let slot_kind = slot_kind_for_container(container)?;
    tree.layout_slot(tree.layout_slot_index_for_edge_kind(parent_id, child_id, slot_kind)?)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::sync::Arc;

    use zircon_runtime_interface::ui::{
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        layout::{Anchor, StretchMode, UiContainerKind, UiSlot, UiSlotKind},
        tree::{UiTree, UiTreeNode},
    };

    use super::{slot_for_container_child, UiLayoutSlotIndex};

    #[test]
    fn indexed_lookup_preserves_first_matching_slot_semantics() {
        let parent_id = UiNodeId::new(1);
        let child_id = UiNodeId::new(2);
        let mut tree = UiTree::default();
        tree.replace_layout_slots(vec![
            UiSlot::new(parent_id, child_id, UiSlotKind::Free).with_order(9),
            UiSlot::new(parent_id, child_id, UiSlotKind::Linear).with_order(1),
            UiSlot::new(parent_id, child_id, UiSlotKind::Linear).with_order(2),
        ]);
        let slot_index = UiLayoutSlotIndex::for_tree(&tree);

        let slot = slot_for_container_child(
            &tree,
            &slot_index,
            parent_id,
            child_id,
            UiContainerKind::HorizontalBox(Default::default()),
        )
        .expect("linear slot should be indexed");

        assert_eq!(slot.order, 1);
    }

    #[test]
    fn indexed_lookup_repairs_same_cardinality_edge_replacement() {
        let parent_id = UiNodeId::new(1);
        let old_child_id = UiNodeId::new(2);
        let next_child_id = UiNodeId::new(3);
        let mut tree = UiTree::default();
        tree.replace_layout_slots(vec![UiSlot::new(
            parent_id,
            old_child_id,
            UiSlotKind::Linear,
        )]);
        let slot_index = UiLayoutSlotIndex::for_tree(&tree);

        tree.replace_layout_slots(vec![UiSlot::new(
            parent_id,
            next_child_id,
            UiSlotKind::Linear,
        )
        .with_order(7)]);

        let slot = slot_for_container_child(
            &tree,
            &slot_index,
            parent_id,
            next_child_id,
            UiContainerKind::HorizontalBox(Default::default()),
        )
        .expect("replacement edge should repair the cached slot lookup");
        assert_eq!(slot.order, 7);
        assert!(slot_for_container_child(
            &tree,
            &slot_index,
            parent_id,
            old_child_id,
            UiContainerKind::HorizontalBox(Default::default()),
        )
        .is_none());
    }

    #[test]
    fn ordered_children_reuse_one_generation_and_patch_same_cardinality_order_changes() {
        let parent_id = UiNodeId::new(1);
        let first_child_id = UiNodeId::new(2);
        let second_child_id = UiNodeId::new(3);
        let container = UiContainerKind::HorizontalBox(Default::default());
        let mut tree = UiTree::new(UiTreeId::new("layout.order.generation"));
        tree.insert_root(
            UiTreeNode::new(parent_id, UiNodePath::new("root")).with_container(container),
        );
        tree.insert_child(
            parent_id,
            UiTreeNode::new(first_child_id, UiNodePath::new("root.first")),
        )
        .expect("insert first child");
        tree.insert_child(
            parent_id,
            UiTreeNode::new(second_child_id, UiNodePath::new("root.second")),
        )
        .expect("insert second child");
        tree.push_layout_slot(
            UiSlot::new(parent_id, first_child_id, UiSlotKind::Linear).with_order(1),
        );
        tree.push_layout_slot(
            UiSlot::new(parent_id, second_child_id, UiSlotKind::Linear).with_order(0),
        );
        let slot_index = UiLayoutSlotIndex::for_tree(&tree);

        let first = slot_index.ordered_children_for_container(&tree, parent_id, container);
        let stable = slot_index.ordered_children_for_container(&tree, parent_id, container);
        assert_eq!(first.as_ref(), &[second_child_id, first_child_id]);
        assert!(Arc::ptr_eq(&first, &stable));

        tree.mutate_layout_slot(0, |slot| slot.order = -1)
            .expect("mutate first slot order");
        let changed = slot_index.ordered_children_for_container(&tree, parent_id, container);
        let changed_stable = slot_index.ordered_children_for_container(&tree, parent_id, container);

        assert_eq!(changed.as_ref(), &[first_child_id, second_child_id]);
        assert!(!Arc::ptr_eq(&first, &changed));
        assert!(Arc::ptr_eq(&changed, &changed_stable));
    }

    #[test]
    fn ordered_children_patch_same_cardinality_public_child_reordering() {
        let parent_id = UiNodeId::new(1);
        let first_child_id = UiNodeId::new(2);
        let second_child_id = UiNodeId::new(3);
        let container = UiContainerKind::ScrollableBox(Default::default());
        let mut tree = UiTree::new(UiTreeId::new("layout.order.defensive-repair"));
        tree.insert_root(
            UiTreeNode::new(parent_id, UiNodePath::new("root")).with_container(container),
        );
        tree.insert_child(
            parent_id,
            UiTreeNode::new(first_child_id, UiNodePath::new("root.first")),
        )
        .expect("insert first child");
        tree.insert_child(
            parent_id,
            UiTreeNode::new(second_child_id, UiNodePath::new("root.second")),
        )
        .expect("insert second child");
        let slot_index = UiLayoutSlotIndex::for_tree(&tree);
        let first = slot_index.ordered_children_for_container(&tree, parent_id, container);

        tree.node_mut(parent_id)
            .expect("parent")
            .children
            .swap(0, 1);
        slot_index.synchronize_ordered_children(&tree, &BTreeSet::from([parent_id]));
        let changed = slot_index.ordered_children_for_container(&tree, parent_id, container);

        assert_eq!(first.as_ref(), &[first_child_id, second_child_id]);
        assert_eq!(changed.as_ref(), &[second_child_id, first_child_id]);
        assert!(!Arc::ptr_eq(&first, &changed));
    }

    #[test]
    fn one_child_dependency_change_patches_only_that_parent_membership() {
        const CHILD_COUNT: u64 = 1_000;
        let parent_id = UiNodeId::new(1);
        let changed_child_id = UiNodeId::new(778);
        let mut tree = UiTree::new(UiTreeId::new("layout.dependencies.exact-child"));
        tree.insert_root(
            UiTreeNode::new(parent_id, UiNodePath::new("root"))
                .with_container(UiContainerKind::Free),
        );
        for child_index in 0..CHILD_COUNT {
            let child_id = UiNodeId::new(child_index + 2);
            let mut child = UiTreeNode::new(
                child_id,
                UiNodePath::new(format!("root.child-{child_index}")),
            );
            child.constraints.width.stretch_mode = StretchMode::Fixed;
            child.constraints.height.stretch_mode = StretchMode::Fixed;
            tree.insert_child(parent_id, child).expect("insert child");
        }
        let slot_index = UiLayoutSlotIndex::for_tree(&tree);
        tree.clear_pending_mutation_node_ids();
        let initial_evaluations = slot_index.parent_size_dependency_evaluations();

        tree.node_mut(changed_child_id)
            .expect("changed child")
            .anchor = Anchor::new(0.5, 0.0);
        slot_index.synchronize_parent_size_dependencies(&tree, &BTreeSet::from([changed_child_id]));

        assert_eq!(
            slot_index.parent_size_dependency_evaluations() - initial_evaluations,
            1
        );
        let mut dependent_children = Vec::new();
        slot_index.copy_parent_size_dependent_children(&tree, parent_id, &mut dependent_children);
        assert_eq!(dependent_children, vec![changed_child_id]);

        let before_removal = slot_index.parent_size_dependency_evaluations();
        tree.node_mut(changed_child_id)
            .expect("changed child")
            .anchor = Anchor::default();
        slot_index.synchronize_parent_size_dependencies(&tree, &BTreeSet::from([changed_child_id]));

        assert_eq!(
            slot_index.parent_size_dependency_evaluations() - before_removal,
            1
        );
        slot_index.copy_parent_size_dependent_children(&tree, parent_id, &mut dependent_children);
        assert!(dependent_children.is_empty());
    }

    #[test]
    fn parent_container_change_rebuilds_the_parent_dependency_projection() {
        const CHILD_COUNT: u64 = 64;
        let parent_id = UiNodeId::new(1);
        let mut tree = UiTree::new(UiTreeId::new("layout.dependencies.container-fallback"));
        tree.insert_root(
            UiTreeNode::new(parent_id, UiNodePath::new("root"))
                .with_container(UiContainerKind::Free),
        );
        for child_index in 0..CHILD_COUNT {
            let child_id = UiNodeId::new(child_index + 2);
            tree.insert_child(
                parent_id,
                UiTreeNode::new(
                    child_id,
                    UiNodePath::new(format!("root.child-{child_index}")),
                ),
            )
            .expect("insert child");
        }
        let slot_index = UiLayoutSlotIndex::for_tree(&tree);
        tree.clear_pending_mutation_node_ids();
        let initial_evaluations = slot_index.parent_size_dependency_evaluations();

        tree.node_mut(parent_id).expect("parent").container = UiContainerKind::Container;
        slot_index.synchronize_parent_size_dependencies(&tree, &BTreeSet::from([parent_id]));

        assert_eq!(
            slot_index.parent_size_dependency_evaluations() - initial_evaluations,
            CHILD_COUNT as usize
        );
    }
}
