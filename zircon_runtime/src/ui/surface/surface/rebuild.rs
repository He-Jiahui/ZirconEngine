use std::{collections::BTreeSet, time::Instant};

use serde::{Deserialize, Serialize};

use crate::ui::layout::{compute_incremental_layout_tree, compute_layout_tree};
use crate::ui::surface::{
    build_arranged_tree, extract_ui_render_tree_from_arranged, render::UiSurfaceRenderCacheStats,
};
use zircon_runtime_interface::ui::{
    dispatch::{UiPointerDispatchEffect, UiPointerDispatchResult},
    event_ui::UiNodeId,
    layout::{UiLayoutEngineSelectionReport, UiSize},
    surface::UiSurfaceRebuildDebugStats,
    tree::{UiDirtyFlags, UiTree, UiTreeError, UiTreeNode},
};

use super::UiSurface;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSurfaceRebuildReport {
    pub dirty_flags: UiDirtyFlags,
    pub dirty_node_count: usize,
    pub layout_recomputed: bool,
    pub arranged_rebuilt: bool,
    pub hit_grid_rebuilt: bool,
    pub render_rebuilt: bool,
    pub arranged_node_count: usize,
    pub render_command_count: usize,
    pub hit_grid_entry_count: usize,
    pub hit_grid_cell_count: usize,
    #[serde(default)]
    pub layout_visited_node_count: usize,
    #[serde(default)]
    pub layout_geometry_changed_node_count: usize,
    #[serde(default)]
    pub layout_skipped_node_count: usize,
    #[serde(default)]
    pub render_command_reused_count: usize,
    #[serde(default)]
    pub render_command_rebuilt_count: usize,
    #[serde(default)]
    pub render_damage_rect_count: usize,
    #[serde(default)]
    pub control_pool_created_count: usize,
    #[serde(default)]
    pub control_pool_reused_count: usize,
    #[serde(default)]
    pub control_pool_recycled_count: usize,
    #[serde(default)]
    pub control_pool_discarded_count: usize,
    pub layout_elapsed_micros: u64,
    pub arranged_elapsed_micros: u64,
    pub hit_grid_elapsed_micros: u64,
    pub render_elapsed_micros: u64,
}

impl UiSurfaceRebuildReport {
    pub fn debug_stats(self) -> UiSurfaceRebuildDebugStats {
        UiSurfaceRebuildDebugStats {
            dirty_flags: self.dirty_flags,
            dirty_node_count: self.dirty_node_count,
            layout_recomputed: self.layout_recomputed,
            arranged_rebuilt: self.arranged_rebuilt,
            hit_grid_rebuilt: self.hit_grid_rebuilt,
            render_rebuilt: self.render_rebuilt,
            arranged_node_count: self.arranged_node_count,
            render_command_count: self.render_command_count,
            hit_grid_entry_count: self.hit_grid_entry_count,
            hit_grid_cell_count: self.hit_grid_cell_count,
            layout_visited_node_count: self.layout_visited_node_count,
            layout_geometry_changed_node_count: self.layout_geometry_changed_node_count,
            layout_skipped_node_count: self.layout_skipped_node_count,
            render_command_reused_count: self.render_command_reused_count,
            render_command_rebuilt_count: self.render_command_rebuilt_count,
            render_damage_rect_count: self.render_damage_rect_count,
            control_pool_created_count: self.control_pool_created_count,
            control_pool_reused_count: self.control_pool_reused_count,
            control_pool_recycled_count: self.control_pool_recycled_count,
            control_pool_discarded_count: self.control_pool_discarded_count,
            layout_elapsed_micros: self.layout_elapsed_micros,
            arranged_elapsed_micros: self.arranged_elapsed_micros,
            hit_grid_elapsed_micros: self.hit_grid_elapsed_micros,
            render_elapsed_micros: self.render_elapsed_micros,
        }
    }

