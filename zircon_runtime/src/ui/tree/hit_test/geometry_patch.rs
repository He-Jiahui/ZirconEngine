use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    surface::{UiArrangedNode, UiArrangedTree, UiPersistentSequenceCowStats},
};

use super::route_index::route_node_index_for_node;
use super::{
    bounded_cells_for_frame, entry_sort_key, frame_is_contained, stable_geometry_entry,
    UiHitTestIndex,
};

impl UiHitTestIndex {
    pub(crate) fn patch_arranged_geometry(
        &mut self,
        arranged_tree: &UiArrangedTree,
        changed_node_ids: &BTreeSet<UiNodeId>,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    ) -> Result<bool, ()> {
        if changed_node_ids.is_empty() {
            return Ok(false);
        }
        if (self.entry_cells.is_empty() || self.entry_indices.is_empty())
            && !self.grid.entries.is_empty()
        {
            self.reindex_entry_cells();
        }

        let mut updates = Vec::with_capacity(changed_node_ids.len());
        for node_id in changed_node_ids {
            let node =
                arranged_node_for_patch(arranged_tree, arranged_node_indices, *node_id).ok_or(())?;
            let route_node_index =
                route_node_index_for_node(arranged_node_indices, *node_id).ok_or(())?;
            let next_entry =
                stable_geometry_entry(self.grid.route_nodes.as_slice(), node, route_node_index);
            let entry_index = self.entry_index_by_node_id(*node_id);
            let (entry_index, next_entry) = match (entry_index, next_entry) {
                (Some(entry_index), Some(next_entry)) => (entry_index, next_entry),
                (None, None) => continue,
                _ => return Err(()),
            };
            let previous_entry = self.grid.entries.get(entry_index).ok_or(())?;
            let previous_cells = self.entry_cells.get(node_id).cloned().ok_or(())?;
            let next_cells =
                if next_entry.clip_frame.width > 0.0 && next_entry.clip_frame.height > 0.0 {
                    if self.grid.columns == 0
                        || self.grid.rows == 0
                        || !frame_is_contained(self.grid.bounds, next_entry.clip_frame)
                    {
                        return Err(());
                    }
                    bounded_cells_for_frame(
                        self.grid.bounds,
                        self.grid.columns,
                        self.grid.rows,
                        self.grid.cell_size,
                        next_entry.clip_frame,
                    )
                } else {
                    Vec::new()
                };
            if next_cells
                .iter()
                .any(|cell_index| self.grid.cells.get(*cell_index).is_none())
            {
                return Err(());
            }
            if previous_entry != &next_entry || previous_cells != next_cells {
                updates.push((entry_index, next_entry, previous_cells, next_cells));
            }
        }

        let changed = !updates.is_empty();
        let mut entry_cow_stats = UiPersistentSequenceCowStats::default();
        let mut cell_cow_stats = UiPersistentSequenceCowStats::default();
        let mut cell_membership_clone_count = 0_usize;
        for (entry_index, entry, previous_cells, next_cells) in updates {
            for cell_index in previous_cells {
                if let Some((cell, stats)) = self.grid.cells.get_mut_with_stats(cell_index) {
                    cell_cow_stats.accumulate(stats);
                    cell_membership_clone_count = cell_membership_clone_count
                        .saturating_add(cell.entries.retain(|candidate| *candidate != entry_index));
                }
            }
            let entry_node_id = entry.node_id;
            let (current_entry, stats) = self
                .grid
                .entries
                .get_mut_with_stats(entry_index)
                .ok_or(())?;
            entry_cow_stats.accumulate(stats);
            *current_entry = entry;
            self.entry_cells.insert(entry_node_id, next_cells.clone());
            for cell_index in next_cells {
                let key = entry_sort_key(&self.grid.entries[entry_index]);
                let insertion_index =
                    self.grid.cells[cell_index]
                        .entries
                        .partition_point(|candidate| {
                            self.grid
                                .entries
                                .get(*candidate)
                                .map(entry_sort_key)
                                .unwrap_or_default()
                                <= key
                        });
                let (cell, stats) = self.grid.cells.get_mut_with_stats(cell_index).ok_or(())?;
                cell_cow_stats.accumulate(stats);
                cell_membership_clone_count = cell_membership_clone_count
                    .saturating_add(cell.entries.insert(insertion_index, entry_index));
            }
        }
        record_hit_grid_persistent_cow(
            entry_cow_stats,
            cell_cow_stats,
            cell_membership_clone_count,
        );
        Ok(changed)
    }
}

