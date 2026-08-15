use serde::{Deserialize, Serialize};

use zircon_runtime_interface::ui::{
    pipeline::{
        UiPipelineDirtyReason, UiPipelineFrameReport, UiPipelineStage, UiPipelineStageCounters,
        UiPipelineStageReport,
    },
    surface::UiSurfaceRebuildDebugStats,
    tree::UiDirtyFlags,
};
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
    /// Nodes entered by the arranged-tree builder's outer traversal.
    pub arranged_outer_node_visit_count: usize,
    #[serde(default)]
    /// Nodes entered by the hit-grid builder's outer traversal.
    pub hit_grid_outer_node_visit_count: usize,
    #[serde(default)]
    /// Nodes entered by the render extractor's outer traversal.
    pub render_outer_node_visit_count: usize,
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
    pub text_measure_cache_hit_count: u64,
    #[serde(default)]
    pub text_measure_cache_miss_count: u64,
    #[serde(default)]
    pub text_layout_cache_hit_count: u64,
    #[serde(default)]
    pub text_layout_cache_miss_count: u64,
    #[serde(default)]
    pub text_shape_cache_hit_count: u64,
    #[serde(default)]
    pub text_shape_cache_miss_count: u64,
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
            arranged_outer_node_visit_count: self.arranged_outer_node_visit_count,
            hit_grid_outer_node_visit_count: self.hit_grid_outer_node_visit_count,
            render_outer_node_visit_count: self.render_outer_node_visit_count,
            layout_visited_node_count: self.layout_visited_node_count,
            layout_geometry_changed_node_count: self.layout_geometry_changed_node_count,
            layout_skipped_node_count: self.layout_skipped_node_count,
            render_command_reused_count: self.render_command_reused_count,
            render_command_rebuilt_count: self.render_command_rebuilt_count,
            render_damage_rect_count: self.render_damage_rect_count,
            text_measure_cache_hit_count: self.text_measure_cache_hit_count,
            text_measure_cache_miss_count: self.text_measure_cache_miss_count,
            text_layout_cache_hit_count: self.text_layout_cache_hit_count,
            text_layout_cache_miss_count: self.text_layout_cache_miss_count,
            text_shape_cache_hit_count: self.text_shape_cache_hit_count,
            text_shape_cache_miss_count: self.text_shape_cache_miss_count,
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

    pub fn pipeline_report(self, frame_index: u64) -> UiPipelineFrameReport {
        UiPipelineFrameReport::from_stage_reports(
            frame_index,
            vec![
                skipped_stage(
                    UiPipelineStage::InputCollect,
                    dirty_reasons_for_input(self.dirty_flags),
                    "input collection is recorded by dispatch results, not rebuild timing",
                ),
                skipped_stage(
                    UiPipelineStage::Focus,
                    dirty_reasons_for_focus(self.dirty_flags),
                    "focus routing is recorded by UiFocusState, not rebuild timing",
                ),
                skipped_stage(
                    UiPipelineStage::WidgetBehavior,
                    dirty_reasons_for_widget_behavior(self.dirty_flags),
                    "widget behavior is recorded by dispatch replies, not rebuild timing",
                ),
                text_measure_stage_report(self),
                measured_or_skipped_stage(
                    UiPipelineStage::Layout,
                    self.layout_recomputed,
                    self.layout_elapsed_micros,
                    dirty_reasons_for_layout(self.dirty_flags),
                    UiPipelineStageCounters {
                        layout_node_count: self.layout_visited_node_count as u64,
                        full_layout_count: u64::from(
                            self.layout_recomputed && self.layout_skipped_node_count == 0,
                        ),
                        incremental_layout_count: u64::from(
                            self.layout_recomputed && self.layout_skipped_node_count > 0,
                        ),
                        ..UiPipelineStageCounters::default()
                    },
                    "layout did not run for this surface rebuild",
                ),
                measured_or_skipped_stage(
                    UiPipelineStage::PostLayout,
                    self.arranged_rebuilt,
                    self.arranged_elapsed_micros,
                    dirty_reasons_for_post_layout(self.dirty_flags),
                    UiPipelineStageCounters {
                        stack_node_count: self.arranged_node_count as u64,
                        post_layout_outer_node_visit_count: self.arranged_outer_node_visit_count
                            as u64,
                        ..UiPipelineStageCounters::default()
                    },
                    "post-layout arranged tree did not rebuild",
                ),
                measured_or_skipped_stage(
                    UiPipelineStage::Picking,
                    self.hit_grid_rebuilt,
                    self.hit_grid_elapsed_micros,
                    dirty_reasons_for_picking(self.dirty_flags),
                    UiPipelineStageCounters {
                        picking_candidate_count: self.hit_grid_entry_count as u64,
                        picking_outer_node_visit_count: self.hit_grid_outer_node_visit_count as u64,
                        hit_grid_rebuild_count: u64::from(self.hit_grid_rebuilt),
                        ..UiPipelineStageCounters::default()
                    },
                    "picking grid did not rebuild",
                ),
                skipped_stage(
                    UiPipelineStage::A11yExtract,
                    dirty_reasons_for_a11y(self.dirty_flags),
                    "accessibility extraction is exposed through UiAccessibilityTreeSnapshot",
                ),
                measured_or_skipped_stage(
                    UiPipelineStage::RenderExtract,
                    self.render_rebuilt,
                    self.render_elapsed_micros,
                    dirty_reasons_for_render(self.dirty_flags),
                    UiPipelineStageCounters {
                        render_extract_command_count: self.render_command_count as u64,
                        render_extract_outer_node_visit_count: self.render_outer_node_visit_count
                            as u64,
                        render_command_reuse_count: self.render_command_reused_count as u64,
                        render_command_rebuild_count: self.render_command_rebuilt_count as u64,
                        ..UiPipelineStageCounters::default()
                    },
                    "render extract did not rebuild",
                ),
                skipped_stage(
                    UiPipelineStage::BatchPrepare,
                    dirty_reasons_for_batch_prepare(self.dirty_flags),
                    "batch preparation is owned by renderer consumers",
                ),
            ],
        )
    }

    pub(super) fn with_counts(mut self, counts: UiSurfaceRebuildReport) -> Self {
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

    pub(super) fn with_text_cache_stats(mut self, stats: UiTextCacheFrameStats) -> Self {
        self.text_measure_cache_hit_count = stats.measure_hit_count;
        self.text_measure_cache_miss_count = stats.measure_miss_count;
        self.text_layout_cache_hit_count = stats.layout_hit_count;
        self.text_layout_cache_miss_count = stats.layout_miss_count;
        self.text_shape_cache_hit_count = stats.shape_hit_count;
        self.text_shape_cache_miss_count = stats.shape_miss_count;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct UiTextCacheFrameStats {
    pub(super) measure_hit_count: u64,
    pub(super) measure_miss_count: u64,
    pub(super) layout_hit_count: u64,
    pub(super) layout_miss_count: u64,
    pub(super) shape_hit_count: u64,
    pub(super) shape_miss_count: u64,
}

fn text_measure_stage_report(rebuild: UiSurfaceRebuildReport) -> UiPipelineStageReport {
    let counters = UiPipelineStageCounters {
        text_measure_count: rebuild
            .text_measure_cache_hit_count
            .saturating_add(rebuild.text_measure_cache_miss_count),
        content_measure_count: rebuild
            .text_layout_cache_hit_count
            .saturating_add(rebuild.text_layout_cache_miss_count),
        text_measure_cache_hit_count: rebuild.text_measure_cache_hit_count,
        text_measure_cache_miss_count: rebuild.text_measure_cache_miss_count,
        text_layout_cache_hit_count: rebuild.text_layout_cache_hit_count,
        text_layout_cache_miss_count: rebuild.text_layout_cache_miss_count,
        text_shape_cache_hit_count: rebuild.text_shape_cache_hit_count,
        text_shape_cache_miss_count: rebuild.text_shape_cache_miss_count,
        ..UiPipelineStageCounters::default()
    };
    if counters.text_measure_count == 0
        && counters.content_measure_count == 0
        && counters.text_shape_cache_hit_count == 0
        && counters.text_shape_cache_miss_count == 0
    {
        return skipped_stage(
            UiPipelineStage::TextMeasure,
            dirty_reasons_for_text_measure(rebuild.dirty_flags),
            "text measurement did not run for this surface rebuild",
        );
    }

    let mut report = UiPipelineStageReport::new(
        UiPipelineStage::TextMeasure,
        0,
        dirty_reasons_for_text_measure(rebuild.dirty_flags),
        counters,
    );
    report
        .notes
        .push("text timing is folded into layout and render extract stages".to_string());
    report
}

fn measured_or_skipped_stage(
    stage: UiPipelineStage,
    measured: bool,
    elapsed_micros: u64,
    dirty_reasons: Vec<UiPipelineDirtyReason>,
    counters: UiPipelineStageCounters,
    skipped_note: &str,
) -> UiPipelineStageReport {
    if measured {
        UiPipelineStageReport::new(stage, elapsed_micros, dirty_reasons, counters)
    } else {
        skipped_stage(stage, dirty_reasons, skipped_note)
    }
}

fn skipped_stage(
    stage: UiPipelineStage,
    dirty_reasons: Vec<UiPipelineDirtyReason>,
    note: &str,
) -> UiPipelineStageReport {
    let mut report = UiPipelineStageReport::skipped(stage, dirty_reasons);
    report.notes.push(note.to_string());
    report
}

fn dirty_reasons_for_input(dirty: UiDirtyFlags) -> Vec<UiPipelineDirtyReason> {
    dirty_reasons(&[(dirty.input, UiPipelineDirtyReason::Input)])
}

fn dirty_reasons_for_focus(dirty: UiDirtyFlags) -> Vec<UiPipelineDirtyReason> {
    dirty_reasons(&[(dirty.input, UiPipelineDirtyReason::Focus)])
}

fn dirty_reasons_for_widget_behavior(dirty: UiDirtyFlags) -> Vec<UiPipelineDirtyReason> {
    dirty_reasons(&[
        (dirty.input, UiPipelineDirtyReason::WidgetBehavior),
        (dirty.style, UiPipelineDirtyReason::Style),
    ])
}

fn dirty_reasons_for_text_measure(dirty: UiDirtyFlags) -> Vec<UiPipelineDirtyReason> {
    dirty_reasons(&[(dirty.text, UiPipelineDirtyReason::Text)])
}

fn dirty_reasons_for_layout(dirty: UiDirtyFlags) -> Vec<UiPipelineDirtyReason> {
    dirty_reasons(&[
        (dirty.layout, UiPipelineDirtyReason::Layout),
        (dirty.style, UiPipelineDirtyReason::Style),
        (dirty.text, UiPipelineDirtyReason::Text),
        (dirty.visible_range, UiPipelineDirtyReason::LayoutMetrics),
    ])
}

fn dirty_reasons_for_post_layout(dirty: UiDirtyFlags) -> Vec<UiPipelineDirtyReason> {
    dirty_reasons_for_layout(dirty)
}

fn dirty_reasons_for_picking(dirty: UiDirtyFlags) -> Vec<UiPipelineDirtyReason> {
    dirty_reasons(&[
        (dirty.hit_test, UiPipelineDirtyReason::Picking),
        (dirty.input, UiPipelineDirtyReason::Input),
        (dirty.layout, UiPipelineDirtyReason::LayoutMetrics),
        (dirty.visible_range, UiPipelineDirtyReason::LayoutMetrics),
    ])
}

fn dirty_reasons_for_a11y(dirty: UiDirtyFlags) -> Vec<UiPipelineDirtyReason> {
    dirty_reasons(&[
        (dirty.input, UiPipelineDirtyReason::A11y),
        (dirty.style, UiPipelineDirtyReason::A11y),
        (dirty.text, UiPipelineDirtyReason::A11y),
        (dirty.layout, UiPipelineDirtyReason::A11y),
        (dirty.visible_range, UiPipelineDirtyReason::A11y),
    ])
}

fn dirty_reasons_for_render(dirty: UiDirtyFlags) -> Vec<UiPipelineDirtyReason> {
    dirty_reasons(&[
        (dirty.render, UiPipelineDirtyReason::Render),
        (dirty.style, UiPipelineDirtyReason::Style),
        (dirty.text, UiPipelineDirtyReason::Text),
        (dirty.layout, UiPipelineDirtyReason::LayoutMetrics),
        (dirty.visible_range, UiPipelineDirtyReason::LayoutMetrics),
    ])
}

fn dirty_reasons_for_batch_prepare(dirty: UiDirtyFlags) -> Vec<UiPipelineDirtyReason> {
    dirty_reasons_for_render(dirty)
}

fn dirty_reasons(candidates: &[(bool, UiPipelineDirtyReason)]) -> Vec<UiPipelineDirtyReason> {
    let mut reasons = Vec::new();
    for (enabled, reason) in candidates {
        if *enabled && !reasons.contains(reason) {
            reasons.push(*reason);
        }
    }
    reasons
}
