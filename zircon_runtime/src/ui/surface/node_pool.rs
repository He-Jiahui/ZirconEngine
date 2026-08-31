use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    event_ui::{UiNodeId, UiStateFlags},
    focus::UiFocusChangeReason,
    tree::{UiDirtyFlags, UiTree, UiTreeError, UiTreeNode},
};

use super::UiInvalidationReason;
use super::surface::UiSurface;

// Detached template identities can otherwise grow without bound. These limits
// preserve a small reusable working set while making retention explicit.
const MAX_POOLED_NODE_BUCKETS: usize = 256;
const MAX_POOLED_NODES_PER_BUCKET: usize = 4;
const MAX_POOLED_NODES: usize = MAX_POOLED_NODE_BUCKETS * MAX_POOLED_NODES_PER_BUCKET;

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
    #[serde(default)]
    /// Nodes discarded because the bounded reusable working set is full.
    pub capacity_rejected_count: usize,
    #[serde(default)]
    /// Snapshot of all reusable nodes retained after this operation.
    pub resident_node_count: usize,
    #[serde(default)]
    /// Snapshot of template-identity buckets retained after this operation.
    pub resident_bucket_count: usize,
    #[serde(default)]
    /// The fixed upper bound for `resident_node_count`.
    pub max_resident_node_count: usize,
    #[serde(default)]
    /// Detached reusable nodes explicitly released by a maintenance trim.
    pub trimmed_node_count: usize,
    #[serde(default)]
    /// Template-identity buckets explicitly released by a maintenance trim.
    pub trimmed_bucket_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
struct UiSurfaceNodePoolKey {
    component: String,
    control_id: Option<String>,
    node_path: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UiSurfaceNodePoolRecycleOutcome {
    Recycled,
    NotPoolable,
    CapacityRejected,
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
        let node_ids = subtree_node_ids(&self.tree, node_id)?;
        self.reset_detached_transient_state_for_nodes(&node_ids, UiFocusChangeReason::Despawned);
        let mutation =
            detach_subtree_to_pool(&mut self.tree, &mut self.node_pool, node_id, node_ids)?;
        for detached_node_id in &mutation.node_ids {
            self.invalidation
                .record_reason(*detached_node_id, UiInvalidationReason::Structure);
        }
        self.component_states.clear_nodes(&mutation.node_ids);
        self.add_pool_report(mutation.report.clone());
        Ok(mutation.report)
    }

    pub fn insert_or_reuse_pooled_child(
        &mut self,
        parent_id: UiNodeId,
        node: UiTreeNode,
    ) -> Result<UiSurfaceNodePoolReport, UiTreeError> {
        let node_id = node.node_id;
        let report =
            insert_or_reuse_pooled_child(&mut self.tree, &mut self.node_pool, parent_id, node)?;
        self.invalidation
            .record_reason(parent_id, UiInvalidationReason::Structure);
        self.invalidation
            .record_reason(node_id, UiInvalidationReason::Structure);
        self.add_pool_report(report.clone());
        Ok(report)
    }

    /// Releases detached reusable nodes without touching the live UI tree.
    /// Normal rebuilds retain pool contents; callers use this for an explicit
    /// idle or memory-pressure maintenance action.
    pub fn trim_retained_node_pool(&mut self) -> UiSurfaceNodePoolReport {
        let (trimmed_node_count, trimmed_bucket_count) = self.node_pool.trim();
        let mut report = UiSurfaceNodePoolReport {
            trimmed_node_count,
            trimmed_bucket_count,
            ..UiSurfaceNodePoolReport::default()
        };
        report.record_residency(&self.node_pool);
        self.add_pool_report(report.clone());
        report
    }

    fn add_pool_report(&mut self, report: UiSurfaceNodePoolReport) {
        self.pending_pool_report.created_count += report.created_count;
        self.pending_pool_report.reused_count += report.reused_count;
        self.pending_pool_report.recycled_count += report.recycled_count;
        self.pending_pool_report.discarded_count += report.discarded_count;
    }
}

impl UiSurfaceNodePool {
    pub const fn max_bucket_count() -> usize {
        MAX_POOLED_NODE_BUCKETS
    }

    pub const fn max_nodes_per_bucket() -> usize {
        MAX_POOLED_NODES_PER_BUCKET
    }

    pub const fn max_resident_node_count() -> usize {
        MAX_POOLED_NODES
    }

    pub fn resident_node_count(&self) -> usize {
        self.buckets.values().map(Vec::len).sum()
    }

    pub fn resident_bucket_count(&self) -> usize {
        self.buckets.len()
    }

