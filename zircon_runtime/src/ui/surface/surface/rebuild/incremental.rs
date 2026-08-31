use std::{collections::BTreeSet, time::Instant};

use crate::ui::layout::compute_incremental_layout_tree_with_text_measure_cache;
use crate::ui::surface::{
    build_arranged_tree, patch_arranged_tree_geometry, patch_arranged_tree_input,
};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::{UiLayoutEngineSelectionReport, UiSize},
    tree::{UiTree, UiTreeError},
};

#[cfg(feature = "profiling")]
use super::record_surface_rebuild_profile;
use super::{
    dirty_summary, dirty_summary_for_nodes, elapsed_micros, merge_dirty_flag_values,
    requires_layout_rebuild, UiSurface, UiSurfaceRebuildReport,
};

// Incremental layout is beneficial only while the dirty set is genuinely smaller than the
// surface. A bounded budget avoids probing a large dirty graph and then falling back through the
// arranged, hit, and render passes a second time.
const UI_LAYOUT_INCREMENTAL_MAX_DIRTY_RATIO_DENOMINATOR: usize = 4;
const UI_LAYOUT_INCREMENTAL_MAX_DIRTY_NODE_COUNT: usize = 256;

impl UiSurface {
    pub fn rebuild_dirty(
        &mut self,
        root_size: UiSize,
    ) -> Result<UiSurfaceRebuildReport, UiTreeError> {
        #[cfg(feature = "profiling")]
        let rebuild_profile_start = Instant::now();
        if self.invalidate_for_changed_text_font_generation()? {
            crate::profile_counter!("runtime", "ui.text.font_generation_rebuild_count", 1);
            self.compute_layout(root_size)?;
            return Ok(self.last_rebuild_report);
        }
        self.last_layout_geometry_changed_node_ids.clear();
        let root_size_changed = self.last_layout_root_size.map_or_else(
            || !self.arranged_tree.nodes.is_empty(),
            |previous| previous != root_size,
        );
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
        let mut dirty =
            merge_dirty_flag_values(dirty_summary.dirty, self.invalidation.pending_dirty_flags());
        let layout_dirty_before_resize = requires_layout_rebuild(dirty);
        if root_size_changed {
            dirty.layout = true;
        }
        let dirty_node_count = self.invalidation.pending_changed_node_count();
        let dirty_node_ids = self.invalidation.pending_changed_node_ids();
        self.render_extract.raster_scale = self.current_raster_scale();
        if !dirty.any() {
            self.last_layout_root_size.get_or_insert(root_size);
            let next_report = UiSurfaceRebuildReport::default().with_counts(self.rebuild_counts());
            if self.last_rebuild_report != next_report {
                self.last_rebuild_report = next_report;
                self.mark_surface_frame_metadata_dirty();
            }
            self.reset_pending_pool_report();
            return Ok(self.last_rebuild_report);
        }

        let mut layout_dirty_node_ids = dirty_summary
            .changed_nodes
            .iter()
            .filter_map(|(node_id, node_dirty)| {
                requires_layout_rebuild(*node_dirty).then_some(*node_id)
            })
            .collect::<BTreeSet<_>>();
        layout_dirty_node_ids.extend(self.tree.pending_layout_source_node_ids().iter().copied());
        let layout_dirty_node_count =
            if layout_dirty_before_resize && layout_dirty_node_ids.is_empty() {
                self.tree.nodes.len()
            } else {
                layout_dirty_node_ids.len()
            };
        if requires_layout_rebuild(dirty)
            && should_use_full_layout_rebuild(layout_dirty_node_count, self.tree.nodes.len())
        {
            crate::profile_counter!("runtime", "ui.layout.full_rebuild_threshold_count", 1);
            self.compute_layout(root_size)?;
            return Ok(self.last_rebuild_report);
        }

        if dirty.layout || dirty.style || dirty.text || dirty.visible_range {
            self.text_measure_cache.begin_frame();
            let layout_start = Instant::now();
            let layout_stats = compute_incremental_layout_tree_with_text_measure_cache(
                &mut self.tree,
                root_size,
                &mut self.text_measure_cache,
                &layout_dirty_node_ids,
                root_size_changed,
                &self.layout_slot_index,
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
                &layout_stats.layout_engine_route_node_ids,
                &layout_stats.removed_node_ids,
            ) {
                self.layout_engine_report = merge_incremental_layout_engine_report(
                    &self.layout_engine_report,
                    &layout_stats.layout_engine_report,
                    &layout_stats.layout_engine_route_node_ids,
                    &self.tree,
                );
                self.refresh_layout_engine_selection_indices();
            }
            let layout_elapsed_micros = elapsed_micros(layout_start);
            let arranged_start = Instant::now();
            let layout_input_patch_node_ids = dirty_summary
                .changed_nodes
                .iter()
                .filter_map(|(node_id, node_dirty)| {
                    (node_dirty.input
                        || (node_dirty.hit_test && !requires_layout_rebuild(*node_dirty)))
                    .then_some(*node_id)
                })
                .collect::<BTreeSet<_>>();
            let input_patch_affected_node_ids = if layout_input_patch_node_ids.is_empty() {
                Some(BTreeSet::new())
            } else {
                patch_arranged_tree_input(
                    &self.tree,
                    &mut self.arranged_tree,
                    &layout_input_patch_node_ids,
                    &layout_stats.geometry_changed_node_ids,
                    &self.arranged_node_indices,
                    &self.arranged_slot_indices,
                )
            };
            let local_layout_patch_dirty =
                (dirty.layout || dirty.style || dirty.text || dirty.visible_range)
                    && input_patch_affected_node_ids.is_some()
                    && !layout_stats.visited_node_ids.is_empty();
            let arranged_geometry_patch_node_ids = local_layout_patch_dirty
                .then(|| {
                    patch_arranged_tree_geometry(
                        &self.tree,
                        &mut self.arranged_tree,
                        &layout_stats.geometry_changed_node_ids,
                        &layout_stats.visited_node_ids,
                        &self.arranged_node_indices,
                        &self.arranged_slot_indices,
                    )
                })
                .flatten();
            let arranged_patch_node_ids =
                arranged_geometry_patch_node_ids
                    .as_ref()
                    .map(|geometry_patch_node_ids| {
                        let mut combined_patch_node_ids = geometry_patch_node_ids.clone();
                        combined_patch_node_ids.extend(
                            input_patch_affected_node_ids
                                .as_ref()
                                .expect("local layout patch requires a validated input patch")
                                .iter()
                                .copied(),
                        );
                        combined_patch_node_ids
                    });
            let arranged_patched = arranged_patch_node_ids.is_some();
            if !arranged_patched {
                self.arranged_tree = build_arranged_tree(&self.tree);
                self.refresh_arranged_node_indices();
            }
            let arranged_elapsed_micros = elapsed_micros(arranged_start);
            let hit_start = Instant::now();
            let hit_grid_patch = arranged_patch_node_ids
                .as_ref()
                .filter(|node_ids| !node_ids.is_empty())
                .map(|node_ids| {
                    if layout_input_patch_node_ids.is_empty() {
                        self.hit_test.patch_arranged_geometry(
                            &self.arranged_tree,
                            node_ids,
                            &self.arranged_node_indices,
                        )
                    } else {
                        self.hit_test.patch_arranged_input(
                            &self.arranged_tree,
                            node_ids,
                            &self.arranged_node_indices,
                        )
                    }
                });
            let hit_grid_changed = hit_grid_patch
                .as_ref()
                .and_then(|result| result.as_ref().ok().copied())
                .unwrap_or(false);
            let hit_grid_full_rebuild =
                !arranged_patched || hit_grid_patch.as_ref().is_some_and(Result::is_err);
            if hit_grid_full_rebuild {
                self.hit_test
                    .rebuild_arranged_indexed(&self.arranged_tree, &self.arranged_node_indices);
            }
            let hit_grid_elapsed_micros = elapsed_micros(hit_start);
            let render_start = Instant::now();
            let render_payload_dirty_node_ids = dirty_summary
                .changed_nodes
                .iter()
                .filter_map(|(node_id, node_dirty)| {
                    (node_dirty.render
                        || node_dirty.style
                        || node_dirty.text
                        || node_dirty.visible_range)
                        .then_some(*node_id)
                })
                .collect::<BTreeSet<_>>();
            let mut render_changed_node_ids = render_payload_dirty_node_ids.clone();
            render_changed_node_ids.extend(layout_stats.geometry_changed_node_ids.iter().copied());
            let mut popup_dependency_node_ids = dirty_node_ids.clone();
            popup_dependency_node_ids
                .extend(layout_stats.geometry_changed_node_ids.iter().copied());
            let popup_dependency_impact = self.popup_dependency_impact(&popup_dependency_node_ids);
            let render_local_patch = if !popup_dependency_impact.render_extract
                && arranged_patched
                && local_layout_patch_dirty
            {
                if !render_payload_dirty_node_ids.is_empty() {
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
                        arranged_geometry_patch_node_ids
                            .as_ref()
                            .expect("geometry patch ids must exist on the local render path"),
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
            } else if !popup_dependency_impact.render_extract
                && !dirty.style
                && !(dirty.visible_range || dirty.hit_test || dirty.input)
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
                    arranged_patch_node_ids.as_ref().map_or(0, BTreeSet::len)
                } else {
                    self.tree.nodes.len()
                },
                hit_grid_outer_node_visit_count: if hit_grid_changed {
                    arranged_patch_node_ids.as_ref().map_or(0, BTreeSet::len)
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
                layout_measure_probe_node_count: layout_stats.layout_measure_probe_node_count,
                layout_arrange_probe_node_count: layout_stats.layout_arrange_probe_node_count,
                layout_taffy_tree_build_count: layout_stats
                    .layout_engine_report
                    .taffy_tree_build_count,
                layout_taffy_tree_node_build_count: layout_stats
                    .layout_engine_report
                    .taffy_tree_node_count,
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
            if popup_dependency_impact.stack_reconciliation {
                self.seed_popup_stack_from_tree_metadata();
            }
            let projected_hit_changed = self.synchronize_projected_hit_test(
                arranged_patch_node_ids
                    .as_ref()
                    .unwrap_or(&layout_stats.geometry_changed_node_ids),
                hit_grid_full_rebuild,
            );
            let navigation_projected_geometry_requires_rebuild = projected_hit_changed
                && match self.navigation_index_patch_projected_geometry() {
                    Ok(patched_node_count) => {
                        if patched_node_count > 0 {
                            crate::profile_counter!(
                                "runtime",
                                "ui.navigation_index.projected_geometry_patch_node_count",
                                patched_node_count
                            );
                        }
                        false
                    }
                    Err(()) => true,
                };
            if projected_hit_changed && !navigation_projected_geometry_requires_rebuild {
                crate::profile_counter!(
                    "runtime",
                    "ui.navigation_index.projected_geometry_skip_count",
                    1
                );
            }
            let navigation_geometry_requires_rebuild = self
                .navigation_index_patch_changed_geometry(
                    arranged_patch_node_ids
                        .as_ref()
                        .unwrap_or(&layout_stats.geometry_changed_node_ids),
                    &layout_stats.removed_node_ids,
                )
                .map(|patched_node_count| {
                    if patched_node_count > 0 {
                        crate::profile_counter!(
                            "runtime",
                            "ui.navigation_index.geometry_patch_node_count",
                            patched_node_count
                        );
                    }
                    false
                })
                .unwrap_or(true);
            let navigation_semantics_dirty =
                dirty.style || dirty.text || dirty.visible_range || dirty.input;
            let mut navigation_semantic_node_ids = dirty_node_ids.clone();
            if dirty.visible_range {
                navigation_semantic_node_ids.extend(layout_stats.visited_node_ids.iter().copied());
            }
            let navigation_semantics_changed = navigation_semantics_dirty
                && self.navigation_index_needs_semantics_rebuild(
                    &navigation_semantic_node_ids,
                    &layout_stats.removed_node_ids,
                );
            if navigation_semantics_dirty && !navigation_semantics_changed {
                if dirty.style || dirty.text || dirty.visible_range {
                    crate::profile_counter!(
                        "runtime",
                        "ui.navigation_index.style_text_visible_range_semantics_skip_count",
                        1
                    );
                }
                if dirty.input {
                    crate::profile_counter!(
                        "runtime",
                        "ui.navigation_index.input_semantics_skip_count",
                        1
                    );
                }
            }
            if navigation_geometry_requires_rebuild
                || navigation_semantics_changed
                || popup_dependency_impact.stack_reconciliation
                || navigation_projected_geometry_requires_rebuild
            {
                self.rebuild_navigation_index();
            } else {
                crate::profile_counter!("runtime", "ui.navigation_index.geometry_skip_count", 1);
            }
            let _ = self
                .invalidation
                .commit_pending()
                .expect("surface-owned invalidation transaction must use the current generation");
            self.clear_dirty_flags();
            self.reset_pending_pool_report();
            self.mark_surface_frame_rebuild_dirty(
                report.arranged_rebuilt,
                report.render_rebuilt,
                report.hit_grid_rebuilt || projected_hit_changed,
                render_local_patch
                    .as_ref()
                    .map(|_| &render_changed_node_ids),
            );
            self.publish_surface_frame_after_rebuild();
            #[cfg(feature = "profiling")]
            record_surface_rebuild_profile(&report, elapsed_micros(rebuild_profile_start));
            return Ok(report);
        }

        let mut report = UiSurfaceRebuildReport {
            dirty_flags: dirty,
            dirty_node_count,
            ..UiSurfaceRebuildReport::default()
        };
        let mut input_patch_node_ids = None;
        let mut render_local_patch_node_ids = None::<BTreeSet<UiNodeId>>;
        let mut dirty_popup_dependency_impact = None;
        let mut hit_grid_full_rebuild = false;
        if dirty.hit_test || dirty.input {
            let arranged_start = Instant::now();
            input_patch_node_ids = patch_arranged_tree_input(
                &self.tree,
                &mut self.arranged_tree,
                &dirty_node_ids,
                &BTreeSet::new(),
                &self.arranged_node_indices,
                &self.arranged_slot_indices,
            );
            if input_patch_node_ids.is_none() {
                self.arranged_tree = build_arranged_tree(&self.tree);
                self.refresh_arranged_node_indices();
            }
            report.arranged_elapsed_micros = elapsed_micros(arranged_start);
            let hit_start = Instant::now();
            let hit_grid_patch = input_patch_node_ids.as_ref().map(|node_ids| {
                self.hit_test.patch_arranged_input(
                    &self.arranged_tree,
                    node_ids,
                    &self.arranged_node_indices,
                )
            });
            let hit_grid_changed = hit_grid_patch
                .as_ref()
                .and_then(|result| result.as_ref().ok().copied())
                .unwrap_or(false);
            hit_grid_full_rebuild = input_patch_node_ids.is_none()
                || hit_grid_patch.as_ref().is_some_and(Result::is_err);
            if hit_grid_full_rebuild {
                self.hit_test
                    .rebuild_arranged_indexed(&self.arranged_tree, &self.arranged_node_indices);
            }
            report.hit_grid_elapsed_micros = elapsed_micros(hit_start);
            report.arranged_rebuilt = true;
            report.hit_grid_rebuilt = hit_grid_changed || hit_grid_full_rebuild;
            report.arranged_outer_node_visit_count = input_patch_node_ids
                .as_ref()
                .map_or(self.tree.nodes.len(), BTreeSet::len);
            report.hit_grid_outer_node_visit_count = if hit_grid_changed {
                input_patch_node_ids.as_ref().map_or(0, BTreeSet::len)
            } else if hit_grid_full_rebuild {
                self.arranged_tree.draw_order.len()
            } else {
                0
            };
        }
        if dirty.render {
            let render_start = Instant::now();
            self.text_measure_cache.begin_frame();
            let render_patch_uses_dirty_node_ids = input_patch_node_ids
                .as_ref()
                .filter(|_| dirty.hit_test || dirty.input)
                .is_none();
            let render_patch_node_ids = input_patch_node_ids
                .as_ref()
                .filter(|_| dirty.hit_test || dirty.input)
                .unwrap_or(&dirty_node_ids);
            let render_patch_eligible = (!dirty.hit_test && !dirty.input
                || input_patch_node_ids.is_some())
                && !render_patch_node_ids.is_empty();
            let render_popup_dependency_impact =
                render_patch_eligible.then(|| self.popup_dependency_impact(render_patch_node_ids));
            if render_patch_uses_dirty_node_ids {
                dirty_popup_dependency_impact = render_popup_dependency_impact;
            }
            let render_local_patch =
                if render_popup_dependency_impact.is_some_and(|impact| !impact.render_extract) {
                    match self.patch_render_nodes(render_patch_node_ids) {
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
            render_local_patch_node_ids = render_local_patch
                .as_ref()
                .map(|_| render_patch_node_ids.clone());
            report.render_elapsed_micros = elapsed_micros(render_start);
            report.render_rebuilt = true;
            report.render_command_reused_count = render_stats.reused_command_count;
            report.render_command_rebuilt_count = render_stats.rebuilt_command_count;
            report.render_damage_rect_count = render_stats.damage_rect_count;
            report.render_outer_node_visit_count = if render_local_patch.is_some() {
                render_patch_node_ids.len()
            } else {
                self.arranged_tree.draw_order.len()
            };
            report = report.with_text_cache_stats(self.text_cache_frame_stats());
        }
        report = UiSurfaceRebuildReport {
            ..report.with_counts(self.rebuild_counts())
        };
        self.last_rebuild_report = report;
        let popup_dependency_impact = dirty_popup_dependency_impact
            .unwrap_or_else(|| self.popup_dependency_impact(&dirty_node_ids));
        let popup_stack_reconciled = popup_dependency_impact.stack_reconciliation;
        if popup_stack_reconciled {
            self.seed_popup_stack_from_tree_metadata();
        }
        let hit_grid_affected_node_ids = input_patch_node_ids.unwrap_or_default();
        let projected_geometry_changed =
            self.synchronize_projected_hit_test(&hit_grid_affected_node_ids, hit_grid_full_rebuild);
        let navigation_projected_geometry_requires_rebuild = projected_geometry_changed
            && match self.navigation_index_patch_projected_geometry() {
                Ok(patched_node_count) => {
                    if patched_node_count > 0 {
                        crate::profile_counter!(
                            "runtime",
                            "ui.navigation_index.projected_geometry_patch_node_count",
                            patched_node_count
                        );
                    }
                    false
                }
                Err(()) => true,
            };
        if projected_geometry_changed && !navigation_projected_geometry_requires_rebuild {
            crate::profile_counter!(
                "runtime",
                "ui.navigation_index.projected_geometry_skip_count",
                1
            );
        }
        let navigation_semantics_dirty =
            dirty.style || dirty.text || dirty.visible_range || dirty.hit_test || dirty.input;
        let navigation_semantics_changed = if navigation_semantics_dirty {
            let changed =
                self.navigation_index_needs_semantics_rebuild(&dirty_node_ids, &BTreeSet::new());
            if !changed && (dirty.hit_test || dirty.input) {
                crate::profile_counter!(
                    "runtime",
                    "ui.navigation_index.input_semantics_skip_count",
                    1
                );
            }
            if !changed && (dirty.style || dirty.text || dirty.visible_range) {
                crate::profile_counter!(
                    "runtime",
                    "ui.navigation_index.style_text_visible_range_semantics_skip_count",
                    1
                );
            }
            changed
        } else {
            false
        };
        if navigation_projected_geometry_requires_rebuild
            || popup_stack_reconciled
            || navigation_semantics_changed
        {
            self.rebuild_navigation_index();
        }
        let _ = self
            .invalidation
            .commit_pending()
            .expect("surface-owned invalidation transaction must use the current generation");
        self.clear_dirty_flags();
        self.reset_pending_pool_report();
        self.mark_surface_frame_rebuild_dirty(
            report.arranged_rebuilt,
            report.render_rebuilt,
            report.hit_grid_rebuilt || projected_geometry_changed,
            render_local_patch_node_ids.as_ref(),
        );
        self.publish_surface_frame_after_rebuild();
        #[cfg(feature = "profiling")]
        record_surface_rebuild_profile(&report, elapsed_micros(rebuild_profile_start));
        Ok(report)
    }
}

fn should_use_full_layout_rebuild(layout_dirty_node_count: usize, total_node_count: usize) -> bool {
    if layout_dirty_node_count == 0 {
        return false;
    }
    let incremental_budget = total_node_count
        .saturating_div(UI_LAYOUT_INCREMENTAL_MAX_DIRTY_RATIO_DENOMINATOR)
        .max(1)
        .min(UI_LAYOUT_INCREMENTAL_MAX_DIRTY_NODE_COUNT);
    layout_dirty_node_count > incremental_budget
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
    removed_node_ids: &BTreeSet<UiNodeId>,
) -> bool {
    if removed_node_ids
        .iter()
        .any(|node_id| previous_indices.contains_key(node_id))
    {
        return false;
    }
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
