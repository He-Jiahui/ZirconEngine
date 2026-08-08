use std::{collections::BTreeSet, time::Instant};

use crate::ui::layout::{
    compute_incremental_layout_tree_with_text_measure_cache,
    compute_layout_tree_with_text_measure_cache,
};
use crate::ui::surface::{
    arranged_node_indices, arranged_slot_indices, build_arranged_tree,
    invalidation::UiInvalidationReason,
    patch_arranged_tree_geometry,
    render::{
        extract_ui_render_commands_for_nodes_with_component_states_and_text_measure_cache,
        extract_ui_render_tree_from_arranged_with_component_states_and_text_measure_cache,
        UiSurfaceRenderCacheStats,
    },
};
use zircon_runtime_interface::ui::{
    dispatch::{UiPointerDispatchEffect, UiPointerDispatchResult},
    event_ui::UiNodeId,
    layout::{UiLayoutEngineSelectionReport, UiSize},
    tree::{UiDirtyFlags, UiTree, UiTreeError, UiTreeNode},
};

use super::UiSurface;

mod report;
pub use report::UiSurfaceRebuildReport;
use report::UiTextCacheFrameStats;

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
        self.rebuild_render_extract_with_text_frame(force_rebuild, true)
    }

    fn rebuild_render_extract_with_text_frame(
        &mut self,
        force_rebuild: bool,
        begin_text_frame: bool,
    ) -> UiSurfaceRenderCacheStats {
        if force_rebuild {
            self.render_cache = Default::default();
        }
        if begin_text_frame {
            self.text_measure_cache.begin_frame();
        }
        let mut extract =
            extract_ui_render_tree_from_arranged_with_component_states_and_text_measure_cache(
                &self.tree,
                &self.arranged_tree,
                Some(&self.component_states),
                Some(&mut self.text_measure_cache),
            );
        extract.raster_scale = self.current_raster_scale();
        let update = self.render_cache.update_for_arranged(
            extract,
            force_rebuild,
            &self.arranged_tree,
            &self.arranged_node_indices,
        );
        self.render_extract = update.extract;
        self.text_measure_cache.finish_frame();
        update.stats
    }

    fn patch_render_nodes(
        &mut self,
        changed_node_ids: &BTreeSet<UiNodeId>,
    ) -> Result<UiSurfaceRenderCacheStats, ()> {
        let changed_extract =
            extract_ui_render_commands_for_nodes_with_component_states_and_text_measure_cache(
                &self.tree,
                &self.arranged_tree,
                &self.arranged_node_indices,
                changed_node_ids,
                Some(&self.component_states),
                Some(&mut self.text_measure_cache),
            )?;
        self.render_cache.patch_nodes(
            &mut self.render_extract,
            changed_node_ids,
            changed_extract,
            &self.arranged_tree,
            &self.arranged_node_indices,
        )
    }

    fn text_cache_frame_stats(&self) -> UiTextCacheFrameStats {
        let measure = self.text_measure_cache.frame_measure_report();
        let layout = self.text_measure_cache.frame_layout_report();
        let shape = self.text_measure_cache.frame_shaped_run_report();
        UiTextCacheFrameStats {
            measure_hit_count: measure.hit_count,
            measure_miss_count: measure.miss_count,
            layout_hit_count: layout.hit_count,
            layout_miss_count: layout.miss_count,
            shape_hit_count: shape.hit_count,
            shape_miss_count: shape.miss_count,
        }
    }

    fn current_raster_scale(&self) -> f32 {
        self.window_state
            .metrics
            .map(|metrics| metrics.scale_factor as f32)
            .filter(|scale| scale.is_finite() && *scale > 0.0)
            .unwrap_or(1.0)
    }

    pub(crate) fn refresh_render_extract_for_current_tree(&mut self) {
        self.arranged_tree = build_arranged_tree(&self.tree);
        self.refresh_arranged_node_indices();
        let _ = self.rebuild_render_extract(false);
        self.mark_surface_frame_dirty();
    }

    fn refresh_arranged_node_indices(&mut self) {
        self.arranged_node_indices = arranged_node_indices(&self.arranged_tree);
        self.arranged_slot_indices = arranged_slot_indices(&self.tree);
    }

    fn refresh_layout_engine_selection_indices(&mut self) {
        self.layout_engine_selection_indices = self
            .layout_engine_report
            .selections
            .iter()
            .enumerate()
            .filter_map(|(index, selection)| selection.node_id.map(|node_id| (node_id, index)))
            .collect();
    }

    pub fn rebuild(&mut self) {
        let dirty_summary = dirty_summary(&self.tree);
        self.record_dirty_summary(&dirty_summary);
        let dirty_flags =
            merge_dirty_flag_values(dirty_summary.dirty, self.invalidation.pending_dirty_flags());
        let dirty_node_count = self.invalidation.pending_changed_node_count();
        let arranged_start = Instant::now();
        self.arranged_tree = build_arranged_tree(&self.tree);
        self.refresh_arranged_node_indices();
        let arranged_elapsed_micros = elapsed_micros(arranged_start);
        let hit_start = Instant::now();
        self.hit_test.rebuild_arranged(&self.arranged_tree);
        let hit_grid_elapsed_micros = elapsed_micros(hit_start);
        let render_start = Instant::now();
        let render_stats = self.rebuild_render_extract(true);
        let render_elapsed_micros = elapsed_micros(render_start);
        let text_cache_stats = self.text_cache_frame_stats();
        self.last_rebuild_report = UiSurfaceRebuildReport {
            dirty_flags,
            dirty_node_count,
            arranged_rebuilt: true,
            hit_grid_rebuilt: true,
            render_rebuilt: true,
            arranged_outer_node_visit_count: self.tree.nodes.len(),
            hit_grid_outer_node_visit_count: self.arranged_tree.draw_order.len(),
            render_outer_node_visit_count: self.arranged_tree.draw_order.len(),
            layout_visited_node_count: self.tree.nodes.len(),
            layout_geometry_changed_node_count: self.tree.nodes.len(),
            render_command_reused_count: render_stats.reused_command_count,
            render_command_rebuilt_count: render_stats.rebuilt_command_count,
            render_damage_rect_count: render_stats.damage_rect_count,
            arranged_elapsed_micros,
            hit_grid_elapsed_micros,
            render_elapsed_micros,
            ..self.rebuild_counts()
        }
        .with_text_cache_stats(text_cache_stats);
        self.seed_popup_stack_from_tree_metadata();
        self.mark_surface_frame_dirty();
        if !requires_layout_rebuild(dirty_flags) {
            let _ = self
                .invalidation
                .commit_pending()
                .expect("surface-owned invalidation transaction must use the current generation");
            self.clear_dirty_flags();
            self.reset_pending_pool_report();
        }
    }

    pub fn dirty_flags(&self) -> UiDirtyFlags {
        let mut dirty_candidates = self.dirty_node_ids.clone();
        dirty_candidates.extend(self.invalidation.pending_changed_node_ids());
        dirty_candidates.extend(self.tree.pending_mutation_node_ids().iter().copied());
        let tree_dirty = if !self.dirty_index_initialized {
            dirty_summary(&self.tree).dirty
        } else {
            dirty_summary_for_nodes(&self.tree, &dirty_candidates).dirty
        };
        merge_dirty_flag_values(tree_dirty, self.invalidation.pending_dirty_flags())
    }

    fn record_dirty_summary(&mut self, summary: &UiDirtySummary) {
        for (node_id, dirty) in &summary.changed_nodes {
            self.invalidation.record_dirty(*node_id, *dirty);
            self.dirty_node_ids.insert(*node_id);
        }
    }

    pub fn clear_dirty_flags(&mut self) {
        let mut dirty_node_ids = std::mem::take(&mut self.dirty_node_ids);
        dirty_node_ids.extend(self.tree.pending_mutation_node_ids().iter().copied());
        for node_id in dirty_node_ids {
            if let Some(node) = self.tree.nodes.get_mut(&node_id) {
                node.dirty = UiDirtyFlags::default();
                node.state_flags.dirty = false;
            }
        }
        self.tree.clear_pending_mutation_node_ids();
        self.dirty_index_initialized = true;
        self.invalidation.clear_pending();
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
        if dirty.text && !node.dirty.text {
            node.layout_cache.advance_text_layout_revision();
        }
        merge_dirty_flags_into(&mut node.dirty, dirty);
        self.dirty_node_ids.insert(node_id);
        self.invalidation.record_dirty(node_id, dirty);
        Ok(())
    }

    pub fn invalidate_node(
        &mut self,
        node_id: UiNodeId,
        reason: UiInvalidationReason,
    ) -> Result<(), UiTreeError> {
        self.mark_node_dirty(node_id, reason.dirty_flags())?;
        self.invalidation.record_reason(node_id, reason);
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
        self.last_layout_geometry_changed_node_ids.clear();
        let root_size_changed = self.last_layout_root_size.map_or_else(
            || !self.arranged_tree.nodes.is_empty(),
            |previous| previous != root_size,
        );
        if root_size_changed {
            for root_id in self.tree.roots.clone() {
                self.invalidate_node(root_id, UiInvalidationReason::Layout)?;
            }
        }
        let mut dirty_candidates = self.dirty_node_ids.clone();
        dirty_candidates.extend(self.invalidation.pending_changed_node_ids());
        dirty_candidates.extend(self.tree.pending_mutation_node_ids().iter().copied());
        let dirty_summary = if !self.dirty_index_initialized {
            dirty_summary(&self.tree)
        } else {
            dirty_summary_for_nodes(&self.tree, &dirty_candidates)
        };
        self.dirty_index_initialized = true;
        self.record_dirty_summary(&dirty_summary);
        let dirty =
            merge_dirty_flag_values(dirty_summary.dirty, self.invalidation.pending_dirty_flags());
        let dirty_node_count = self.invalidation.pending_changed_node_count();
        let dirty_node_ids = self.invalidation.pending_changed_node_ids();
        self.render_extract.raster_scale = self.current_raster_scale();
        if !dirty.any() {
            self.last_layout_root_size.get_or_insert(root_size);
            let next_report = UiSurfaceRebuildReport::default().with_counts(self.rebuild_counts());
            if self.last_rebuild_report != next_report {
                self.last_rebuild_report = next_report;
                self.mark_surface_frame_dirty();
            }
            self.reset_pending_pool_report();
            return Ok(self.last_rebuild_report);
        }

        if dirty.layout || dirty.style || dirty.text || dirty.visible_range {
            self.text_measure_cache.begin_frame();
            let layout_start = Instant::now();
            let layout_stats = compute_incremental_layout_tree_with_text_measure_cache(
                &mut self.tree,
                root_size,
                Some(&mut self.text_measure_cache),
                &dirty_node_ids,
                root_size_changed,
                &self.arranged_slot_indices,
            )?;
            self.last_layout_root_size = Some(root_size);
            self.last_layout_geometry_changed_node_ids =
                layout_stats.geometry_changed_node_ids.clone();
            if self.layout_engine_selection_indices.is_empty()
                && !self.layout_engine_report.selections.is_empty()
            {
                self.refresh_layout_engine_selection_indices();
            }
            if !patch_incremental_layout_engine_report(
                &mut self.layout_engine_report,
                &self.layout_engine_selection_indices,
                &layout_stats.layout_engine_report,
                &layout_stats.visited_node_ids,
            ) {
                self.layout_engine_report = merge_incremental_layout_engine_report(
                    &self.layout_engine_report,
                    &layout_stats.layout_engine_report,
                    &layout_stats.visited_node_ids,
                    &self.tree,
                );
                self.refresh_layout_engine_selection_indices();
            }
            let layout_elapsed_micros = elapsed_micros(layout_start);
            let arranged_start = Instant::now();
            let hit_test_is_layout_derived = dirty_summary
                .changed_nodes
                .iter()
                .all(|(_, node_dirty)| !node_dirty.hit_test || node_dirty.layout);
            let local_layout_patch_dirty = (dirty.layout || dirty.style || dirty.text)
                && !dirty.visible_range
                && !dirty.input
                && hit_test_is_layout_derived
                && !layout_stats.visited_node_ids.is_empty();
            let arranged_patched = local_layout_patch_dirty
                && patch_arranged_tree_geometry(
                    &self.tree,
                    &mut self.arranged_tree,
                    &layout_stats.geometry_changed_node_ids,
                    &layout_stats.visited_node_ids,
                    &self.arranged_node_indices,
                    &self.arranged_slot_indices,
                );
            if !arranged_patched {
                self.arranged_tree = build_arranged_tree(&self.tree);
                self.refresh_arranged_node_indices();
            }
            let arranged_elapsed_micros = elapsed_micros(arranged_start);
            let hit_start = Instant::now();
            let hit_grid_patch = (arranged_patched
                && !layout_stats.geometry_changed_node_ids.is_empty())
            .then(|| {
                self.hit_test.patch_arranged_geometry(
                    &self.arranged_tree,
                    &layout_stats.geometry_changed_node_ids,
                    &self.arranged_node_indices,
                )
            });
            let hit_grid_changed = hit_grid_patch
                .as_ref()
                .and_then(|result| result.as_ref().ok().copied())
                .unwrap_or(false);
            let hit_grid_full_rebuild =
                !arranged_patched || hit_grid_patch.as_ref().is_some_and(Result::is_err);
            if hit_grid_full_rebuild {
                self.hit_test.rebuild_arranged(&self.arranged_tree);
            }
            let hit_grid_elapsed_micros = elapsed_micros(hit_start);
            let render_start = Instant::now();
            let mut render_changed_node_ids = dirty_node_ids.clone();
            render_changed_node_ids.extend(layout_stats.geometry_changed_node_ids.iter().copied());
            let render_local_patch = if arranged_patched && local_layout_patch_dirty {
                if dirty.style || dirty.text {
                    match self.patch_render_nodes(&render_changed_node_ids) {
                        Ok(stats) => {
                            self.text_measure_cache.finish_frame();
                            Some(stats)
                        }
                        Err(()) => None,
                    }
                } else {
                    match self.render_cache.patch_geometry(
                        &mut self.render_extract,
                        &self.arranged_tree,
                        &self.arranged_node_indices,
                        &layout_stats.geometry_changed_node_ids,
                    ) {
                        Ok(stats) => {
                            self.text_measure_cache.finish_frame();
                            Some(stats)
                        }
                        Err(()) => match self.patch_render_nodes(&render_changed_node_ids) {
                            Ok(stats) => {
                                self.text_measure_cache.finish_frame();
                                Some(stats)
                            }
                            Err(()) => None,
                        },
                    }
                }
            } else if !dirty.style
                && !dirty.visible_range
                && !dirty.hit_test
                && !dirty.input
                && !render_changed_node_ids.is_empty()
            {
                match self.patch_render_nodes(&render_changed_node_ids) {
                    Ok(stats) => {
                        self.text_measure_cache.finish_frame();
                        Some(stats)
                    }
                    Err(()) => None,
                }
            } else {
                None
            };
            let render_stats = render_local_patch
                .clone()
                .unwrap_or_else(|| self.rebuild_render_extract_with_text_frame(false, false));
            let render_elapsed_micros = elapsed_micros(render_start);
            let text_cache_stats = self.text_cache_frame_stats();
            let report = UiSurfaceRebuildReport {
                dirty_flags: dirty,
                dirty_node_count,
                layout_recomputed: true,
                arranged_rebuilt: true,
                hit_grid_rebuilt: hit_grid_changed || hit_grid_full_rebuild,
                render_rebuilt: true,
                arranged_outer_node_visit_count: if arranged_patched {
                    layout_stats.visited_node_count
                } else {
                    self.tree.nodes.len()
                },
                hit_grid_outer_node_visit_count: if hit_grid_changed {
                    layout_stats.geometry_changed_node_count
                } else if hit_grid_full_rebuild {
                    self.arranged_tree.draw_order.len()
                } else {
                    0
                },
                render_outer_node_visit_count: if render_local_patch.is_some() {
                    render_changed_node_ids.len()
                } else {
                    self.arranged_tree.draw_order.len()
                },
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
            }
            .with_text_cache_stats(text_cache_stats);
            self.last_rebuild_report = report;
            let _ = self
                .invalidation
                .commit_pending()
                .expect("surface-owned invalidation transaction must use the current generation");
            self.clear_dirty_flags();
            self.reset_pending_pool_report();
            self.mark_surface_frame_dirty();
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
            self.refresh_arranged_node_indices();
            report.arranged_elapsed_micros = elapsed_micros(arranged_start);
            let hit_start = Instant::now();
            self.hit_test.rebuild_arranged(&self.arranged_tree);
            report.hit_grid_elapsed_micros = elapsed_micros(hit_start);
            report.arranged_rebuilt = true;
            report.hit_grid_rebuilt = true;
            report.arranged_outer_node_visit_count = self.tree.nodes.len();
            report.hit_grid_outer_node_visit_count = self.arranged_tree.draw_order.len();
        }
        if dirty.render {
            let render_start = Instant::now();
            self.text_measure_cache.begin_frame();
            let render_local_patch =
                if !dirty.hit_test && !dirty.input && !dirty_node_ids.is_empty() {
                    match self.patch_render_nodes(&dirty_node_ids) {
                        Ok(stats) => {
                            self.text_measure_cache.finish_frame();
                            Some(stats)
                        }
                        Err(()) => None,
                    }
                } else {
                    None
                };
            let render_stats = render_local_patch
                .clone()
                .unwrap_or_else(|| self.rebuild_render_extract_with_text_frame(false, false));
            report.render_elapsed_micros = elapsed_micros(render_start);
            report.render_rebuilt = true;
            report.render_command_reused_count = render_stats.reused_command_count;
            report.render_command_rebuilt_count = render_stats.rebuilt_command_count;
            report.render_damage_rect_count = render_stats.damage_rect_count;
            report.render_outer_node_visit_count = if render_local_patch.is_some() {
                dirty_node_ids.len()
            } else {
                self.arranged_tree.draw_order.len()
            };
            report = report.with_text_cache_stats(self.text_cache_frame_stats());
        }
        report = UiSurfaceRebuildReport {
            ..report.with_counts(self.rebuild_counts())
        };
        self.last_rebuild_report = report;
        let _ = self
            .invalidation
            .commit_pending()
            .expect("surface-owned invalidation transaction must use the current generation");
        self.clear_dirty_flags();
        self.reset_pending_pool_report();
        self.mark_surface_frame_dirty();
        Ok(report)
    }

    pub fn compute_layout(&mut self, root_size: UiSize) -> Result<(), UiTreeError> {
        let dirty_summary = dirty_summary(&self.tree);
        self.record_dirty_summary(&dirty_summary);
        let dirty_flags =
            merge_dirty_flag_values(dirty_summary.dirty, self.invalidation.pending_dirty_flags());
        let dirty_node_count = self.invalidation.pending_changed_node_count();
        self.text_measure_cache.begin_frame();
        let layout_start = Instant::now();
        self.layout_engine_report = compute_layout_tree_with_text_measure_cache(
            &mut self.tree,
            root_size,
            Some(&mut self.text_measure_cache),
        )?;
        self.refresh_layout_engine_selection_indices();
        self.last_layout_root_size = Some(root_size);
        let layout_elapsed_micros = elapsed_micros(layout_start);
        let arranged_start = Instant::now();
        self.arranged_tree = build_arranged_tree(&self.tree);
        self.refresh_arranged_node_indices();
        let arranged_elapsed_micros = elapsed_micros(arranged_start);
        let hit_start = Instant::now();
        self.hit_test.rebuild_arranged(&self.arranged_tree);
        let hit_grid_elapsed_micros = elapsed_micros(hit_start);
        let render_start = Instant::now();
        let render_stats = self.rebuild_render_extract_with_text_frame(true, false);
        let render_elapsed_micros = elapsed_micros(render_start);
        let text_cache_stats = self.text_cache_frame_stats();
        self.last_rebuild_report = UiSurfaceRebuildReport {
            dirty_flags,
            dirty_node_count,
            layout_recomputed: true,
            arranged_rebuilt: true,
            hit_grid_rebuilt: true,
            render_rebuilt: true,
            arranged_outer_node_visit_count: self.tree.nodes.len(),
            hit_grid_outer_node_visit_count: self.arranged_tree.draw_order.len(),
            render_outer_node_visit_count: self.arranged_tree.draw_order.len(),
            layout_visited_node_count: self.tree.nodes.len(),
            layout_geometry_changed_node_count: self.tree.nodes.len(),
            layout_skipped_node_count: 0,
            render_command_reused_count: render_stats.reused_command_count,
            render_command_rebuilt_count: render_stats.rebuilt_command_count,
            render_damage_rect_count: render_stats.damage_rect_count,
            layout_elapsed_micros,
            arranged_elapsed_micros,
            hit_grid_elapsed_micros,
            render_elapsed_micros,
            ..self.rebuild_counts()
        }
        .with_text_cache_stats(text_cache_stats);
        self.seed_popup_stack_from_tree_metadata();
        let _ = self
            .invalidation
            .commit_pending()
            .expect("surface-owned invalidation transaction must use the current generation");
        self.clear_dirty_flags();
        self.reset_pending_pool_report();
        self.mark_surface_frame_dirty();
        Ok(())
    }
}

