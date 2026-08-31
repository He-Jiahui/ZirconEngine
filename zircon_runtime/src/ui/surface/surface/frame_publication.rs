use std::{collections::BTreeSet, ops::Range, sync::Arc};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    surface::{UiHitTestQuery, UiRenderFrameExtract, UiRenderFramePatchStats, UiSurfaceFrame},
};

use crate::ui::{
    surface::{
        arranged_focus_path_matches_indexed,
        frame_hit_test::hit_test_surface_frame_with_query_using_index,
    },
    tree::UiHitTestResult,
};

use super::UiSurface;

#[derive(Clone, Debug)]
pub(super) struct UiSurfaceFramePublication {
    dirty: bool,
    dirty_domains: UiSurfaceFrameDirtyDomains,
    render_patch_ranges: Vec<Range<usize>>,
    render_requires_full_snapshot: bool,
    generation: u64,
    frame: Option<Arc<UiSurfaceFrame>>,
}

impl Default for UiSurfaceFramePublication {
    fn default() -> Self {
        Self {
            dirty: true,
            dirty_domains: UiSurfaceFrameDirtyDomains::all(),
            render_patch_ranges: Vec::new(),
            render_requires_full_snapshot: true,
            generation: 0,
            frame: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct UiSurfaceFrameDirtyDomains {
    layout: bool,
    render: bool,
    hit_test: bool,
    focus: bool,
    pipeline: bool,
}

impl UiSurfaceFrameDirtyDomains {
    const fn all() -> Self {
        Self {
            layout: true,
            render: true,
            hit_test: true,
            focus: true,
            pipeline: true,
        }
    }

    fn merge(&mut self, other: Self) {
        self.layout |= other.layout;
        self.render |= other.render;
        self.hit_test |= other.hit_test;
        self.focus |= other.focus;
        self.pipeline |= other.pipeline;
    }
}

// Publication state is an ephemeral read cache and does not change UiSurface value equality.
impl PartialEq for UiSurfaceFramePublication {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl UiSurface {
    pub(super) fn mark_surface_frame_dirty(&mut self) {
        let publication = self.frame_publication.get_mut();
        publication.dirty = true;
        publication.render_patch_ranges.clear();
        publication.render_requires_full_snapshot = true;
        publication
            .dirty_domains
            .merge(UiSurfaceFrameDirtyDomains::all());
    }

    pub(super) fn mark_surface_frame_rebuild_dirty(
        &mut self,
        layout: bool,
        render: bool,
        hit_test: bool,
        render_local_patch_node_ids: Option<&BTreeSet<UiNodeId>>,
    ) {
        let render_patch_ranges = render_local_patch_node_ids.map(|node_ids| {
            node_ids
                .iter()
                .filter_map(|node_id| {
                    self.render_cache
                        .commands_for_node(&self.render_extract, *node_id)
                        .map(|(start, commands)| start..start + commands.len())
                })
                .collect::<Vec<_>>()
        });
        let publication = self.frame_publication.get_mut();
        publication.dirty = true;
        publication.record_render_patch_ranges(render, render_patch_ranges);
        publication.dirty_domains.merge(UiSurfaceFrameDirtyDomains {
            layout,
            render,
            hit_test,
            focus: false,
            pipeline: true,
        });
    }

    pub(super) fn mark_surface_frame_metadata_dirty(&mut self) {
        let publication = self.frame_publication.get_mut();
        publication.dirty = true;
        publication.dirty_domains.pipeline = true;
    }

    /// Materialize the retained frame before a rebuild-owned caller returns to its consumer.
    ///
    /// `surface_frame` keeps a lazy fallback for transient window/focus changes, but the
    /// authoritative layout/render/hit domains are already available at the rebuild boundary.
    /// Publishing here prevents the first render or host consumer from paying that domain clone.
    pub(crate) fn publish_surface_frame_after_rebuild(&mut self) {
        let _ = self.surface_frame();
    }

    /// Return the persistent render domain prepared at the rebuild boundary.
    ///
    /// Non-render metadata may leave the outer frame dirty without invalidating this domain.
    pub(crate) fn render_frame_extract(&self) -> Arc<UiRenderFrameExtract> {
        let publication = self.frame_publication.borrow();
        let retained = publication.frame.as_ref().filter(|frame| {
            !publication.dirty_domains.render
                && frame.render_extract.tree_id == self.render_extract.tree_id
                && frame.render_extract.raster_scale.to_bits()
                    == self.render_extract.raster_scale.to_bits()
        });
        if let Some(frame) = retained {
            return Arc::clone(&frame.render_extract);
        }
        drop(publication);
        Arc::clone(&self.surface_frame().render_extract)
    }

    pub(super) fn hit_test_published_surface_frame_with_query(
        &self,
        query: UiHitTestQuery,
    ) -> UiHitTestResult {
        let publication = self.frame_publication.borrow();
        if let Some(surface_frame) = publication.frame.as_deref() {
            return hit_test_surface_frame_with_query_using_index(
                surface_frame,
                query,
                &self.hit_test,
            );
        }
        drop(publication);

        self.hit_test.hit_test_owned_grid_arranged_with_query(
            self.projected_hit_test
                .authoritative_grid(&self.hit_test.grid),
            &self.arranged_tree,
            query,
        )
    }

    pub fn surface_frame(&self) -> Arc<UiSurfaceFrame> {
        let mut publication = self.frame_publication.borrow_mut();
        let previous_frame = publication.frame.as_deref();
        let window_state_changed =
            previous_frame.is_none_or(|frame| frame.window_state != self.window_state);
        let focus_state_changed =
            previous_frame.is_none_or(|frame| frame.focus_state.as_ref() != &self.focus);
        let transient_state_changed = window_state_changed || focus_state_changed;

        if publication.dirty || transient_state_changed {
            #[cfg(feature = "profiling")]
            let publication_start = std::time::Instant::now();
            publication.generation = publication.generation.saturating_add(1);
            let generation = publication.generation;
            let dirty_domains = publication.dirty_domains;
            let previous_frame = publication.frame.as_ref();
            let layout_changed = dirty_domains.layout || previous_frame.is_none();
            let render_metadata_changed = previous_frame.is_none_or(|frame| {
                frame.render_extract.tree_id != self.render_extract.tree_id
                    || frame.render_extract.raster_scale.to_bits()
                        != self.render_extract.raster_scale.to_bits()
            });
            let render_changed =
                dirty_domains.render || render_metadata_changed || previous_frame.is_none();
            let hit_test_changed = dirty_domains.hit_test || previous_frame.is_none();
            let focus_state_domain_changed =
                dirty_domains.focus || focus_state_changed || previous_frame.is_none();
            let focus_path_changed = dirty_domains.focus
                || previous_frame.is_none_or(|frame| {
                    frame.focus_path.focused != self.focus.focused
                        || (layout_changed
                            && !arranged_focus_path_matches_indexed(
                                &self.arranged_tree,
                                &self.arranged_node_indices,
                                &frame.focus_path,
                                self.focus.focused,
                            ))
                });
            let focus_changed = focus_state_domain_changed || focus_path_changed;
            let pipeline_changed = dirty_domains.pipeline || previous_frame.is_none();
            let mut domain_generations = previous_frame
                .as_deref()
                .map(|frame| frame.domain_generations)
                .unwrap_or_default();
            advance_domain_generation(&mut domain_generations.layout, layout_changed);
            advance_domain_generation(&mut domain_generations.render, render_changed);
            advance_domain_generation(&mut domain_generations.hit_test, hit_test_changed);
            advance_domain_generation(&mut domain_generations.focus, focus_changed);
            advance_domain_generation(&mut domain_generations.pipeline, pipeline_changed);
            advance_domain_generation(&mut domain_generations.window, window_state_changed);

            let arranged_tree = shared_domain(
                previous_frame.map(|frame| &frame.arranged_tree),
                layout_changed,
                || self.arranged_tree.clone(),
            );
            let layout_engine_report = shared_domain(
                previous_frame.map(|frame| &frame.layout_engine_report),
                layout_changed,
                || self.layout_engine_report.clone(),
            );
            let (render_extract, render_patch_stats, render_full_snapshot_built) =
                published_render_domain(
                    previous_frame.map(|frame| &frame.render_extract),
                    render_changed,
                    publication.render_requires_full_snapshot,
                    &publication.render_patch_ranges,
                    &self.render_extract,
                );
            let hit_grid = shared_domain(
                previous_frame.map(|frame| &frame.hit_grid),
                hit_test_changed,
                || {
                    self.projected_hit_test
                        .authoritative_grid(&self.hit_test.grid)
                        .clone()
                },
            );
            let focus_state = shared_domain(
                previous_frame.map(|frame| &frame.focus_state),
                focus_state_domain_changed,
                || self.focus.clone(),
            );
            let focus_path = shared_domain(
                previous_frame.map(|frame| &frame.focus_path),
                focus_path_changed,
                || self.focus_path(),
            );
            let pipeline_report = shared_domain(
                previous_frame.map(|frame| &frame.pipeline_report),
                pipeline_changed,
                || self.last_rebuild_report.pipeline_report(generation),
            );
            let next_frame = Arc::new(UiSurfaceFrame {
                generation,
                domain_generations,
                tree_id: self.tree.tree_id.clone(),
                window_state: self.window_state.clone(),
                arranged_tree,
                render_extract,
                hit_grid,
                focus_state,
                focus_path,
                last_rebuild: self.last_rebuild_report.debug_stats(),
                layout_engine_report,
                pipeline_report,
            });
            #[cfg(feature = "profiling")]
            let publication_elapsed_us = publication_start.elapsed().as_micros() as f64;
            #[cfg(feature = "profiling")]
            crate::core::diagnostics::profiling::record_counter_batch(
                "runtime",
                &[
                    ("ui.surface_frame.publication_build_count", 1.0),
                    (
                        "ui.surface_frame.publication_elapsed_us",
                        publication_elapsed_us,
                    ),
                    ("ui.surface_frame.arranged_node_clone_count", 0.0),
                    (
                        "ui.surface_frame.arranged_segment_share_count",
                        if layout_changed {
                            next_frame.arranged_tree.nodes.segment_count() as f64
                        } else {
                            0.0
                        },
                    ),
                    (
                        "ui.surface_frame.render_command_clone_count",
                        if render_changed {
                            render_patch_stats.cloned_command_count as f64
                        } else {
                            0.0
                        },
                    ),
                    (
                        "ui.surface_frame.render_segment_clone_count",
                        render_patch_stats.cloned_segment_count as f64,
                    ),
                    (
                        "ui.surface_frame.render_directory_node_clone_count",
                        render_patch_stats.cloned_directory_node_count as f64,
                    ),
                    (
                        "ui.surface_frame.render_full_snapshot_build_count",
                        if render_full_snapshot_built { 1.0 } else { 0.0 },
                    ),
                    ("ui.surface_frame.hit_entry_clone_count", 0.0),
                    (
                        "ui.surface_frame.hit_entry_segment_share_count",
                        if hit_test_changed {
                            next_frame.hit_grid.entries.segment_count() as f64
                        } else {
                            0.0
                        },
                    ),
                    ("ui.surface_frame.hit_cell_entry_clone_count", 0.0),
                    (
                        "ui.surface_frame.hit_cell_segment_share_count",
                        if hit_test_changed {
                            next_frame.hit_grid.cells.segment_count() as f64
                        } else {
                            0.0
                        },
                    ),
                    (
                        "ui.surface_frame.focus_state_build_count",
                        if focus_state_domain_changed { 1.0 } else { 0.0 },
                    ),
                    (
                        "ui.surface_frame.focus_path_build_count",
                        if focus_path_changed { 1.0 } else { 0.0 },
                    ),
                    (
                        "ui.surface_frame.focus_path_validation_node_count_upper_bound",
                        if layout_changed && !dirty_domains.focus {
                            previous_frame
                                .filter(|frame| frame.focus_path.focused == self.focus.focused)
                                .map_or(0, |frame| frame.focus_path.bubble_route.len())
                                as f64
                        } else {
                            0.0
                        },
                    ),
                    (
                        "ui.surface_frame.pipeline_stage_build_count",
                        if pipeline_changed {
                            next_frame.pipeline_report.stages.len() as f64
                        } else {
                            0.0
                        },
                    ),
                ],
            );
            publication.frame = Some(next_frame);
            publication.dirty = false;
            publication.dirty_domains = UiSurfaceFrameDirtyDomains::default();
            publication.render_patch_ranges.clear();
            publication.render_requires_full_snapshot = false;
        }

        Arc::clone(
            publication
                .frame
                .as_ref()
                .expect("surface frame publication must exist after refresh"),
        )
    }
}

impl UiSurfaceFramePublication {
    fn record_render_patch_ranges(
        &mut self,
        render_changed: bool,
        patch_ranges: Option<Vec<Range<usize>>>,
    ) {
        if !render_changed {
            return;
        }
        let Some(patch_ranges) = patch_ranges else {
            self.render_patch_ranges.clear();
            self.render_requires_full_snapshot = true;
            return;
        };
        if self.render_requires_full_snapshot {
            return;
        }
        self.render_patch_ranges.extend(patch_ranges);
        merge_render_patch_ranges(&mut self.render_patch_ranges);
    }
}

fn advance_domain_generation(generation: &mut u64, changed: bool) {
    if changed {
        *generation = generation.saturating_add(1);
    }
}

fn shared_domain<T>(previous: Option<&Arc<T>>, changed: bool, build: impl FnOnce() -> T) -> Arc<T> {
    if !changed {
        if let Some(previous) = previous {
            return Arc::clone(previous);
        }
    }
    Arc::new(build())
}

fn published_render_domain(
    previous: Option<&Arc<UiRenderFrameExtract>>,
    changed: bool,
    requires_full_snapshot: bool,
    patch_ranges: &[Range<usize>],
    extract: &zircon_runtime_interface::ui::surface::UiRenderExtract,
) -> (Arc<UiRenderFrameExtract>, UiRenderFramePatchStats, bool) {
    if !changed {
        if let Some(previous) = previous {
            return (
                Arc::clone(previous),
                UiRenderFramePatchStats::default(),
                false,
            );
        }
    }
    if !requires_full_snapshot {
        if let Some((next, stats)) =
            previous.and_then(|previous| previous.patch_ranges_from_extract(extract, patch_ranges))
        {
            return (Arc::new(next), stats, false);
        }
    }
    let next = UiRenderFrameExtract::from_extract(extract);
    let stats = UiRenderFramePatchStats {
        cloned_command_count: next.list.commands.len(),
        cloned_segment_count: next.list.commands.segment_count(),
        cloned_directory_node_count: next.list.commands.directory_node_count(),
    };
    (Arc::new(next), stats, true)
}

fn merge_render_patch_ranges(ranges: &mut Vec<Range<usize>>) {
    ranges.retain(|range| !range.is_empty());
    ranges.sort_unstable_by_key(|range| (range.start, range.end));
    let mut write_index = 0;
    for read_index in 0..ranges.len() {
        let read_start = ranges[read_index].start;
        let read_end = ranges[read_index].end;
        if write_index > 0 && read_start <= ranges[write_index - 1].end {
            ranges[write_index - 1].end = ranges[write_index - 1].end.max(read_end);
        } else {
            ranges.swap(write_index, read_index);
            write_index += 1;
        }
    }
    ranges.truncate(write_index);
}