fn record_hit_grid_persistent_cow(
    entry_stats: UiPersistentSequenceCowStats,
    cell_stats: UiPersistentSequenceCowStats,
    cell_membership_clone_count: usize,
) {
    crate::profile_counter!(
        "runtime",
        "ui.hit_grid.persistent_entry_item_clone_count",
        entry_stats.cloned_item_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.hit_grid.persistent_entry_segment_clone_count",
        entry_stats.cloned_segment_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.hit_grid.persistent_cell_item_clone_count",
        cell_stats.cloned_item_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.hit_grid.persistent_cell_segment_clone_count",
        cell_stats.cloned_segment_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.hit_grid.persistent_cell_membership_clone_count",
        cell_membership_clone_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.hit_grid.persistent_directory_node_clone_count",
        entry_stats
            .cloned_directory_node_count
            .saturating_add(cell_stats.cloned_directory_node_count)
    );
}

fn arranged_node_for_patch<'a>(
    arranged_tree: &'a UiArrangedTree,
    arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    node_id: UiNodeId,
) -> Option<&'a UiArrangedNode> {
    let index = arranged_node_indices.get(&node_id).copied()?;
    arranged_tree
        .nodes
        .get(index)
        .filter(|node| node.node_id == node_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime_interface::ui::{
        event_ui::{UiNodePath, UiTreeId},
        layout::{UiFrame, UiPoint},
        tree::{UiInputPolicy, UiVisibility},
    };

    #[test]
    fn geometry_patch_activates_and_deactivates_stable_entry_cells() {
        let anchor_id = UiNodeId::new(1);
        let moving_id = UiNodeId::new(2);
        let anchor_frame = UiFrame::new(0.0, 0.0, 100.0, 100.0);
        let mut arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.hit.stable-clipped-entry"),
            roots: vec![anchor_id, moving_id].into(),
            nodes: vec![
                pointer_node(anchor_id, 0, anchor_frame, anchor_frame),
                pointer_node(
                    moving_id,
                    1,
                    UiFrame::new(200.0, 0.0, 20.0, 20.0),
                    anchor_frame,
                ),
            ]
            .into(),
            draw_order: vec![anchor_id, moving_id].into(),
            canvas_layers: Vec::new().into(),
        };
        let node_indices = BTreeMap::from([(anchor_id, 0), (moving_id, 1)]);
        let mut index = UiHitTestIndex::default();
        index.rebuild_arranged_indexed(&arranged_tree, &node_indices);

        assert_eq!(index.grid.entries.len(), 2);
        assert_eq!(
            index
                .hit_test_arranged(&arranged_tree, UiPoint::new(10.0, 10.0))
                .top_hit,
            Some(anchor_id)
        );

        arranged_tree.nodes[1].frame = UiFrame::new(5.0, 5.0, 20.0, 20.0);
        assert!(index
            .patch_arranged_geometry(&arranged_tree, &BTreeSet::from([moving_id]), &node_indices,)
            .unwrap());
        assert_eq!(
            index
                .hit_test_arranged(&arranged_tree, UiPoint::new(10.0, 10.0))
                .top_hit,
            Some(moving_id)
        );

        arranged_tree.nodes[1].frame = UiFrame::new(200.0, 0.0, 20.0, 20.0);
        assert!(index
            .patch_arranged_geometry(&arranged_tree, &BTreeSet::from([moving_id]), &node_indices,)
            .unwrap());
        assert_eq!(
            index
                .hit_test_arranged(&arranged_tree, UiPoint::new(10.0, 10.0))
                .top_hit,
            Some(anchor_id)
        );
    }

    fn pointer_node(
        node_id: UiNodeId,
        paint_order: u64,
        frame: UiFrame,
        clip_frame: UiFrame,
    ) -> UiArrangedNode {
        UiArrangedNode {
            node_id,
            node_path: UiNodePath::new(format!("root/{}", node_id.0)),
            parent: None,
            children: Vec::new(),
            frame,
            clip_frame,
            z_index: 0,
            paint_order,
            visibility: UiVisibility::Visible,
            input_policy: UiInputPolicy::Receive,
            pointer_events: Default::default(),
            enabled: true,
            clickable: true,
            hoverable: true,
            focusable: false,
            clip_to_bounds: false,
            control_id: None,
            slot: None,
        }
    }
}
