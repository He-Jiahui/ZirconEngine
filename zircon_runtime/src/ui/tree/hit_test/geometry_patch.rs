use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    surface::{UiArrangedNode, UiArrangedTree, UiHitRouteNode, UiPersistentSequenceCowStats},
};

use super::cell_membership_patch::{UiCellMembershipPatchStats, UiCellMembershipPatches};
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
        self.patch_arranged_geometry_with_stats(
            arranged_tree,
            changed_node_ids,
            arranged_node_indices,
        )
        .map(|(changed, _stats)| changed)
    }

    fn patch_arranged_geometry_with_stats(
        &mut self,
        arranged_tree: &UiArrangedTree,
        changed_node_ids: &BTreeSet<UiNodeId>,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    ) -> Result<(bool, UiCellMembershipPatchStats), ()> {
        let route_nodes = self.grid.route_nodes.clone();
        self.patch_geometry_with_routes_and_stats(
            arranged_tree,
            changed_node_ids,
            arranged_node_indices,
            route_nodes.as_slice(),
        )
    }

    pub(super) fn patch_arranged_geometry_with_routes(
        &mut self,
        arranged_tree: &UiArrangedTree,
        changed_node_ids: &BTreeSet<UiNodeId>,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
        route_nodes: &[UiHitRouteNode],
    ) -> Result<bool, ()> {
        self.patch_geometry_with_routes_and_stats(
            arranged_tree,
            changed_node_ids,
            arranged_node_indices,
            route_nodes,
        )
        .map(|(changed, _stats)| changed)
    }

    fn patch_geometry_with_routes_and_stats(
        &mut self,
        arranged_tree: &UiArrangedTree,
        changed_node_ids: &BTreeSet<UiNodeId>,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
        route_nodes: &[UiHitRouteNode],
    ) -> Result<(bool, UiCellMembershipPatchStats), ()> {
        if changed_node_ids.is_empty() {
            return Ok((false, UiCellMembershipPatchStats::default()));
        }
        let cold_lookup = (self.entry_cells.is_empty() || self.entry_indices.is_empty())
            && !self.grid.entries.is_empty();
        if !cold_lookup {
            return self.patch_geometry_with_routes_and_stats_inner(
                arranged_tree,
                changed_node_ids,
                arranged_node_indices,
                route_nodes,
            );
        }

        // Reverse maps are serde-skipped and may be cold after deserialization.  Keep their
        // preflight repair isolated so a later admission failure cannot become an observable
        // mutation of the index.
        let previous_entry_cells = self.entry_cells.clone();
        let previous_entry_indices = self.entry_indices.clone();
        self.reindex_entry_cells();
        let result = self.patch_geometry_with_routes_and_stats_inner(
            arranged_tree,
            changed_node_ids,
            arranged_node_indices,
            route_nodes,
        );
        if result.is_err() {
            self.entry_cells = previous_entry_cells;
            self.entry_indices = previous_entry_indices;
        }
        result
    }

    fn patch_geometry_with_routes_and_stats_inner(
        &mut self,
        arranged_tree: &UiArrangedTree,
        changed_node_ids: &BTreeSet<UiNodeId>,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
        route_nodes: &[UiHitRouteNode],
    ) -> Result<(bool, UiCellMembershipPatchStats), ()> {
        if changed_node_ids.is_empty() {
            return Ok((false, UiCellMembershipPatchStats::default()));
        }
        let mut updates = Vec::with_capacity(changed_node_ids.len());
        for node_id in changed_node_ids {
            let node =
                arranged_node_for_patch(arranged_tree, arranged_node_indices, *node_id).ok_or(())?;
            let route_node_index =
                route_node_index_for_node(arranged_node_indices, *node_id).ok_or(())?;
            let next_entry = stable_geometry_entry(route_nodes, node, route_node_index);
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
                    .collect::<Vec<_>>()
                } else {
                    Vec::new()
                };
            if previous_cells
                .iter()
                .chain(&next_cells)
                .any(|cell_index| self.grid.cells.get(*cell_index).is_none())
            {
                return Err(());
            }
            if previous_entry != &next_entry || previous_cells != next_cells {
                let reorder_membership =
                    entry_sort_key(previous_entry) != entry_sort_key(&next_entry);
                updates.push((
                    entry_index,
                    next_entry,
                    previous_cells,
                    next_cells,
                    reorder_membership,
                ));
            }
        }

        let changed = !updates.is_empty();
        let mut entry_cow_stats = UiPersistentSequenceCowStats::default();
        let mut membership_patches = UiCellMembershipPatches::default();
        let mut staged_entries = BTreeMap::new();
        for (entry_index, entry, previous_cells, next_cells, reorder_membership) in &updates {
            membership_patches.stage(
                *entry_index,
                previous_cells,
                next_cells,
                *reorder_membership,
            );
            staged_entries.insert(*entry_index, entry.clone());
        }
        let entries = &self.grid.entries;
        let membership_stats = membership_patches.apply(&mut self.grid.cells, |entry_index| {
            staged_entries
                .get(&entry_index)
                .map(entry_sort_key)
                .or_else(|| entries.get(entry_index).map(entry_sort_key))
                .unwrap_or_default()
        })?;
        for (entry_index, entry, _previous_cells, next_cells, _reorder_membership) in updates {
            let entry_node_id = entry.node_id;
            let (current_entry, stats) = self
                .grid
                .entries
                .get_mut_with_stats(entry_index)
                .ok_or(())?;
            entry_cow_stats.accumulate(stats);
            *current_entry = entry;
            self.entry_cells.insert(entry_node_id, next_cells);
        }
        record_hit_grid_persistent_cow(entry_cow_stats, membership_stats);
        Ok((changed, membership_stats))
    }
}

fn record_hit_grid_persistent_cow(
    entry_stats: UiPersistentSequenceCowStats,
    membership_stats: UiCellMembershipPatchStats,
) {
    let cell_stats = membership_stats.cell_cow_stats;
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
        "ui.hit_grid.persistent_cell_membership_arc_cow_clone_count",
        0
    );
    crate::profile_counter!(
        "runtime",
        "ui.hit_grid.persistent_directory_node_clone_count",
        entry_stats
            .cloned_directory_node_count
            .saturating_add(cell_stats.cloned_directory_node_count)
    );
    crate::profile_counter!(
        "runtime",
        "ui.hit_grid.cell_patch_staged_count",
        membership_stats.staged_cell_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.hit_grid.cell_patch_published_count",
        membership_stats.published_cell_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.hit_grid.cell_patch_source_membership_count",
        membership_stats.source_membership_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.hit_grid.cell_patch_removal_count",
        membership_stats.staged_removal_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.hit_grid.cell_patch_addition_count",
        membership_stats.staged_addition_count
    );
    // Counts replacement buffers and values materialized in them, excluding delta/set/Arc
    // allocations and ordering comparisons that remain covered by elapsed time.
    crate::profile_counter!(
        "runtime",
        "ui.hit_grid.cell_patch_materialized_membership_count",
        membership_stats.materialized_membership_count
    );
    crate::profile_counter!(
        "runtime",
        "ui.hit_grid.cell_patch_replacement_buffer_count",
        membership_stats.replacement_buffer_count
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
#[path = "tests/geometry_patch.rs"]
mod tests;
