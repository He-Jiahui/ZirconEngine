use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiNodePath},
    layout::{UiSlot, UiSlotKind},
    tree::{UiTree, UiTreeError, UiTreeNode},
};

use super::surface::UiSurface;
use super::virtual_list_materialization::UiVirtualListMaterializationError;

/// Rebuildable topology cache for bounded virtual-list row subtrees.
#[derive(Clone, Debug, Default)]
pub(super) struct UiVirtualListPrototypePoolIndex {
    owners: BTreeMap<UiNodeId, UiVirtualListPrototypePoolState>,
}

#[derive(Clone, Debug)]
struct UiVirtualListPrototypePoolState {
    blueprint: UiVirtualListPrototypeBlueprint,
    slot_root_ids: Vec<UiNodeId>,
}

#[derive(Clone, Debug)]
struct UiVirtualListPrototypeBlueprint {
    root_id: UiNodeId,
    prototype_node_ids: Vec<UiNodeId>,
    nodes: Vec<UiTreeNode>,
    root_slot: UiSlot,
    internal_slots: Vec<UiSlot>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiVirtualListPrototypeNodeContext {
    pub owner_id: UiNodeId,
    pub slot_index: usize,
    pub prototype_node_id: UiNodeId,
    pub node_id: UiNodeId,
    pub is_slot_root: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiVirtualListPrototypePoolReport {
    pub owner_id: UiNodeId,
    pub slot_capacity: usize,
    pub live_slot_count: usize,
    pub created_slot_count: usize,
    pub removed_slot_count: usize,
    pub created_node_count: usize,
    pub reused_node_count: usize,
    pub recycled_node_count: usize,
    pub slot_root_ids: Vec<UiNodeId>,
}

#[derive(Debug, Error)]
pub enum UiVirtualListPrototypePoolError {
    #[error("virtual-list owner {owner_id:?} has no reconciled materialization assignment")]
    MissingAssignmentState { owner_id: UiNodeId },
    #[error("virtual-list prototype root {prototype_root_id:?} is missing")]
    MissingPrototype { prototype_root_id: UiNodeId },
    #[error(
        "virtual-list prototype root {prototype_root_id:?} is not a direct child of owner {owner_id:?}"
    )]
    InvalidPrototypeRoot {
        owner_id: UiNodeId,
        prototype_root_id: UiNodeId,
    },
    #[error("virtual-list prototype subtree contains a duplicate or cyclic node {node_id:?}")]
    InvalidPrototypeTopology { node_id: UiNodeId },
    #[error("virtual-list prototype pool exhausted UiNodeId space")]
    NodeIdExhausted,
    #[error(transparent)]
    Materialization(#[from] UiVirtualListMaterializationError),
    #[error(transparent)]
    Tree(#[from] UiTreeError),
}

impl UiSurface {
    /// Ensures one complete reusable prototype subtree per physical materialization slot.
    ///
    /// `configure_clone` is invoked only while a new physical subtree is created. Logical model
    /// data should be applied separately from `UiVirtualListMaterializationChange`, so scrolling
    /// touches only the slots whose logical assignment changed.
    pub fn ensure_virtual_list_prototype_slots<F>(
        &mut self,
        owner_id: UiNodeId,
        prototype_root_id: UiNodeId,
        mut configure_clone: F,
    ) -> Result<UiVirtualListPrototypePoolReport, UiVirtualListPrototypePoolError>
    where
        F: FnMut(&mut UiTreeNode, UiVirtualListPrototypeNodeContext),
    {
        let slot_capacity = self
            .virtual_list_slot_map(owner_id)
            .ok_or(UiVirtualListPrototypePoolError::MissingAssignmentState { owner_id })?
            .slot_count();
        let mut topology_changed = false;
        let mut state = match self.virtual_list_prototype_pool.remove(owner_id) {
            Some(state) => state,
            None => {
                topology_changed = true;
                UiVirtualListPrototypePoolState::capture(&self.tree, owner_id, prototype_root_id)?
            }
        };
        let result = reconcile_prototype_slot_capacity(
            self,
            owner_id,
            slot_capacity,
            topology_changed,
            &mut state,
            &mut configure_clone,
        );
        self.virtual_list_prototype_pool.insert(owner_id, state);
        result
    }

    pub fn virtual_list_prototype_slot_roots(&self, owner_id: UiNodeId) -> Option<&[UiNodeId]> {
        self.virtual_list_prototype_pool.slot_roots(owner_id)
    }

    pub fn virtual_list_prototype_root_id(&self, owner_id: UiNodeId) -> Option<UiNodeId> {
        self.virtual_list_prototype_pool.prototype_root_id(owner_id)
    }
}

fn reconcile_prototype_slot_capacity<F>(
    surface: &mut UiSurface,
    owner_id: UiNodeId,
    slot_capacity: usize,
    mut topology_changed: bool,
    state: &mut UiVirtualListPrototypePoolState,
    configure_clone: &mut F,
) -> Result<UiVirtualListPrototypePoolReport, UiVirtualListPrototypePoolError>
where
    F: FnMut(&mut UiTreeNode, UiVirtualListPrototypeNodeContext),
{
    let mut report = UiVirtualListPrototypePoolReport {
        owner_id,
        slot_capacity,
        ..UiVirtualListPrototypePoolReport::default()
    };

    if state.slot_root_ids.len() < slot_capacity {
        let missing_slot_count = slot_capacity - state.slot_root_ids.len();
        let missing_node_count = state
            .blueprint
            .nodes
            .len()
            .checked_mul(missing_slot_count)
            .ok_or(UiVirtualListPrototypePoolError::NodeIdExhausted)?;
        let mut next_node_id = next_node_id_cursor(&surface.tree, missing_node_count)?;
        while state.slot_root_ids.len() < slot_capacity {
            let slot_index = state.slot_root_ids.len();
            let clone = clone_slot_subtree(
                surface,
                &state.blueprint,
                owner_id,
                slot_index,
                &mut next_node_id,
                configure_clone,
            )?;
            state.slot_root_ids.push(clone.root_id);
            report.created_slot_count += 1;
            report.created_node_count += clone.created_node_count;
            report.reused_node_count += clone.reused_node_count;
            topology_changed = true;
        }
    }

    while state.slot_root_ids.len() > slot_capacity {
        let slot_root_id = *state
            .slot_root_ids
            .last()
            .expect("checked non-empty slot root list");
        let node_pool_report = surface.detach_subtree_to_pool(slot_root_id)?;
        state.slot_root_ids.pop();
        report.removed_slot_count += 1;
        report.recycled_node_count += node_pool_report.recycled_count;
        topology_changed = true;
    }

    if topology_changed {
        surface.register_virtual_list_slots(owner_id, &state.slot_root_ids)?;
    }
    report.live_slot_count = state.slot_root_ids.len();
    report.slot_root_ids.clone_from(&state.slot_root_ids);
    Ok(report)
}

impl UiVirtualListPrototypePoolIndex {
    fn remove(&mut self, owner_id: UiNodeId) -> Option<UiVirtualListPrototypePoolState> {
        self.owners.remove(&owner_id)
    }

    fn insert(&mut self, owner_id: UiNodeId, state: UiVirtualListPrototypePoolState) {
        self.owners.insert(owner_id, state);
    }

    fn slot_roots(&self, owner_id: UiNodeId) -> Option<&[UiNodeId]> {
        self.owners
            .get(&owner_id)
            .map(|state| state.slot_root_ids.as_slice())
    }

    fn prototype_root_id(&self, owner_id: UiNodeId) -> Option<UiNodeId> {
        self.owners
            .get(&owner_id)
            .map(|state| state.blueprint.root_id)
    }

    pub(super) fn prune_removed(&mut self, tree: &UiTree) -> usize {
        let previous_count = self.owners.len();
        self.owners
            .retain(|owner_id, _| tree.nodes.contains_key(owner_id));
        previous_count - self.owners.len()
    }
}

impl UiVirtualListPrototypePoolState {
    fn capture(
        tree: &UiTree,
        owner_id: UiNodeId,
        prototype_root_id: UiNodeId,
    ) -> Result<Self, UiVirtualListPrototypePoolError> {
        let root = tree
            .node(prototype_root_id)
            .ok_or(UiVirtualListPrototypePoolError::MissingPrototype { prototype_root_id })?;
        if root.parent != Some(owner_id) {
            return Err(UiVirtualListPrototypePoolError::InvalidPrototypeRoot {
                owner_id,
                prototype_root_id,
            });
        }
        let blueprint =
            UiVirtualListPrototypeBlueprint::capture(tree, owner_id, prototype_root_id)?;
        Ok(Self {
            blueprint,
            slot_root_ids: vec![prototype_root_id],
        })
    }
}

impl UiVirtualListPrototypeBlueprint {
    fn capture(
        tree: &UiTree,
        owner_id: UiNodeId,
        root_id: UiNodeId,
    ) -> Result<Self, UiVirtualListPrototypePoolError> {
        let mut prototype_node_ids = Vec::new();
        let mut nodes = Vec::new();
        let mut visited = BTreeSet::new();
        let mut pending = vec![root_id];
        while let Some(node_id) = pending.pop() {
            if !visited.insert(node_id) {
                return Err(UiVirtualListPrototypePoolError::InvalidPrototypeTopology { node_id });
            }
            let node =
                tree.node(node_id)
                    .ok_or(UiVirtualListPrototypePoolError::MissingPrototype {
                        prototype_root_id: node_id,
                    })?;
            prototype_node_ids.push(node_id);
            nodes.push(node.clone());
            for child_id in node.children.iter().rev().copied() {
                let child = tree.node(child_id).ok_or(
                    UiVirtualListPrototypePoolError::MissingPrototype {
                        prototype_root_id: child_id,
                    },
                )?;
                if child.parent != Some(node_id) {
                    return Err(UiVirtualListPrototypePoolError::InvalidPrototypeTopology {
                        node_id: child_id,
                    });
                }
                pending.push(child_id);
            }
        }

        let root_slot = tree
            .slots
            .iter()
            .find(|slot| slot.parent_id == owner_id && slot.child_id == root_id)
            .cloned()
            .unwrap_or_else(|| UiSlot::new(owner_id, root_id, UiSlotKind::Linear));
        let internal_slots = tree
            .slots
            .iter()
            .filter(|slot| visited.contains(&slot.parent_id) && visited.contains(&slot.child_id))
            .cloned()
            .collect();
        Ok(Self {
            root_id,
            prototype_node_ids,
            nodes,
            root_slot,
            internal_slots,
        })
    }
}

struct ClonedSlotSubtree {
    root_id: UiNodeId,
    created_node_count: usize,
    reused_node_count: usize,
}

fn clone_slot_subtree<F>(
    surface: &mut UiSurface,
    blueprint: &UiVirtualListPrototypeBlueprint,
    owner_id: UiNodeId,
    slot_index: usize,
    next_node_id: &mut u64,
    configure_clone: &mut F,
) -> Result<ClonedSlotSubtree, UiVirtualListPrototypePoolError>
where
    F: FnMut(&mut UiTreeNode, UiVirtualListPrototypeNodeContext),
{
    let mut remapped_ids = BTreeMap::new();
    for prototype_node_id in &blueprint.prototype_node_ids {
        let node_id = UiNodeId::new(*next_node_id);
        *next_node_id += 1;
        remapped_ids.insert(*prototype_node_id, node_id);
    }

    let mut created_node_count = 0;
    let mut reused_node_count = 0;
    for prototype in &blueprint.nodes {
        let node_id = remapped_ids[&prototype.node_id];
        let parent_id = if prototype.node_id == blueprint.root_id {
            owner_id
        } else {
            let prototype_parent = prototype.parent.ok_or(
                UiVirtualListPrototypePoolError::InvalidPrototypeTopology {
                    node_id: prototype.node_id,
                },
            )?;
            *remapped_ids.get(&prototype_parent).ok_or(
                UiVirtualListPrototypePoolError::InvalidPrototypeTopology {
                    node_id: prototype.node_id,
                },
            )?
        };
        let mut node = prototype.clone();
        node.node_id = node_id;
        node.node_path = virtual_slot_node_path(&prototype.node_path, slot_index);
        node.parent = None;
        node.children.clear();
        node.layout_cache = Default::default();
        if let Some(control_id) = node
            .template_metadata
            .as_mut()
            .and_then(|metadata| metadata.control_id.as_mut())
        {
            *control_id = format!("{control_id}__virtual_slot_{slot_index}");
        }
        configure_clone(
            &mut node,
            UiVirtualListPrototypeNodeContext {
                owner_id,
                slot_index,
                prototype_node_id: prototype.node_id,
                node_id,
                is_slot_root: prototype.node_id == blueprint.root_id,
            },
        );
        let node_report = surface.insert_or_reuse_pooled_child(parent_id, node)?;
        created_node_count += node_report.created_count;
        reused_node_count += node_report.reused_count;
    }

    let root_id = remapped_ids[&blueprint.root_id];
    let mut root_slot = blueprint.root_slot.clone();
    root_slot.parent_id = owner_id;
    root_slot.child_id = root_id;
    surface.tree.push_layout_slot(root_slot);
    for prototype_slot in &blueprint.internal_slots {
        let mut slot = prototype_slot.clone();
        slot.parent_id = remapped_ids[&prototype_slot.parent_id];
        slot.child_id = remapped_ids[&prototype_slot.child_id];
        surface.tree.push_layout_slot(slot);
    }
    Ok(ClonedSlotSubtree {
        root_id,
        created_node_count,
        reused_node_count,
    })
}

fn next_node_id_cursor(
    tree: &UiTree,
    required_count: usize,
) -> Result<u64, UiVirtualListPrototypePoolError> {
    let current_max = tree
        .nodes
        .keys()
        .next_back()
        .map(|node_id| node_id.0)
        .unwrap_or_default();
    let required_count = u64::try_from(required_count)
        .map_err(|_| UiVirtualListPrototypePoolError::NodeIdExhausted)?;
    let last_reserved = current_max
        .checked_add(required_count)
        .ok_or(UiVirtualListPrototypePoolError::NodeIdExhausted)?;
    last_reserved
        .checked_add(1)
        .ok_or(UiVirtualListPrototypePoolError::NodeIdExhausted)?;
    current_max
        .checked_add(1)
        .ok_or(UiVirtualListPrototypePoolError::NodeIdExhausted)
}

fn virtual_slot_node_path(prototype: &UiNodePath, slot_index: usize) -> UiNodePath {
    UiNodePath::new(format!("{}#virtual-slot-{slot_index}", prototype.0))
}

// This index is a rebuildable cache and does not contribute to serialized surface identity.
impl PartialEq for UiVirtualListPrototypePoolIndex {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::UiVirtualListPrototypePoolReport;
    use crate::ui::surface::UiSurface;
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodeId, UiNodePath, UiTreeId},
        layout::{
            UiContainerKind, UiScrollState, UiScrollableBoxConfig, UiSlot, UiSlotKind,
            UiVirtualListConfig,
        },
        tree::{UiTemplateNodeMetadata, UiTreeNode},
    };