    fn trim(&mut self) -> (usize, usize) {
        let trimmed_node_count = self.resident_node_count();
        let trimmed_bucket_count = self.resident_bucket_count();
        self.buckets.clear();
        (trimmed_node_count, trimmed_bucket_count)
    }

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
        matches!(
            self.recycle_with_outcome(node),
            UiSurfaceNodePoolRecycleOutcome::Recycled
        )
    }

    fn recycle_with_outcome(&mut self, node: UiTreeNode) -> UiSurfaceNodePoolRecycleOutcome {
        let Some(key) = UiSurfaceNodePoolKey::from_node(&node) else {
            return UiSurfaceNodePoolRecycleOutcome::NotPoolable;
        };
        if let Some(bucket) = self.buckets.get_mut(&key) {
            if bucket.len() >= MAX_POOLED_NODES_PER_BUCKET {
                return UiSurfaceNodePoolRecycleOutcome::CapacityRejected;
            }
            bucket.push(node);
            return UiSurfaceNodePoolRecycleOutcome::Recycled;
        }
        if self.buckets.len() >= MAX_POOLED_NODE_BUCKETS {
            return UiSurfaceNodePoolRecycleOutcome::CapacityRejected;
        }
        self.buckets.insert(key, vec![node]);
        UiSurfaceNodePoolRecycleOutcome::Recycled
    }
}

impl UiSurfaceNodePoolReport {
    fn record_residency(&mut self, pool: &UiSurfaceNodePool) {
        self.resident_node_count = pool.resident_node_count();
        self.resident_bucket_count = pool.resident_bucket_count();
        self.max_resident_node_count = UiSurfaceNodePool::max_resident_node_count();
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
    node_ids: Vec<UiNodeId>,
) -> Result<UiSurfaceNodePoolMutation, UiTreeError> {
    if !tree.nodes.contains_key(&node_id) {
        return Err(UiTreeError::MissingNode(node_id));
    }

    detach_from_parent(tree, node_id)?;
    tree.roots.retain(|root_id| *root_id != node_id);
    let detached_set = node_ids.iter().copied().collect::<BTreeSet<_>>();
    tree.retain_layout_slots(|slot| {
        !detached_set.contains(&slot.parent_id) && !detached_set.contains(&slot.child_id)
    });

    let mut report = UiSurfaceNodePoolReport::default();
    for node_id in node_ids.iter().copied().rev() {
        let Some(mut node) = tree.nodes.remove(&node_id) else {
            continue;
        };
        node.parent = None;
        node.children.clear();
        reset_recycled_node(&mut node);
        match pool.recycle_with_outcome(node) {
            UiSurfaceNodePoolRecycleOutcome::Recycled => report.recycled_count += 1,
            UiSurfaceNodePoolRecycleOutcome::NotPoolable => report.discarded_count += 1,
            UiSurfaceNodePoolRecycleOutcome::CapacityRejected => {
                report.discarded_count += 1;
                report.capacity_rejected_count += 1;
            }
        }
    }
    report.record_residency(pool);
    Ok(UiSurfaceNodePoolMutation { report, node_ids })
}

fn subtree_node_ids(tree: &UiTree, node_id: UiNodeId) -> Result<Vec<UiNodeId>, UiTreeError> {
    let mut node_ids = Vec::new();
    collect_subtree_node_ids(tree, node_id, &mut node_ids)?;
    Ok(node_ids)
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
    tree.insert_child(parent_id, node)?;
    report.record_residency(pool);
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
    tree.mark_layout_order_changed(parent_id);
    let parent = tree
        .node_mut(parent_id)
        .ok_or(UiTreeError::MissingParent(parent_id))?;
    parent.children.retain(|child_id| *child_id != node_id);
    mark_node_structure_dirty(parent);
    Ok(())
}

fn merge_reused_node(mut pooled: UiTreeNode, desired: UiTreeNode) -> UiTreeNode {
    let retained_layout_cache = pooled.layout_cache.clone();
    pooled = desired;
    pooled.layout_cache = retained_layout_cache;
    pooled.layout_cache.invalidate_measure();
    pooled.layout_cache.advance_text_layout_revision();
    pooled
}

fn reset_recycled_node(node: &mut UiTreeNode) {
    node.state_flags = reusable_state_flags(node.state_flags.clone());
    node.dirty = UiDirtyFlags::default();
}

fn reset_reinserted_node(node: &mut UiTreeNode) {
    node.children.clear();
    node.layout_cache.invalidate_measure();
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