    fn with_counts(mut self, counts: UiSurfaceRebuildReport) -> Self {
        self.arranged_node_count = counts.arranged_node_count;
        self.render_command_count = counts.render_command_count;
        self.hit_grid_entry_count = counts.hit_grid_entry_count;
        self.hit_grid_cell_count = counts.hit_grid_cell_count;
        self.control_pool_created_count = counts.control_pool_created_count;
        self.control_pool_reused_count = counts.control_pool_reused_count;
        self.control_pool_recycled_count = counts.control_pool_recycled_count;
        self.control_pool_discarded_count = counts.control_pool_discarded_count;
        self
    }
}

impl UiSurface {
    fn rebuild_counts(&self) -> UiSurfaceRebuildReport {
        UiSurfaceRebuildReport {
            arranged_node_count: self.arranged_tree.nodes.len(),
            render_command_count: self.render_extract.list.commands.len(),
            hit_grid_entry_count: self.hit_test.grid.entries.len(),
            hit_grid_cell_count: self.hit_test.grid.cells.len(),
            control_pool_created_count: self.pending_pool_report.created_count,
            control_pool_reused_count: self.pending_pool_report.reused_count,
            control_pool_recycled_count: self.pending_pool_report.recycled_count,
            control_pool_discarded_count: self.pending_pool_report.discarded_count,
            ..UiSurfaceRebuildReport::default()
        }
    }

    fn reset_pending_pool_report(&mut self) {
        self.pending_pool_report = Default::default();
    }

    fn rebuild_render_extract(&mut self, force_rebuild: bool) -> UiSurfaceRenderCacheStats {
        if force_rebuild {
            self.render_cache = Default::default();
        }
        let extract = extract_ui_render_tree_from_arranged(&self.tree, &self.arranged_tree);
        let update = self.render_cache.update(extract, force_rebuild);
        self.render_extract = update.extract;
        update.stats
    }

    pub fn rebuild(&mut self) {
        let dirty_flags = self.dirty_flags();
        let dirty_node_count = dirty_node_count(&self.tree);
        let arranged_start = Instant::now();
        self.arranged_tree = build_arranged_tree(&self.tree);
        let arranged_elapsed_micros = elapsed_micros(arranged_start);
        let hit_start = Instant::now();
        self.hit_test.rebuild_arranged(&self.arranged_tree);
        let hit_grid_elapsed_micros = elapsed_micros(hit_start);
        let render_start = Instant::now();
        let render_stats = self.rebuild_render_extract(true);
        let render_elapsed_micros = elapsed_micros(render_start);
        self.last_rebuild_report = UiSurfaceRebuildReport {
            dirty_flags,
            dirty_node_count,
            arranged_rebuilt: true,
            hit_grid_rebuilt: true,
            render_rebuilt: true,
            layout_visited_node_count: self.tree.nodes.len(),
            layout_geometry_changed_node_count: self.tree.nodes.len(),
            render_command_reused_count: render_stats.reused_command_count,
            render_command_rebuilt_count: render_stats.rebuilt_command_count,
            render_damage_rect_count: render_stats.damage_rect_count,
            arranged_elapsed_micros,
            hit_grid_elapsed_micros,
            render_elapsed_micros,
            ..self.rebuild_counts()
        };
        self.reset_pending_pool_report();
    }

    pub fn dirty_flags(&self) -> UiDirtyFlags {
        self.tree
            .nodes
            .values()
            .fold(UiDirtyFlags::default(), merge_dirty_flags)
    }

    pub fn clear_dirty_flags(&mut self) {
        for node in self.tree.nodes.values_mut() {
            node.dirty = UiDirtyFlags::default();
            node.state_flags.dirty = false;
        }
    }

    pub fn mark_node_dirty(
        &mut self,
        node_id: zircon_runtime_interface::ui::event_ui::UiNodeId,
        dirty: UiDirtyFlags,
    ) -> Result<(), UiTreeError> {
        let node = self
            .tree
            .nodes
            .get_mut(&node_id)
            .ok_or(UiTreeError::MissingNode(node_id))?;
        merge_dirty_flags_into(&mut node.dirty, dirty);
        Ok(())
    }