    #[test]
    fn prototype_pool_clones_complete_subtree_for_each_physical_slot() {
        let mut surface = surface_with_prototype();
        reconcile(&mut surface, 100_000);

        let report = ensure(&mut surface);

        assert_eq!(report.slot_capacity, 3);
        assert_eq!(report.live_slot_count, 3);
        assert_eq!(report.created_slot_count, 2);
        assert_eq!(surface.tree.nodes.len(), 10);
        for root_id in report.slot_root_ids {
            let root = surface.tree.node(root_id).unwrap();
            assert_eq!(root.children.len(), 1);
            let child = surface.tree.node(root.children[0]).unwrap();
            assert_eq!(child.children.len(), 1);
            assert!(surface.tree.node(child.children[0]).is_some());
            assert!(
                surface
                    .tree
                    .slots
                    .iter()
                    .any(|slot| slot.parent_id == root.node_id && slot.child_id == child.node_id)
            );
        }
    }

    #[test]
    fn unchanged_capacity_preserves_slot_roots_without_new_nodes() {
        let mut surface = surface_with_prototype();
        reconcile(&mut surface, 100_000);
        let first = ensure(&mut surface);
        let node_count = surface.tree.nodes.len();

        let second = ensure(&mut surface);

        assert_eq!(second.slot_root_ids, first.slot_root_ids);
        assert_eq!(second.created_slot_count, 0);
        assert_eq!(second.created_node_count, 0);
        assert_eq!(surface.tree.nodes.len(), node_count);
    }

