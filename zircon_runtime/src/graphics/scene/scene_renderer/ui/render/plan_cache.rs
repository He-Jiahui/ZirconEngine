use std::sync::Arc;

use zircon_runtime_interface::ui::layout::{UiFrame, UiLayoutMetrics};
use zircon_runtime_interface::ui::surface::{UiPaintElement, UiRenderCommand};

use crate::core::framework::render::{
    UiRenderNodeIdProjection, UiRenderSubmission, UiRenderSubmissionSegment,
};
use crate::core::math::UVec2;
use crate::text::font::FontCollectionRevision;

use super::background::{ScreenSpaceUiBackgroundEffect, ScreenSpaceUiBackgroundTracker};
use super::paint_projection::ScreenSpaceUiTextPaintProjectionReport;
use super::{
    PlannedScreenSpaceUi, PreparedScreenSpaceUi, ScreenSpaceUiScissor,
    append_screen_space_ui_command_batches, record_background_tracker_profile,
};

const SCREEN_SPACE_UI_INITIAL_BACKGROUND_GENERATION: u64 = 0;

#[derive(Default)]
pub(in crate::graphics::scene::scene_renderer::ui) struct ScreenSpaceUiPlanCache {
    key: Option<ScreenSpaceUiPlanCacheKey>,
    cached_plan: Option<Arc<PreparedScreenSpaceUi>>,
    segment_entries: Vec<ScreenSpaceUiSegmentPlanCacheEntry>,
    next_background_generation: u64,
}

struct ScreenSpaceUiPlanCacheKey {
    submission: Arc<UiRenderSubmission>,
    viewport_size: UVec2,
    framebuffer_background_bits: Option<[u32; 4]>,
    font_revision: FontCollectionRevision,
}

struct ScreenSpaceUiSegmentPlanCacheEntry {
    commands: Arc<[UiRenderCommand]>,
    route_tree_id: Arc<str>,
    node_id_projection: Option<UiRenderNodeIdProjection>,
    raster_scale_bits: u32,
    incoming_background_generation: u64,
    outgoing_background_generation: u64,
    background_effects: Arc<[ScreenSpaceUiBackgroundEffect]>,
    plan: Arc<PlannedScreenSpaceUi>,
}

struct ScreenSpaceUiSegmentPlanner {
    viewport: UiFrame,
    full_scissor: ScreenSpaceUiScissor,
    metrics: UiLayoutMetrics,
    paint_elements: Vec<UiPaintElement>,
    paint_projection_report: ScreenSpaceUiTextPaintProjectionReport,
    backgrounds: ScreenSpaceUiBackgroundTracker,
}

impl ScreenSpaceUiSegmentPlanner {
    fn new(viewport_size: UVec2, framebuffer_background_color: Option<[f32; 4]>) -> Self {
        let viewport = UiFrame::new(
            0.0,
            0.0,
            viewport_size.x.max(1) as f32,
            viewport_size.y.max(1) as f32,
        );
        Self {
            viewport,
            full_scissor: ScreenSpaceUiScissor {
                x: 0,
                y: 0,
                width: viewport_size.x.max(1),
                height: viewport_size.y.max(1),
            },
            metrics: UiLayoutMetrics::default(),
            paint_elements: Vec::new(),
            paint_projection_report: ScreenSpaceUiTextPaintProjectionReport::default(),
            backgrounds: ScreenSpaceUiBackgroundTracker::with_framebuffer_background(
                viewport,
                framebuffer_background_color,
            ),
        }
    }

    fn plan_segment(
        &mut self,
        segment: &UiRenderSubmissionSegment,
        commands: &[UiRenderCommand],
    ) -> (
        Arc<PlannedScreenSpaceUi>,
        Arc<[ScreenSpaceUiBackgroundEffect]>,
    ) {
        let effect_start = self.backgrounds.effect_count();
        let mut plan = PlannedScreenSpaceUi::default();
        append_screen_space_ui_command_batches(
            commands,
            segment.extract().normalized_raster_scale(),
            segment.route_tree_id(),
            |node_id| segment.project_node_id(node_id),
            self.viewport,
            self.full_scissor,
            self.metrics,
            &mut self.paint_elements,
            &mut self.paint_projection_report,
            &mut self.backgrounds,
            &mut plan,
        );
        let background_effects = Arc::from(self.backgrounds.effects_since(effect_start));
        (Arc::new(plan), background_effects)
    }

    fn replay_background_effects(&mut self, effects: &[ScreenSpaceUiBackgroundEffect]) {
        self.backgrounds.replay_effects(effects);
    }

    fn background_stats(&self) -> super::background::ScreenSpaceUiBackgroundTrackerStats {
        self.backgrounds.stats()
    }

    fn publish_paint_projection_profile(&self) {
        self.paint_projection_report.publish_profile_counters();
    }
}

