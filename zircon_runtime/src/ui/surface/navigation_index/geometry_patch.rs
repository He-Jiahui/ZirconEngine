use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    surface::{UiArrangedTree, UiHitTestGrid},
    tree::UiTree,
};

use super::super::frame_hit_test::UiProjectedHitTestIndex;
use super::{UiSurfaceNavigationIndex, is_navigation_geometry_authority, navigation_geometry};

impl UiSurfaceNavigationIndex {
    /// Updates frame-only candidate movement in place. Membership, z, and paint-order changes
    /// affect sorted/modal structures and deliberately fall back to a complete rebuild.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn patch_changed_geometry(
        &mut self,
        tree: &UiTree,
        arranged_tree: &UiArrangedTree,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
        projected_hit_test: &UiProjectedHitTestIndex,
        base_hit_grid: &UiHitTestGrid,
        changed_node_ids: &BTreeSet<UiNodeId>,
        removed_node_ids: &BTreeSet<UiNodeId>,
    ) -> Result<usize, ()> {
        if !self.initialized || self.build_error.is_some() {
            return Err(());
        }
        if removed_node_ids
            .iter()
            .any(|node_id| self.geometry_authority_node_ids.contains(node_id))
        {
            return Err(());
        }

        let authority_node_ids = &self.geometry_authority_node_ids;
        let referenced_modal_root_node_ids = &self.referenced_modal_root_node_ids;
        let indexed_nodes = &mut self.nodes;
        let mut patched_node_count = 0usize;
        for node_id in changed_node_ids {
            let was_authority = authority_node_ids.contains(node_id);
            let Some(node) = tree.nodes.get(node_id) else {
                if was_authority {
                    return Err(());
                }
                continue;
            };
            let is_authority = is_navigation_geometry_authority(*node_id, node)
                || referenced_modal_root_node_ids.contains(node_id);
            if was_authority != is_authority {
                return Err(());
            }
            if !was_authority {
                continue;
            }
            let previous = indexed_nodes.get_mut(node_id).ok_or(())?;
            let current = navigation_geometry(
                *node_id,
                node.layout_cache.frame,
                node.z_index,
                node.paint_order,
                arranged_tree,
                arranged_node_indices,
                projected_hit_test,
                base_hit_grid,
            );
            if previous.z_index != current.z_index || previous.paint_order != current.paint_order {
                return Err(());
            }
            if previous.frame != current.frame {
                previous.frame = current.frame;
                patched_node_count = patched_node_count.saturating_add(1);
            }
        }
        Ok(patched_node_count)
    }

    /// Popup projection can move candidate frames without changing membership or ordering. The
    /// authority set bounds this scan to navigation-relevant nodes rather than the whole tree.
    pub(super) fn patch_projected_geometry(
        &mut self,
        tree: &UiTree,
        arranged_tree: &UiArrangedTree,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
        projected_hit_test: &UiProjectedHitTestIndex,
        base_hit_grid: &UiHitTestGrid,
    ) -> Result<usize, ()> {
        if !self.initialized || self.build_error.is_some() {
            return Err(());
        }

        let authority_node_ids = &self.geometry_authority_node_ids;
        let referenced_modal_root_node_ids = &self.referenced_modal_root_node_ids;
        let indexed_nodes = &mut self.nodes;
        let mut patched_node_count = 0usize;
        for node_id in authority_node_ids {
            let node = tree.nodes.get(node_id).ok_or(())?;
            if !is_navigation_geometry_authority(*node_id, node)
                && !referenced_modal_root_node_ids.contains(node_id)
            {
                return Err(());
            }
            let previous = indexed_nodes.get_mut(node_id).ok_or(())?;
            let current = navigation_geometry(
                *node_id,
                node.layout_cache.frame,
                node.z_index,
                node.paint_order,
                arranged_tree,
                arranged_node_indices,
                projected_hit_test,
                base_hit_grid,
            );
            if previous.z_index != current.z_index || previous.paint_order != current.paint_order {
                return Err(());
            }
            if previous.frame != current.frame {
                previous.frame = current.frame;
                patched_node_count = patched_node_count.saturating_add(1);
            }
        }
        Ok(patched_node_count)
    }
}
