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
use crate::ui::layout::taffy_bridge::{taffy_main_axis, TaffyParentProductCache};

#[derive(Debug, Default)]
pub(crate) struct UiLayoutSlotIndex {
    state: RefCell<UiLayoutSlotIndexState>,
    workspace: RefCell<UiLayoutPassWorkspace>,
    taffy_parent_products: RefCell<TaffyParentProductCache>,
    empty_ordered_children: Arc<[UiNodeId]>,
    pub(super) virtual_lists: RefCell<UiMaterializedVirtualListLayoutIndex>,
}

impl Clone for UiLayoutSlotIndex {
    fn clone(&self) -> Self {
        Self {
            state: RefCell::new(self.state.borrow().clone()),
            workspace: RefCell::default(),
            taffy_parent_products: RefCell::default(),
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
    ordered_children_revision_cursor: u64,
    parent_by_child: BTreeMap<UiNodeId, UiNodeId>,
    ordered_children_by_parent: BTreeMap<UiNodeId, UiOrderedChildren>,
    parent_size_dependencies_by_parent: BTreeMap<UiNodeId, UiParentSizeDependencies>,
    responsive_candidates: MuiResponsiveCandidates,
    #[cfg(test)]
    parent_size_dependency_evaluations: usize,
}

#[derive(Clone, Debug)]
struct UiOrderedChildren {
    revision: u64,
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
        self.taffy_parent_products.borrow_mut().retain(|parent_id| {
            tree.node(parent_id)
                .is_some_and(|parent| taffy_main_axis(parent.container).is_some())
        });
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
        let mut products = self.taffy_parent_products.borrow_mut();
        for node_id in node_ids.iter().copied() {
            if tree
                .node(node_id)
                .is_none_or(|node| taffy_main_axis(node.container).is_none())
            {
                products.discard(node_id);
            }
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

    pub(super) fn ordered_children_revision_for_container(
        &self,
        tree: &UiTree,
        parent_id: UiNodeId,
        container: UiContainerKind,
    ) -> u64 {
        self.ensure_initialized(tree);
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
            .map_or(0, |ordered| ordered.revision)
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

    pub(super) fn with_taffy_parent_products<T>(
        &self,
        update: impl FnOnce(&mut TaffyParentProductCache) -> T,
    ) -> T {
        update(&mut self.taffy_parent_products.borrow_mut())
    }

    pub(super) fn discard_taffy_parent_product(&self, parent_id: UiNodeId) {
        self.taffy_parent_products.borrow_mut().discard(parent_id);
    }

    pub(super) fn taffy_parent_product_matches_order_revision(
        &self,
        parent_id: UiNodeId,
        revision: u64,
    ) -> bool {
        self.taffy_parent_products
            .borrow()
            .matches_order_revision(parent_id, revision)
    }

    pub(super) fn taffy_active_children_contain_all(
        &self,
        parent_id: UiNodeId,
        node_ids: &[UiNodeId],
    ) -> bool {
        self.taffy_parent_products
            .borrow()
            .active_children_contain_all(parent_id, node_ids)
    }

    #[cfg(test)]
    pub(super) fn taffy_parent_product_counts(&self) -> (usize, usize) {
        self.taffy_parent_products.borrow().product_counts()
    }

    #[cfg(test)]
    pub(super) fn taffy_parent_product_last_update(
        &self,
        parent_id: UiNodeId,
    ) -> Option<crate::ui::layout::taffy_bridge::TaffyParentProductUpdate> {
        self.taffy_parent_products.borrow().last_update(parent_id)
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
        self.ordered_children_revision_cursor =
            self.ordered_children_revision_cursor.wrapping_add(1).max(1);
        self.ordered_children_by_parent.insert(
            parent_id,
            UiOrderedChildren {
                revision: self.ordered_children_revision_cursor,
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
#[path = "tests/slot.rs"]
mod tests;