fn compose_screen_space_ui_segment_plans(
    segment_plans: &[Arc<PlannedScreenSpaceUi>],
) -> (Option<PreparedScreenSpaceUi>, usize) {
    let mut combined = PlannedScreenSpaceUi::default();
    let mut has_render_activity = false;
    for segment in segment_plans {
        has_render_activity |= segment.has_render_activity();
        combined.append_non_render_payload_cloned(segment);
    }
    if !has_render_activity {
        return (None, 0);
    }
    (
        Some(PreparedScreenSpaceUi {
            render_segments: Arc::from(segment_plans.to_vec()),
            resolved_glyph_artifact_routes: combined.resolved_glyph_artifact_routes,
        }),
        0,
    )
}

impl ScreenSpaceUiPlanCache {
    #[cfg(test)]
    pub(super) fn prepare(
        &mut self,
        submission: &Arc<UiRenderSubmission>,
        viewport_size: UVec2,
        framebuffer_background_color: Option<[f32; 4]>,
        font_generation: u64,
    ) -> Option<Arc<PreparedScreenSpaceUi>> {
        self.prepare_with_font_revision(
            submission,
            viewport_size,
            framebuffer_background_color,
            FontCollectionRevision::new(
                crate::text::font::shared_font_collection_handle(),
                font_generation,
            ),
        )
    }

    pub(super) fn prepare_with_font_revision(
        &mut self,
        submission: &Arc<UiRenderSubmission>,
        viewport_size: UVec2,
        framebuffer_background_color: Option<[f32; 4]>,
        font_revision: FontCollectionRevision,
    ) -> Option<Arc<PreparedScreenSpaceUi>> {
        let framebuffer_background_bits =
            framebuffer_background_color.map(|color| color.map(f32::to_bits));
        if self.key.as_ref().is_some_and(|key| {
            key.matches_exact(
                submission,
                viewport_size,
                framebuffer_background_bits,
                font_revision,
            )
        }) {
            ScreenSpaceUiTextPaintProjectionReport::default().publish_profile_counters();
            record_screen_space_ui_plan_full_reuse_profile(self.segment_entries.len());
            return self.cached_plan.as_ref().map(Arc::clone);
        }

        let planner_inputs_match = self.key.as_ref().is_some_and(|key| {
            key.matches_planner_inputs(viewport_size, framebuffer_background_bits, font_revision)
        });
        let command_segment_count = submission
            .segments()
            .iter()
            .map(|segment| segment.extract().list.commands.segment_count())
            .sum::<usize>();
        let previous_entry_count = self.segment_entries.len();
        let previous_entries = if planner_inputs_match {
            std::mem::take(&mut self.segment_entries)
        } else {
            self.segment_entries.clear();
            Vec::new()
        };
        let mut previous_entries = previous_entries.into_iter();
        let mut planner =
            ScreenSpaceUiSegmentPlanner::new(viewport_size, framebuffer_background_color);
        let mut incoming_background_generation = SCREEN_SPACE_UI_INITIAL_BACKGROUND_GENERATION;
        let mut next_entries = Vec::with_capacity(command_segment_count);
        let mut segment_plans = Vec::with_capacity(command_segment_count);
        let mut segment_cache_hit_count = 0_usize;
        let mut segment_command_visit_count = 0_usize;
        let mut all_segments_reused =
            planner_inputs_match && previous_entry_count == command_segment_count;

        for segment in submission.segments() {
            for command_segment in segment.extract().command_segments() {
                let segment_incoming_background_generation = incoming_background_generation;
                let mut previous = previous_entries.next();
                if previous.as_ref().is_some_and(|entry| {
                    entry.matches(
                        segment,
                        command_segment,
                        segment_incoming_background_generation,
                    )
                }) {
                    let entry = previous
                        .take()
                        .expect("matching segment cache entry must remain available");
                    segment_cache_hit_count = segment_cache_hit_count.saturating_add(1);
                    planner.replay_background_effects(&entry.background_effects);
                    incoming_background_generation = entry.outgoing_background_generation;
                    segment_plans.push(Arc::clone(&entry.plan));
                    next_entries.push(entry);
                    continue;
                }

                all_segments_reused = false;
                segment_command_visit_count =
                    segment_command_visit_count.saturating_add(command_segment.len());
                let (plan, background_effects) =
                    planner.plan_segment(segment, command_segment.as_ref());
                let outgoing_background_generation = previous
                    .as_ref()
                    .filter(|entry| {
                        entry.incoming_background_generation
                            == segment_incoming_background_generation
                            && entry.background_effects.as_ref() == background_effects.as_ref()
                    })
                    .map_or_else(
                        || self.allocate_background_generation(),
                        |entry| entry.outgoing_background_generation,
                    );
                incoming_background_generation = outgoing_background_generation;
                segment_plans.push(Arc::clone(&plan));
                next_entries.push(ScreenSpaceUiSegmentPlanCacheEntry {
                    commands: Arc::clone(command_segment),
                    route_tree_id: Arc::clone(segment.route_tree_id()),
                    node_id_projection: segment.node_id_projection(),
                    raster_scale_bits: segment.extract().normalized_raster_scale().to_bits(),
                    incoming_background_generation: segment_incoming_background_generation,
                    outgoing_background_generation,
                    background_effects,
                    plan,
                });
            }
        }
        all_segments_reused &= previous_entries.next().is_none();
        planner.publish_paint_projection_profile();
        record_background_tracker_profile(planner.background_stats());

        self.key = Some(ScreenSpaceUiPlanCacheKey {
            submission: Arc::clone(submission),
            viewport_size,
            framebuffer_background_bits,
            font_revision,
        });
        self.segment_entries = next_entries;

        if all_segments_reused {
            record_screen_space_ui_plan_full_reuse_profile(segment_cache_hit_count);
            return self.cached_plan.as_ref().map(Arc::clone);
        }

        let (cached_plan, composition_payload_clone_count) =
            compose_screen_space_ui_segment_plans(&segment_plans);
        crate::core::diagnostics::profiling::record_counter_batch(
            "runtime",
            &[
                ("ui.screen_space_ui_plan.build_count", 1.0),
                (
                    "ui.screen_space_ui_plan.command_visit_count",
                    segment_command_visit_count as f64,
                ),
                (
                    "ui.screen_space_ui_plan.segment_cache_hit_count",
                    segment_cache_hit_count as f64,
                ),
                (
                    "ui.screen_space_ui_plan.command_leaf_cache_hit_count",
                    segment_cache_hit_count as f64,
                ),
                (
                    "ui.screen_space_ui_plan.command_leaf_count",
                    command_segment_count as f64,
                ),
                (
                    "ui.screen_space_ui_plan.command_leaf_rebuild_count",
                    command_segment_count.saturating_sub(segment_cache_hit_count) as f64,
                ),
                (
                    "ui.screen_space_ui_plan.segment_command_visit_count",
                    segment_command_visit_count as f64,
                ),
                (
                    "ui.screen_space_ui_plan.composition_payload_clone_count",
                    composition_payload_clone_count as f64,
                ),
            ],
        );
        self.cached_plan = cached_plan.map(Arc::new);
        self.cached_plan.as_ref().map(Arc::clone)
    }