#[derive(Debug, Default)]
struct UiDirtySummary {
    dirty: UiDirtyFlags,
    changed_nodes: Vec<(UiNodeId, UiDirtyFlags)>,
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

fn dirty_summary(tree: &UiTree) -> UiDirtySummary {
    tree.nodes
        .values()
        .fold(UiDirtySummary::default(), |mut summary, node| {
            let node_dirty = effective_node_dirty(node);
            if node_dirty.any() {
                summary.dirty = merge_dirty_flag_values(summary.dirty, node_dirty);
                summary.changed_nodes.push((node.node_id, node_dirty));
            }
            summary
        })
}

fn dirty_summary_for_nodes(tree: &UiTree, node_ids: &BTreeSet<UiNodeId>) -> UiDirtySummary {
    node_ids
        .iter()
        .filter_map(|node_id| tree.nodes.get(node_id))
        .fold(UiDirtySummary::default(), |mut summary, node| {
            let node_dirty = effective_node_dirty(node);
            if node_dirty.any() {
                summary.dirty = merge_dirty_flag_values(summary.dirty, node_dirty);
                summary.changed_nodes.push((node.node_id, node_dirty));
            }
            summary
        })
}

fn effective_node_dirty(node: &UiTreeNode) -> UiDirtyFlags {
    let mut dirty = node.dirty;
    if node.state_flags.dirty {
        dirty.hit_test = true;
        dirty.render = true;
        dirty.input = true;
    }
    dirty
}

fn merge_dirty_flag_values(mut target: UiDirtyFlags, dirty: UiDirtyFlags) -> UiDirtyFlags {
    merge_dirty_flags_into(&mut target, dirty);
    target
}

fn requires_layout_rebuild(dirty: UiDirtyFlags) -> bool {
    dirty.layout || dirty.style || dirty.text || dirty.visible_range
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

fn patch_incremental_layout_engine_report(
    previous: &mut UiLayoutEngineSelectionReport,
    previous_indices: &std::collections::BTreeMap<UiNodeId, usize>,
    incremental: &UiLayoutEngineSelectionReport,
    visited_node_ids: &BTreeSet<UiNodeId>,
) -> bool {
    let incremental_by_node = incremental
        .selections
        .iter()
        .filter_map(|selection| selection.node_id.map(|node_id| (node_id, selection)))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut replacements = Vec::new();

    for node_id in visited_node_ids {
        match (
            previous_indices.get(node_id).copied(),
            incremental_by_node.get(node_id).copied(),
        ) {
            (None, None) => {}
            (Some(index), Some(next)) => {
                let Some(current) = previous.selections.get(index) else {
                    return false;
                };
                if current.node_id != Some(*node_id) {
                    return false;
                }
                if current != next {
                    replacements.push((index, next.clone()));
                }
            }
            _ => return false,
        }
    }

    if replacements.is_empty() {
        return true;
    }
    for (index, replacement) in replacements {
        if !previous.replace_selection_at(index, replacement) {
            return false;
        }
    }
    true
}

fn elapsed_micros(start: Instant) -> u64 {
    start.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}
