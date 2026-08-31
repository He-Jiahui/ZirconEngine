use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    navigation::{UiNavigationGroup, UiTabIndex},
    surface::{UiArrangedTree, UiHitTestGrid},
    tree::{UiTree, UiTreeError, UiTreeNode},
};

use super::super::frame_hit_test::UiProjectedHitTestIndex;
use super::{
    UiNavigationIndexNode, UiSurfaceNavigationIndex, active_modal_navigation_group_scope,
    is_active_mui_modal_focus_scope, navigation_geometry, ranked_mui_scope, retain_topmost_scope,
};

struct UiResolvedNavigationContext<'a> {
    group: Option<&'a UiNavigationGroup>,
    modal_group: Option<(UiNodeId, &'a UiNavigationGroup)>,
    mui_modal_root: Option<UiNodeId>,
}

impl UiSurfaceNavigationIndex {
    /// Local input, style, text, and visible-range changes can preserve the retained index when
    /// their resolved navigation signature is unchanged. Work is bounded by changed nodes and
    /// ancestor depth; the event path never pays this comparison cost.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn needs_semantics_rebuild(
        &self,
        tree: &UiTree,
        arranged_tree: &UiArrangedTree,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
        projected_hit_test: &UiProjectedHitTestIndex,
        base_hit_grid: &UiHitTestGrid,
        changed_node_ids: &BTreeSet<UiNodeId>,
        removed_node_ids: &BTreeSet<UiNodeId>,
    ) -> bool {
        if !self.initialized
            || self.build_error.is_some()
            || (changed_node_ids.is_empty() && removed_node_ids.is_empty())
        {
            return true;
        }
        if removed_node_ids.iter().any(|node_id| {
            self.nodes
                .get(node_id)
                .is_some_and(|node| node.subtree_navigation_authority)
        }) {
            return true;
        }

        changed_node_ids.iter().any(|node_id| {
            if removed_node_ids.contains(node_id) {
                return false;
            }
            let Some(node) = tree.nodes.get(node_id) else {
                return true;
            };
            let Some(previous) = self.nodes.get(node_id) else {
                return subtree_has_navigation_semantic_authority(tree, *node_id);
            };
            let Ok(context) = resolved_navigation_context(tree, *node_id) else {
                return true;
            };
            !retained_semantics_match(
                previous,
                node,
                *node_id,
                context,
                tree,
                arranged_tree,
                arranged_node_indices,
                projected_hit_test,
                base_hit_grid,
            )
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn retained_semantics_match(
    previous: &UiNavigationIndexNode,
    node: &UiTreeNode,
    node_id: UiNodeId,
    context: UiResolvedNavigationContext<'_>,
    tree: &UiTree,
    arranged_tree: &UiArrangedTree,
    arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    projected_hit_test: &UiProjectedHitTestIndex,
    base_hit_grid: &UiHitTestGrid,
) -> bool {
    let mut local_modal_scope = context.modal_group.and_then(|(owner, group)| {
        active_modal_navigation_group_scope(
            tree,
            arranged_tree,
            arranged_node_indices,
            projected_hit_test,
            base_hit_grid,
            owner,
            group,
        )
    });
    if let Some(root) = context.mui_modal_root {
        retain_topmost_scope(
            &mut local_modal_scope,
            ranked_mui_scope(
                tree,
                arranged_tree,
                arranged_node_indices,
                projected_hit_test,
                base_hit_grid,
                root,
            ),
        );
    }

    let focus_candidate = node.is_focus_candidate();
    let focus_candidate_changed = previous.focus_candidate != node.is_focus_candidate();
    let candidate_semantics_changed = (previous.focus_candidate || focus_candidate) && {
        let tab_index = node.navigation.tab_index.unwrap_or_else(|| {
            let geometry = navigation_geometry(
                node_id,
                node.layout_cache.frame,
                node.z_index,
                node.paint_order,
                arranged_tree,
                arranged_node_indices,
                projected_hit_test,
                base_hit_grid,
            );
            UiTabIndex {
                order: geometry.paint_order.min(i32::MAX as u64) as i32,
                tabbable: focus_candidate,
            }
        });
        previous.tab_order != tab_index.order
            || previous.tabbable != tab_index.tabbable
            || previous.directional.as_ref() != node.navigation.directional.as_ref()
    };
    focus_candidate_changed
        || candidate_semantics_changed
        || previous.group_order != context.group.map_or(0, |group| group.order)
        || previous.group_id.as_ref() != context.group.map(|group| &group.group_id)
        || previous.modal_group_id.as_ref() != context.modal_group.map(|(_, group)| &group.group_id)
        || previous.mui_modal_root != context.mui_modal_root
        || previous.local_modal_scope.as_ref() != local_modal_scope.as_ref()
}

fn resolved_navigation_context(
    tree: &UiTree,
    node_id: UiNodeId,
) -> Result<UiResolvedNavigationContext<'_>, UiTreeError> {
    let mut group = None;
    let mut modal_group = None;
    let mut mui_modal_root = None;
    let mut current = Some(node_id);
    while let Some(current_id) = current {
        let node = tree
            .nodes
            .get(&current_id)
            .ok_or(UiTreeError::MissingNode(current_id))?;
        if group.is_none() {
            group = node.navigation.group.as_ref();
        }
        if modal_group.is_none() {
            modal_group = node
                .navigation
                .group
                .as_ref()
                .filter(|group| group.modal)
                .map(|group| (current_id, group));
        }
        if mui_modal_root.is_none() && is_active_mui_modal_focus_scope(node) {
            mui_modal_root = Some(current_id);
        }
        current = node.parent;
    }
    Ok(UiResolvedNavigationContext {
        group,
        modal_group,
        mui_modal_root,
    })
}

fn subtree_has_navigation_semantic_authority(tree: &UiTree, node_id: UiNodeId) -> bool {
    let Some(node) = tree.nodes.get(&node_id) else {
        return true;
    };
    node.is_focus_candidate()
        || node.navigation.tab_index.is_some()
        || node.navigation.group.is_some()
        || node.navigation.directional.is_some()
        || is_active_mui_modal_focus_scope(node)
        || node
            .children
            .iter()
            .any(|child_id| subtree_has_navigation_semantic_authority(tree, *child_id))
}