    pub(super) fn clear(&mut self) {
        self.key = None;
        self.cached_plan = None;
        self.segment_entries.clear();
    }

    fn allocate_background_generation(&mut self) -> u64 {
        self.next_background_generation = self.next_background_generation.wrapping_add(1).max(1);
        self.next_background_generation
    }

    #[cfg(test)]
    pub(super) fn cached_segment_plan(&self, index: usize) -> Option<&Arc<PlannedScreenSpaceUi>> {
        self.segment_entries.get(index).map(|entry| &entry.plan)
    }
}

fn record_screen_space_ui_plan_full_reuse_profile(command_leaf_count: usize) {
    crate::core::diagnostics::profiling::record_counter_batch(
        "runtime",
        &[
            ("ui.screen_space_ui_plan.cache_hit_count", 1.0),
            ("ui.screen_space_ui_plan.command_visit_count", 0.0),
            (
                "ui.screen_space_ui_plan.segment_cache_hit_count",
                command_leaf_count as f64,
            ),
            (
                "ui.screen_space_ui_plan.command_leaf_cache_hit_count",
                command_leaf_count as f64,
            ),
            (
                "ui.screen_space_ui_plan.command_leaf_count",
                command_leaf_count as f64,
            ),
            ("ui.screen_space_ui_plan.command_leaf_rebuild_count", 0.0),
            ("ui.screen_space_ui_plan.segment_command_visit_count", 0.0),
            (
                "ui.screen_space_ui_plan.composition_payload_clone_count",
                0.0,
            ),
        ],
    );
}

impl ScreenSpaceUiPlanCacheKey {
    fn matches_exact(
        &self,
        submission: &Arc<UiRenderSubmission>,
        viewport_size: UVec2,
        framebuffer_background_bits: Option<[u32; 4]>,
        font_revision: FontCollectionRevision,
    ) -> bool {
        Arc::ptr_eq(&self.submission, submission)
            && self.matches_planner_inputs(
                viewport_size,
                framebuffer_background_bits,
                font_revision,
            )
    }

    fn matches_planner_inputs(
        &self,
        viewport_size: UVec2,
        framebuffer_background_bits: Option<[u32; 4]>,
        font_revision: FontCollectionRevision,
    ) -> bool {
        self.viewport_size == viewport_size
            && self.framebuffer_background_bits == framebuffer_background_bits
            && self.font_revision == font_revision
    }
}

impl ScreenSpaceUiSegmentPlanCacheEntry {
    fn matches(
        &self,
        segment: &UiRenderSubmissionSegment,
        command_segment: &Arc<[UiRenderCommand]>,
        incoming_background_generation: u64,
    ) -> bool {
        Arc::ptr_eq(&self.commands, command_segment)
            && self.route_tree_id.as_ref() == segment.route_tree_id().as_ref()
            && self.node_id_projection == segment.node_id_projection()
            && self.raster_scale_bits == segment.extract().normalized_raster_scale().to_bits()
            && self.incoming_background_generation == incoming_background_generation
    }
}