    pub fn apply_pointer_dispatch_dirty(
        &mut self,
        result: &UiPointerDispatchResult,
    ) -> Result<(), UiTreeError> {
        let mut applied = false;
        for invocation in &result.invocations {
            if let UiPointerDispatchEffect::RequestDirty(flags) = invocation.effect {
                if flags.any() {
                    self.mark_node_dirty(invocation.node_id, flags)?;
                    applied = true;
                }
            }
        }

        if result.requested_dirty.any() && !applied {
            if let Some(target) = result.route.target {
                self.mark_node_dirty(target, result.requested_dirty)?;
            } else {
                let roots = self.tree.roots.clone();
                for root in roots {
                    self.mark_node_dirty(root, result.requested_dirty)?;
                }
            }
        }

        Ok(())
    }

    pub fn rebuild_dirty(
        &mut self,
        root_size: UiSize,
    ) -> Result<UiSurfaceRebuildReport, UiTreeError> {
        let dirty = self.dirty_flags();
        let dirty_node_count = dirty_node_count(&self.tree);
        if !dirty.any() {
            self.last_rebuild_report =
                UiSurfaceRebuildReport::default().with_counts(self.rebuild_counts());
            self.reset_pending_pool_report();
            return Ok(self.last_rebuild_report);
        }

        if dirty.layout || dirty.style || dirty.text || dirty.visible_range {
            let layout_start = Instant::now();
            let layout_stats = compute_incremental_layout_tree(&mut self.tree, root_size)?;
            self.layout_engine_report = merge_incremental_layout_engine_report(
                &self.layout_engine_report,
                &layout_stats.layout_engine_report,
                &layout_stats.visited_node_ids,
                &self.tree,
            );
            let layout_elapsed_micros = elapsed_micros(layout_start);
            let arranged_start = Instant::now();
            self.arranged_tree = build_arranged_tree(&self.tree);
            let arranged_elapsed_micros = elapsed_micros(arranged_start);
            let hit_start = Instant::now();
            self.hit_test.rebuild_arranged(&self.arranged_tree);
            let hit_grid_elapsed_micros = elapsed_micros(hit_start);
            let render_start = Instant::now();
            let render_stats = self.rebuild_render_extract(false);
            let render_elapsed_micros = elapsed_micros(render_start);
            let report = UiSurfaceRebuildReport {
                dirty_flags: dirty,
                dirty_node_count,
                layout_recomputed: true,
                arranged_rebuilt: true,
                hit_grid_rebuilt: true,
                render_rebuilt: true,
                layout_visited_node_count: layout_stats.visited_node_count,
                layout_geometry_changed_node_count: layout_stats.geometry_changed_node_count,
                layout_skipped_node_count: layout_stats.skipped_node_count,
                render_command_reused_count: render_stats.reused_command_count,
                render_command_rebuilt_count: render_stats.rebuilt_command_count,
                render_damage_rect_count: render_stats.damage_rect_count,
                layout_elapsed_micros,
                arranged_elapsed_micros,
                hit_grid_elapsed_micros,
                render_elapsed_micros,
                ..self.rebuild_counts()
            };
            self.last_rebuild_report = report;
            self.clear_dirty_flags();
            self.reset_pending_pool_report();
            return Ok(report);
        }

        let mut report = UiSurfaceRebuildReport {
            dirty_flags: dirty,
            dirty_node_count,
            ..UiSurfaceRebuildReport::default()
        };
        if dirty.hit_test || dirty.input {
            let arranged_start = Instant::now();
            self.arranged_tree = build_arranged_tree(&self.tree);
            report.arranged_elapsed_micros = elapsed_micros(arranged_start);
            let hit_start = Instant::now();
            self.hit_test.rebuild_arranged(&self.arranged_tree);
            report.hit_grid_elapsed_micros = elapsed_micros(hit_start);
            report.arranged_rebuilt = true;
            report.hit_grid_rebuilt = true;
        }
        if dirty.render {
            let render_start = Instant::now();
            let render_stats = self.rebuild_render_extract(false);
            report.render_elapsed_micros = elapsed_micros(render_start);
            report.render_rebuilt = true;
            report.render_command_reused_count = render_stats.reused_command_count;
            report.render_command_rebuilt_count = render_stats.rebuilt_command_count;
            report.render_damage_rect_count = render_stats.damage_rect_count;
        }
        report = UiSurfaceRebuildReport {
            ..report.with_counts(self.rebuild_counts())
        };
        self.last_rebuild_report = report;
        self.clear_dirty_flags();
        self.reset_pending_pool_report();
        Ok(report)
    }