    #[test]
    fn logical_count_does_not_change_physical_tree_size() {
        let mut small = surface_with_prototype();
        reconcile(&mut small, 100);
        ensure(&mut small);
        let mut large = surface_with_prototype();
        reconcile(&mut large, 100_000);
        ensure(&mut large);

        assert_eq!(small.tree.nodes.len(), large.tree.nodes.len());
        assert_eq!(
            small
                .virtual_list_prototype_slot_roots(owner_id())
                .unwrap()
                .len(),
            large
                .virtual_list_prototype_slot_roots(owner_id())
                .unwrap()
                .len()
        );
    }

    #[test]
    fn shrinking_then_growing_reuses_bounded_subtrees() {
        let mut surface = surface_with_prototype();
        reconcile(&mut surface, 100_000);
        let first = ensure(&mut surface);
        let retained_root = first.slot_root_ids[0];

        reconcile(&mut surface, 1);
        let shrunk = ensure(&mut surface);
        assert_eq!(shrunk.live_slot_count, 1);
        assert_eq!(shrunk.removed_slot_count, 2);

        reconcile(&mut surface, 100_000);
        let grown = ensure(&mut surface);
        assert_eq!(grown.live_slot_count, 3);
        assert_eq!(grown.slot_root_ids[0], retained_root);
        assert_eq!(grown.created_slot_count, 2);
        assert_eq!(grown.reused_node_count, 6);
        assert_eq!(surface.tree.nodes.len(), 10);
    }

