use std::{collections::BTreeSet, time::Instant};

use crate::ui::layout::compute_layout_tree_with_text_measure_cache_and_slot_index;
use crate::ui::surface::{
    arranged_node_indices, arranged_slot_indices, build_arranged_tree,
    invalidation::UiInvalidationReason,
    render::{
        extract_ui_render_commands_for_nodes_with_component_states_and_text_measure_cache,
        extract_ui_render_tree_from_arranged_indexed_with_component_states_and_text_measure_cache_and_control_index,
        UiSurfaceRenderCacheStats,
    },
};
use zircon_runtime_interface::ui::{
    dispatch::{UiPointerDispatchEffect, UiPointerDispatchResult},
    event_ui::UiNodeId,
    layout::UiSize,
    tree::{UiDirtyFlags, UiTree, UiTreeError, UiTreeNode},
};

use super::UiSurface;

mod authored_geometry;
mod incremental;
mod report;
pub use authored_geometry::{UiAuthoredGeometryFallbackReason, UiAuthoredGeometryPublication};
#[cfg(feature = "profiling")]
use report::record_surface_rebuild_profile;
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
        if begin_text_frame {
            self.text_measure_cache.begin_frame();
        }
        let mut extract =
            extract_ui_render_tree_from_arranged_indexed_with_component_states_and_text_measure_cache_and_control_index(
                &self.tree,
                &self.arranged_tree,
                &self.arranged_node_indices,
                Some(&self.arranged_visibility),
                Some(&self.component_states),
                &mut self.text_measure_cache,
                Some(&self.control_index),
                Some(&self.input.popup_anchor_points),
            );
        extract.raster_scale = self.current_raster_scale();
        let update = self.render_cache.update_for_arranged(
            &self.render_extract,
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
                &self.arranged_visibility,
                changed_node_ids,
                Some(&self.component_states),
                &mut self.text_measure_cache,
                Some(&self.control_index),
                Some(&self.input.popup_anchor_points),
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
        let Some(metrics) = self.window_state.metrics else {
            return 1.0;
        };
        let reported_scale = metrics.scale_factor as f32;
        let reported_scale = if reported_scale.is_finite() && reported_scale > 0.0 {
            reported_scale
        } else {
            1.0
        };
        let physical_scale = [
            physical_axis_raster_scale(metrics.physical_size.width, metrics.logical_size.width),
            physical_axis_raster_scale(metrics.physical_size.height, metrics.logical_size.height),
        ]
        .into_iter()
        .flatten()
        .fold(1.0, f32::max);

        // A DPI transition can publish scale and extent in separate events. Keep raster
        // resources at least as dense as the current physical target throughout the transition.
        reported_scale.max(physical_scale).max(1.0)
    }

    pub(crate) fn refresh_render_extract_for_current_tree(&mut self) {
        self.arranged_tree = build_arranged_tree(&self.tree);
        self.refresh_arranged_node_indices();
        self.hit_test
            .rebuild_arranged_indexed(&self.arranged_tree, &self.arranged_node_indices);
        let _ = self.rebuild_render_extract(false);
        self.seed_popup_stack_from_tree_metadata();
        let _ = self.rebuild_projected_hit_test();
        self.rebuild_navigation_index();
        self.mark_surface_frame_dirty();
        self.publish_surface_frame_after_rebuild();
    }

    fn refresh_arranged_node_indices(&mut self) {
        self.arranged_node_indices = arranged_node_indices(&self.arranged_tree);
        self.arranged_visibility
            .rebuild(&self.arranged_tree, &self.arranged_node_indices);
        crate::profile_counter!("runtime", "ui.arranged_visibility.rebuild_count", 1);
        crate::profile_counter!(
            "runtime",
            "ui.arranged_visibility.node_resolve_count",
            self.arranged_tree.nodes.len()
        );
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
        #[cfg(feature = "profiling")]
        let rebuild_profile_start = Instant::now();
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
        self.hit_test
            .rebuild_arranged_indexed(&self.arranged_tree, &self.arranged_node_indices);
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
        let _ = self.rebuild_projected_hit_test();
        self.rebuild_navigation_index();
        self.mark_surface_frame_dirty();
        self.publish_surface_frame_after_rebuild();
        if !requires_layout_rebuild(dirty_flags) {
            let _ = self
                .invalidation
                .commit_pending()
                .expect("surface-owned invalidation transaction must use the current generation");
            self.clear_dirty_flags();
            self.reset_pending_pool_report();
        }
        #[cfg(feature = "profiling")]
        record_surface_rebuild_profile(
            &self.last_rebuild_report,
            elapsed_micros(rebuild_profile_start),
        );
    }

    /// Publishes geometry already authored in each node's layout cache and prepares the
    /// surface for subsequent node-local incremental layout updates at the same root size.
    pub fn rebuild_authored_frames(&mut self, root_size: UiSize) {
        self.rebuild();
        self.last_layout_root_size = Some(root_size);
        self.dirty_index_initialized = true;
        let _ = self
            .invalidation
            .commit_pending()
            .expect("surface-owned invalidation transaction must use the current generation");
        self.clear_dirty_flags();
        self.reset_pending_pool_report();
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
        self.control_index.synchronize_pending(&self.tree);
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
        let affects_layout = dirty.layout || dirty.style || dirty.text || dirty.visible_range;
        {
            let node = self
                .tree
                .nodes
                .get_mut(&node_id)
                .ok_or(UiTreeError::MissingNode(node_id))?;
            if affects_layout {
                node.layout_cache.invalidate_measure();
            }
            if dirty.text {
                node.layout_cache.advance_text_layout_revision();
            }
            merge_dirty_flags_into(&mut node.dirty, dirty);
        }
        if affects_layout {
            self.tree.nodes.mark_layout_dirty_source(node_id);
        }
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

    pub fn compute_layout(&mut self, root_size: UiSize) -> Result<(), UiTreeError> {
        #[cfg(feature = "profiling")]
        let rebuild_profile_start = Instant::now();
        self.invalidate_for_changed_text_font_generation()?;
        let operation_font_generation = self.text_measure_cache.font_database_generation();
        let dirty_summary = dirty_summary(&self.tree);
        self.record_dirty_summary(&dirty_summary);
        let dirty_flags =
            merge_dirty_flag_values(dirty_summary.dirty, self.invalidation.pending_dirty_flags());
        let dirty_node_count = self.invalidation.pending_changed_node_count();
        self.text_measure_cache.begin_frame();
        let layout_start = Instant::now();
        self.layout_engine_report = compute_layout_tree_with_text_measure_cache_and_slot_index(
            &mut self.tree,
            root_size,
            &mut self.text_measure_cache,
            &self.layout_slot_index,
        )?;
        self.refresh_layout_engine_selection_indices();
        self.last_layout_root_size = Some(root_size);
        let layout_elapsed_micros = elapsed_micros(layout_start);
        let arranged_start = Instant::now();
        let next_arranged_tree = build_arranged_tree(&self.tree);
        self.last_layout_geometry_changed_node_ids = next_arranged_tree
            .nodes
            .iter()
            .filter(|next| {
                self.arranged_node(next.node_id).is_none_or(|previous| {
                    previous.frame != next.frame
                        || previous.clip_frame != next.clip_frame
                        || previous.z_index != next.z_index
                        || previous.parent != next.parent
                        || previous.children != next.children
                })
            })
            .map(|node| node.node_id)
            .collect();
        self.arranged_tree = next_arranged_tree;
        self.refresh_arranged_node_indices();
        let arranged_elapsed_micros = elapsed_micros(arranged_start);
        let hit_start = Instant::now();
        self.hit_test
            .rebuild_arranged_indexed(&self.arranged_tree, &self.arranged_node_indices);
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
            layout_measure_probe_node_count: self.tree.nodes.len(),
            layout_arrange_probe_node_count: self.tree.nodes.len(),
            layout_taffy_tree_build_count: self.layout_engine_report.taffy_tree_build_count,
            layout_taffy_tree_node_build_count: self.layout_engine_report.taffy_tree_node_count,
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
        let _ = self.rebuild_projected_hit_test();
        self.rebuild_navigation_index();
        let _ = self
            .invalidation
            .commit_pending()
            .expect("surface-owned invalidation transaction must use the current generation");
        self.clear_dirty_flags();
        self.reset_pending_pool_report();
        self.mark_surface_frame_dirty();
        self.publish_surface_frame_after_rebuild();
        self.record_text_font_generation_layout(operation_font_generation);
        #[cfg(feature = "profiling")]
        record_surface_rebuild_profile(
            &self.last_rebuild_report,
            elapsed_micros(rebuild_profile_start),
        );
        Ok(())
    }
}

fn physical_axis_raster_scale(physical_extent: u32, logical_extent: f32) -> Option<f32> {
    if physical_extent == 0 || !logical_extent.is_finite() || logical_extent <= 0.0 {
        return None;
    }
    let scale = physical_extent as f32 / logical_extent;
    (scale.is_finite() && scale > 0.0).then_some(scale)
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

fn elapsed_micros(start: Instant) -> u64 {
    start.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}
