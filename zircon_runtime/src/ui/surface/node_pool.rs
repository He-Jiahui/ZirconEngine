use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiStateFlags},
    focus::UiFocusChangeReason,
    tree::{UiDirtyFlags, UiTree, UiTreeError, UiTreeNode},
};

use super::surface::UiSurface;

/// Surface-local pool keyed by template identity so retained UI rebuilds can reuse detached nodes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceNodePool {
    buckets: BTreeMap<UiSurfaceNodePoolKey, Vec<UiTreeNode>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSurfaceNodePoolReport {
    pub created_count: usize,
    pub reused_count: usize,
    pub recycled_count: usize,
    pub discarded_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct UiSurfaceNodePoolKey {
    component: String,
    control_id: Option<String>,
    node_path: String,
}

pub(crate) struct UiSurfaceNodePoolMutation {
    pub report: UiSurfaceNodePoolReport,
    pub node_ids: Vec<UiNodeId>,
}

impl UiSurface {
    pub fn detach_subtree_to_pool(
        &mut self,
        node_id: UiNodeId,
    ) -> Result<UiSurfaceNodePoolReport, UiTreeError> {
        let mutation = detach_subtree_to_pool(&mut self.tree, &mut self.node_pool, node_id)?;
        self.reset_detached_transient_state_for_nodes(
            &mutation.node_ids,
            UiFocusChangeReason::Despawned,
        );
        self.add_pool_report(mutation.report.clone());
        Ok(mutation.report)
    }

    pub fn insert_or_reuse_pooled_child(
        &mut self,
        parent_id: UiNodeId,
        node: UiTreeNode,
    ) -> Result<UiSurfaceNodePoolReport, UiTreeError> {
        let report =
            insert_or_reuse_pooled_child(&mut self.tree, &mut self.node_pool, parent_id, node)?;
        self.add_pool_report(report.clone());
        Ok(report)
    }

    fn add_pool_report(&mut self, report: UiSurfaceNodePoolReport) {
        self.pending_pool_report.created_count += report.created_count;
        self.pending_pool_report.reused_count += report.reused_count;
        self.pending_pool_report.recycled_count += report.recycled_count;
        self.pending_pool_report.discarded_count += report.discarded_count;
    }
}

impl UiSurfaceNodePool {
    pub fn take(&mut self, desired: &UiTreeNode) -> Option<UiTreeNode> {
        let key = UiSurfaceNodePoolKey::from_node(desired)?;
        let bucket = self.buckets.get_mut(&key)?;
        let node = bucket.pop();
        if bucket.is_empty() {
            self.buckets.remove(&key);
        }
        node
    }

    pub fn recycle(&mut self, node: UiTreeNode) -> bool {
        let Some(key) = UiSurfaceNodePoolKey::from_node(&node) else {
            return false;
        };
        self.buckets.entry(key).or_default().push(node);
        true
    }
}

impl UiSurfaceNodePoolKey {
    fn from_node(node: &UiTreeNode) -> Option<Self> {
        let metadata = node.template_metadata.as_ref()?;
        Some(Self {
            component: metadata.component.clone(),
            control_id: metadata.control_id.clone(),
            node_path: node.node_path.0.clone(),
        })
    }
}

pub(crate) fn detach_subtree_to_pool(
    tree: &mut UiTree,
    pool: &mut UiSurfaceNodePool,
    node_id: UiNodeId,
) -> Result<UiSurfaceNodePoolMutation, UiTreeError> {
    if !tree.nodes.contains_key(&node_id) {
        return Err(UiTreeError::MissingNode(node_id));
    }

    let mut detached = Vec::new();
    collect_subtree_node_ids(tree, node_id, &mut detached)?;
    detach_from_parent(tree, node_id)?;
    tree.roots.retain(|root_id| *root_id != node_id);

    let mut report = UiSurfaceNodePoolReport::default();
    for node_id in detached.iter().copied().rev() {
        let Some(mut node) = tree.nodes.remove(&node_id) else {
            continue;
        };
        tree.slots
            .retain(|slot| slot.parent_id != node_id && slot.child_id != node_id);
        node.parent = None;
        node.children.clear();
        reset_recycled_node(&mut node);
        if pool.recycle(node) {
            report.recycled_count += 1;
        } else {
            report.discarded_count += 1;
        }
    }
    Ok(UiSurfaceNodePoolMutation {
        report,
        node_ids: detached,
    })
}

pub(crate) fn insert_or_reuse_pooled_child(
    tree: &mut UiTree,
    pool: &mut UiSurfaceNodePool,
    parent_id: UiNodeId,
    desired: UiTreeNode,
) -> Result<UiSurfaceNodePoolReport, UiTreeError> {
    if !tree.nodes.contains_key(&parent_id) {
        return Err(UiTreeError::MissingParent(parent_id));
    }
    if tree.nodes.contains_key(&desired.node_id) {
        return Err(UiTreeError::DuplicateNode(desired.node_id));
    }

    let mut report = UiSurfaceNodePoolReport::default();
    let mut node = if let Some(pooled) = pool.take(&desired) {
        report.reused_count = 1;
        merge_reused_node(pooled, desired)
    } else {
        report.created_count = 1;
        desired
    };

    reset_reinserted_node(&mut node);
    node.parent = Some(parent_id);
    node.paint_order = next_paint_order(tree);
    let node_id = node.node_id;
    tree.nodes
        .get_mut(&parent_id)
        .ok_or(UiTreeError::MissingParent(parent_id))?
        .children
        .push(node_id);
    tree.nodes.insert(node_id, node);
    mark_parent_structure_dirty(tree, parent_id)?;
    Ok(report)
}

fn collect_subtree_node_ids(
    tree: &UiTree,
    node_id: UiNodeId,
    collected: &mut Vec<UiNodeId>,
) -> Result<(), UiTreeError> {
    let node = tree
        .nodes
        .get(&node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?;
    collected.push(node_id);
    for child_id in &node.children {
        collect_subtree_node_ids(tree, *child_id, collected)?;
    }
    Ok(())
}

fn detach_from_parent(tree: &mut UiTree, node_id: UiNodeId) -> Result<(), UiTreeError> {
    let parent_id = tree
        .nodes
        .get(&node_id)
        .ok_or(UiTreeError::MissingNode(node_id))?
        .parent;
    let Some(parent_id) = parent_id else {
        return Ok(());
    };
    let parent = tree
        .nodes
        .get_mut(&parent_id)
        .ok_or(UiTreeError::MissingParent(parent_id))?;
    parent.children.retain(|child_id| *child_id != node_id);
    mark_node_structure_dirty(parent);
    Ok(())
}

fn merge_reused_node(mut pooled: UiTreeNode, desired: UiTreeNode) -> UiTreeNode {
    let retained_layout_cache = pooled.layout_cache.clone();
    pooled = desired;
    pooled.layout_cache = retained_layout_cache;
    pooled
}

fn reset_recycled_node(node: &mut UiTreeNode) {
    node.state_flags = reusable_state_flags(node.state_flags.clone());
    node.dirty = UiDirtyFlags::default();
}

fn reset_reinserted_node(node: &mut UiTreeNode) {
    node.children.clear();
    node.state_flags = reusable_state_flags(node.state_flags.clone());
    node.dirty = structure_dirty_flags();
}

fn reusable_state_flags(flags: UiStateFlags) -> UiStateFlags {
    UiStateFlags {
        pressed: false,
        dirty: false,
        ..flags
    }
}

fn mark_parent_structure_dirty(tree: &mut UiTree, parent_id: UiNodeId) -> Result<(), UiTreeError> {
    let parent = tree
        .nodes
        .get_mut(&parent_id)
        .ok_or(UiTreeError::MissingParent(parent_id))?;
    mark_node_structure_dirty(parent);
    Ok(())
}

fn mark_node_structure_dirty(node: &mut UiTreeNode) {
    node.dirty = structure_dirty_flags();
}

fn structure_dirty_flags() -> UiDirtyFlags {
    UiDirtyFlags {
        layout: true,
        hit_test: true,
        render: true,
        input: true,
        ..UiDirtyFlags::default()
    }
}

fn next_paint_order(tree: &UiTree) -> u64 {
    tree.nodes
        .values()
        .map(|node| node.paint_order)
        .max()
        .map_or(0, |paint_order| paint_order.saturating_add(1))
}