    pub fn compute_layout(&mut self, root_size: UiSize) -> Result<(), UiTreeError> {
        let dirty_flags = self.dirty_flags();
        let dirty_node_count = dirty_node_count(&self.tree);
        let layout_start = Instant::now();
        self.layout_engine_report = compute_layout_tree(&mut self.tree, root_size)?;
        let layout_elapsed_micros = elapsed_micros(layout_start);
        self.rebuild();
        self.last_rebuild_report.layout_recomputed = true;
        self.last_rebuild_report.layout_elapsed_micros = layout_elapsed_micros;
        self.last_rebuild_report.dirty_flags = dirty_flags;
        self.last_rebuild_report.dirty_node_count = dirty_node_count;
        self.last_rebuild_report.layout_visited_node_count = self.tree.nodes.len();
        self.last_rebuild_report.layout_geometry_changed_node_count = self.tree.nodes.len();
        self.last_rebuild_report.layout_skipped_node_count = 0;
        self.clear_dirty_flags();
        self.reset_pending_pool_report();
        Ok(())
    }
}

fn merge_dirty_flags(mut dirty: UiDirtyFlags, node: &UiTreeNode) -> UiDirtyFlags {
    dirty.layout |= node.dirty.layout;
    dirty.hit_test |= node.dirty.hit_test || node.state_flags.dirty;
    dirty.render |= node.dirty.render || node.state_flags.dirty;
    dirty.style |= node.dirty.style;
    dirty.text |= node.dirty.text;
    dirty.input |= node.dirty.input || node.state_flags.dirty;
    dirty.visible_range |= node.dirty.visible_range;
    dirty
}

fn merge_dirty_flags_into(target: &mut UiDirtyFlags, dirty: UiDirtyFlags) {
    target.layout |= dirty.layout;
    target.hit_test |= dirty.hit_test;
    target.render |= dirty.render;
    target.style |= dirty.style;
    target.text |= dirty.text;
    target.input |= dirty.input;
    target.visible_range |= dirty.visible_range;
}

fn dirty_node_count(tree: &UiTree) -> usize {
    tree.nodes
        .values()
        .filter(|node| node.dirty.any() || node.state_flags.dirty)
        .count()
}

// Incremental layout visits only dirty subtrees, while diagnostics expose a surface-level route map.
// Keep untouched container routes and replace any route owned by the visited subtree.
fn merge_incremental_layout_engine_report(
    previous: &UiLayoutEngineSelectionReport,
    incremental: &UiLayoutEngineSelectionReport,
    visited_node_ids: &BTreeSet<UiNodeId>,
    tree: &UiTree,
) -> UiLayoutEngineSelectionReport {
    let mut selections = Vec::new();

    for selection in &previous.selections {
        let Some(node_id) = selection.node_id else {
            continue;
        };
        if tree.nodes.contains_key(&node_id) && !visited_node_ids.contains(&node_id) {
            selections.push(selection.clone());
        }
    }

    selections.extend(incremental.selections.iter().cloned());
    UiLayoutEngineSelectionReport::from_selections(selections)
}

fn elapsed_micros(start: Instant) -> u64 {
    start.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}