    fn ensure(surface: &mut UiSurface) -> UiVirtualListPrototypePoolReport {
        surface
            .ensure_virtual_list_prototype_slots(owner_id(), prototype_id(), |_, _| {})
            .unwrap()
    }

    fn reconcile(surface: &mut UiSurface, logical_count: usize) {
        surface
            .reconcile_virtual_list_materialization(owner_id(), logical_count, &mut Vec::new())
            .unwrap();
    }

    fn surface_with_prototype() -> UiSurface {
        let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.virtual_list.prototype_pool"));
        surface.tree.insert_root(
            UiTreeNode::new(owner_id(), UiNodePath::new("root/list"))
                .with_container(UiContainerKind::ScrollableBox(UiScrollableBoxConfig {
                    virtualization: Some(UiVirtualListConfig {
                        item_extent: 24.0,
                        overscan: 0,
                    }),
                    ..UiScrollableBoxConfig::default()
                }))
                .with_scroll_state(UiScrollState {
                    viewport_extent: 72.0,
                    content_extent: 72.0,
                    ..UiScrollState::default()
                }),
        );
        insert_template_child(&mut surface, owner_id(), prototype_id(), "row", "Row");
        insert_template_child(
            &mut surface,
            prototype_id(),
            UiNodeId::new(3),
            "row/label",
            "RowLabel",
        );
        insert_template_child(
            &mut surface,
            UiNodeId::new(3),
            UiNodeId::new(4),
            "row/icon",
            "RowIcon",
        );
        surface
    }

    fn insert_template_child(
        surface: &mut UiSurface,
        parent_id: UiNodeId,
        node_id: UiNodeId,
        path: &str,
        control_id: &str,
    ) {
        let mut metadata = UiTemplateNodeMetadata::default();
        metadata.component = "VirtualRow".to_string();
        metadata.control_id = Some(control_id.to_string());
        surface
            .tree
            .insert_child(
                parent_id,
                UiTreeNode::new(node_id, UiNodePath::new(path)).with_template_metadata(metadata),
            )
            .unwrap();
        surface
            .tree
            .slots
            .push(UiSlot::new(parent_id, node_id, UiSlotKind::Linear));
    }

    const fn owner_id() -> UiNodeId {
        UiNodeId::new(1)
    }

    const fn prototype_id() -> UiNodeId {
        UiNodeId::new(2)
    }
}
