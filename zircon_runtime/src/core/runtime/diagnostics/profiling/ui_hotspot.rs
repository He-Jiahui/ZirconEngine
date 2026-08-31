use std::collections::BTreeMap;

use zircon_runtime_interface::{
    ProfileCounterSnapshot, ProfileSnapshot, UiHotspotAlert, UiHotspotReport, UiScenarioHotspot,
};

const UI_COUNTER_PREFIX: &str = "ui.";

pub fn analyze_ui_hotspots(snapshot: &ProfileSnapshot) -> UiHotspotReport {
    let mut scenarios: BTreeMap<String, UiScenarioAccumulator> = BTreeMap::new();
    let mut counter_count = 0;

    for counter in &snapshot.counters {
        let Some((scenario, metric)) = parse_ui_counter(counter) else {
            continue;
        };
        counter_count += 1;
        scenarios
            .entry(scenario.to_string())
            .or_insert_with(|| UiScenarioAccumulator::new(scenario))
            .record(metric, counter);
    }

    let scenarios = scenarios
        .into_values()
        .map(UiScenarioAccumulator::finish)
        .collect::<Vec<_>>();
    let alerts = alerts_for_scenarios(&scenarios);

    UiHotspotReport {
        session_id: snapshot.session_id.clone(),
        frame_budget_ms: snapshot.frame_budget_ms,
        generated_from_counter_count: counter_count,
        scenarios,
        alerts,
    }
}

fn parse_ui_counter(counter: &ProfileCounterSnapshot) -> Option<(&str, &str)> {
    let rest = counter.name.strip_prefix(UI_COUNTER_PREFIX)?;
    let (scenario, metric) = rest.split_once('.')?;
    (!scenario.is_empty() && !metric.is_empty()).then_some((scenario, metric))
}

#[derive(Debug)]
struct UiScenarioAccumulator {
    hotspot: UiScenarioHotspot,
    frame_durations_us: Vec<u64>,
    gpu_times_us: Vec<u64>,
}

impl UiScenarioAccumulator {
    fn new(scenario: &str) -> Self {
        Self {
            hotspot: UiScenarioHotspot::empty(scenario),
            frame_durations_us: Vec::new(),
            gpu_times_us: Vec::new(),
        }
    }

    fn record(&mut self, metric: &str, counter: &ProfileCounterSnapshot) {
        let value = counter_value(counter);
        match metric {
            "frame_duration_us" => self.frame_durations_us.push(value),
            "host_invalidation_transaction_count" => {
                self.hotspot.host_invalidation_transaction_count = self
                    .hotspot
                    .host_invalidation_transaction_count
                    .saturating_add(value)
            }
            "host_invalidation_scope_count" => {
                self.hotspot.host_invalidation_scope_count = self
                    .hotspot
                    .host_invalidation_scope_count
                    .saturating_add(value)
            }
            "host_invalidation_legacy_dirty_transaction_count" => {
                self.hotspot
                    .host_invalidation_legacy_dirty_transaction_count = self
                    .hotspot
                    .host_invalidation_legacy_dirty_transaction_count
                    .saturating_add(value)
            }
            "host_invalidation_full_target_count" => {
                self.hotspot.host_invalidation_full_target_count = self
                    .hotspot
                    .host_invalidation_full_target_count
                    .saturating_add(value)
            }
            "host_invalidation_shell_content_target_count" => {
                self.hotspot.host_invalidation_shell_content_target_count = self
                    .hotspot
                    .host_invalidation_shell_content_target_count
                    .saturating_add(value)
            }
            "host_invalidation_workbench_projection_target_count" => {
                self.hotspot
                    .host_invalidation_workbench_projection_target_count = self
                    .hotspot
                    .host_invalidation_workbench_projection_target_count
                    .saturating_add(value)
            }
            "host_invalidation_view_presentation_target_count" => {
                self.hotspot
                    .host_invalidation_view_presentation_target_count = self
                    .hotspot
                    .host_invalidation_view_presentation_target_count
                    .saturating_add(value)
            }
            "host_invalidation_window_metrics_target_count" => {
                self.hotspot.host_invalidation_window_metrics_target_count = self
                    .hotspot
                    .host_invalidation_window_metrics_target_count
                    .saturating_add(value)
            }
            "host_invalidation_paint_only_target_count" => {
                self.hotspot.host_invalidation_paint_only_target_count = self
                    .hotspot
                    .host_invalidation_paint_only_target_count
                    .saturating_add(value)
            }
            "slow_path_rebuild_count" => {
                self.hotspot.slow_path_rebuild_count =
                    self.hotspot.slow_path_rebuild_count.saturating_add(value)
            }
            "render_path_count" => {
                self.hotspot.render_path_count =
                    self.hotspot.render_path_count.saturating_add(value)
            }
            "presentation_rebuild_count" => {
                self.hotspot.presentation_rebuild_count = self
                    .hotspot
                    .presentation_rebuild_count
                    .saturating_add(value)
            }
            "asset_editor_pane_presentation_build_count" => {
                self.hotspot.asset_editor_pane_presentation_build_count = self
                    .hotspot
                    .asset_editor_pane_presentation_build_count
                    .saturating_add(value)
            }
            "asset_editor_pane_reflection_build_count" => {
                self.hotspot.asset_editor_pane_reflection_build_count = self
                    .hotspot
                    .asset_editor_pane_reflection_build_count
                    .saturating_add(value)
            }
            "asset_editor_pane_preview_build_count" => {
                self.hotspot.asset_editor_pane_preview_build_count = self
                    .hotspot
                    .asset_editor_pane_preview_build_count
                    .saturating_add(value)
            }
            "asset_editor_pane_source_build_count" => {
                self.hotspot.asset_editor_pane_source_build_count = self
                    .hotspot
                    .asset_editor_pane_source_build_count
                    .saturating_add(value)
            }
            "asset_editor_pane_inspector_build_count" => {
                self.hotspot.asset_editor_pane_inspector_build_count = self
                    .hotspot
                    .asset_editor_pane_inspector_build_count
                    .saturating_add(value)
            }
            "asset_editor_pane_style_build_count" => {
                self.hotspot.asset_editor_pane_style_build_count = self
                    .hotspot
                    .asset_editor_pane_style_build_count
                    .saturating_add(value)
            }
            "asset_editor_pane_theme_build_count" => {
                self.hotspot.asset_editor_pane_theme_build_count = self
                    .hotspot
                    .asset_editor_pane_theme_build_count
                    .saturating_add(value)
            }
            "asset_editor_pane_command_availability_build_count" => {
                self.hotspot
                    .asset_editor_pane_command_availability_build_count = self
                    .hotspot
                    .asset_editor_pane_command_availability_build_count
                    .saturating_add(value)
            }
            "full_paint_count" => {
                self.hotspot.full_paint_count = self.hotspot.full_paint_count.saturating_add(value)
            }
            "region_paint_count" => {
                self.hotspot.region_paint_count =
                    self.hotspot.region_paint_count.saturating_add(value)
            }
            "painted_pixels" => {
                self.hotspot.painted_pixels = self.hotspot.painted_pixels.saturating_add(value)
            }
            "presented_surface_pixels" => {
                self.hotspot.presented_surface_pixels =
                    self.hotspot.presented_surface_pixels.saturating_add(value)
            }
            "redraw_full_frame" => {
                self.hotspot.redraw_full_frame_count =
                    self.hotspot.redraw_full_frame_count.saturating_add(value)
            }
            "redraw_region" => {
                self.hotspot.redraw_region_count =
                    self.hotspot.redraw_region_count.saturating_add(value)
            }
            "dirty_layout" => {
                self.hotspot.dirty_layout_count =
                    self.hotspot.dirty_layout_count.saturating_add(value)
            }
            "dirty_presentation" => {
                self.hotspot.dirty_presentation_count =
                    self.hotspot.dirty_presentation_count.saturating_add(value)
            }
            "dirty_render" => {
                self.hotspot.dirty_render_count =
                    self.hotspot.dirty_render_count.saturating_add(value)
            }
            "dirty_paint_only" => {
                self.hotspot.dirty_paint_only_count =
                    self.hotspot.dirty_paint_only_count.saturating_add(value)
            }
            "chrome_snapshot_count" => {
                self.hotspot.chrome_snapshot_count =
                    self.hotspot.chrome_snapshot_count.saturating_add(value)
            }
            "workbench_model_build_count" => {
                self.hotspot.workbench_model_build_count = self
                    .hotspot
                    .workbench_model_build_count
                    .saturating_add(value)
            }
            "workbench_hit_index_build_count" => {
                self.hotspot.workbench_hit_index_build_count = self
                    .hotspot
                    .workbench_hit_index_build_count
                    .saturating_add(value)
            }
            "workbench_hit_index_query_count" => {
                self.hotspot.workbench_hit_index_query_count = self
                    .hotspot
                    .workbench_hit_index_query_count
                    .saturating_add(value)
            }
            "pane_popup_index_query_count" => {
                self.hotspot.pane_popup_index_query_count = self
                    .hotspot
                    .pane_popup_index_query_count
                    .saturating_add(value)
            }
            "pane_popup_index_candidate_count" => {
                self.hotspot.pane_popup_index_candidate_count = self
                    .hotspot
                    .pane_popup_index_candidate_count
                    .saturating_add(value)
            }
            "visual_asset_targeted_invalidation_count" => {
                self.hotspot.visual_asset_targeted_invalidation_count = self
                    .hotspot
                    .visual_asset_targeted_invalidation_count
                    .saturating_add(value)
            }
            "svg_tree_targeted_invalidation_count" => {
                self.hotspot.svg_tree_targeted_invalidation_count = self
                    .hotspot
                    .svg_tree_targeted_invalidation_count
                    .saturating_add(value)
            }
            "visual_asset_reconcile_source_visit_count" => {
                self.hotspot.visual_asset_reconcile_source_visit_count = self
                    .hotspot
                    .visual_asset_reconcile_source_visit_count
                    .saturating_add(value)
            }
            "visual_asset_reconciled_invalidation_count" => {
                self.hotspot.visual_asset_reconciled_invalidation_count = self
                    .hotspot
                    .visual_asset_reconciled_invalidation_count
                    .saturating_add(value)
            }
            "svg_tree_reconcile_source_visit_count" => {
                self.hotspot.svg_tree_reconcile_source_visit_count = self
                    .hotspot
                    .svg_tree_reconcile_source_visit_count
                    .saturating_add(value)
            }
            "svg_tree_reconciled_invalidation_count" => {
                self.hotspot.svg_tree_reconciled_invalidation_count = self
                    .hotspot
                    .svg_tree_reconciled_invalidation_count
                    .saturating_add(value)
            }
            "visual_asset_full_invalidation_count" => {
                self.hotspot.visual_asset_full_invalidation_count = self
                    .hotspot
                    .visual_asset_full_invalidation_count
                    .saturating_add(value)
            }
            "visual_asset_cache_hit_count" => {
                self.hotspot.visual_asset_cache_hit_count = self
                    .hotspot
                    .visual_asset_cache_hit_count
                    .saturating_add(value)
            }
            "visual_asset_cache_miss_count" => {
                self.hotspot.visual_asset_cache_miss_count = self
                    .hotspot
                    .visual_asset_cache_miss_count
                    .saturating_add(value)
            }
            "visual_asset_cache_candidate_build_count" => {
                self.hotspot.visual_asset_cache_candidate_build_count = self
                    .hotspot
                    .visual_asset_cache_candidate_build_count
                    .saturating_add(value)
            }
            "svg_tree_cache_memory_hit_count" => {
                self.hotspot.svg_tree_cache_memory_hit_count = self
                    .hotspot
                    .svg_tree_cache_memory_hit_count
                    .saturating_add(value)
            }
            "svg_tree_cache_miss_count" => {
                self.hotspot.svg_tree_cache_miss_count =
                    self.hotspot.svg_tree_cache_miss_count.saturating_add(value)
            }
            "chrome_command_full_rebuild_count" => {
                self.hotspot.chrome_command_full_rebuild_count = self
                    .hotspot
                    .chrome_command_full_rebuild_count
                    .saturating_add(value)
            }
            "chrome_command_patch_count" => {
                self.hotspot.chrome_command_patch_count = self
                    .hotspot
                    .chrome_command_patch_count
                    .saturating_add(value)
            }
            "software_fallback_present_count" => {
                self.hotspot.software_fallback_present_count = self
                    .hotspot
                    .software_fallback_present_count
                    .saturating_add(value)
            }
            "gpu_upload_bytes" => {
                self.hotspot.gpu_upload_bytes = self.hotspot.gpu_upload_bytes.saturating_add(value)
            }
            "gpu_image_upload_writes" => {
                self.hotspot.gpu_image_upload_write_count = self
                    .hotspot
                    .gpu_image_upload_write_count
                    .saturating_add(value)
            }
            "gpu_image_shared_resolves" => {
                self.hotspot.gpu_image_shared_resolve_count = self
                    .hotspot
                    .gpu_image_shared_resolve_count
                    .saturating_add(value)
            }
            "gpu_image_shared_upload_writes" => {
                self.hotspot.gpu_image_shared_upload_write_count = self
                    .hotspot
                    .gpu_image_shared_upload_write_count
                    .saturating_add(value)
            }
            "gpu_image_shared_upload_bytes" => {
                self.hotspot.gpu_image_shared_upload_bytes = self
                    .hotspot
                    .gpu_image_shared_upload_bytes
                    .saturating_add(value)
            }
            "gpu_image_shared_resident_bytes" => {
                self.hotspot.gpu_image_shared_resident_bytes =
                    self.hotspot.gpu_image_shared_resident_bytes.max(value)
            }
            "gpu_image_cache_key_allocations" => {
                self.hotspot.gpu_image_cache_key_allocation_count = self
                    .hotspot
                    .gpu_image_cache_key_allocation_count
                    .saturating_add(value)
            }
            "gpu_image_cache_prune_visits" => {
                self.hotspot.gpu_image_cache_prune_visit_count = self
                    .hotspot
                    .gpu_image_cache_prune_visit_count
                    .saturating_add(value)
            }
            "gpu_image_cache_admission_rejects" => {
                self.hotspot.gpu_image_cache_admission_reject_count = self
                    .hotspot
                    .gpu_image_cache_admission_reject_count
                    .saturating_add(value)
            }
            "gpu_image_invalid_payloads" => {
                self.hotspot.gpu_image_invalid_payload_count = self
                    .hotspot
                    .gpu_image_invalid_payload_count
                    .saturating_add(value)
            }
            "gpu_image_cache_resident_bytes" => {
                self.hotspot.gpu_image_cache_resident_bytes =
                    self.hotspot.gpu_image_cache_resident_bytes.max(value)
            }
            "gpu_image_prepare_command_visits" => {
                self.hotspot.gpu_image_prepare_command_visit_count = self
                    .hotspot
                    .gpu_image_prepare_command_visit_count
                    .saturating_add(value)
            }
            "gpu_image_prepare_cache_hits" => {
                self.hotspot.gpu_image_prepare_cache_hit_count = self
                    .hotspot
                    .gpu_image_prepare_cache_hit_count
                    .saturating_add(value)
            }
            "gpu_draw_calls" => {
                self.hotspot.gpu_draw_calls = self.hotspot.gpu_draw_calls.saturating_add(value)
            }
            "gpu_timestamp_supported_present_count" => {
                self.hotspot.gpu_timestamp_supported_present_count = self
                    .hotspot
                    .gpu_timestamp_supported_present_count
                    .saturating_add(value)
            }
            "gpu_time_us" => self.gpu_times_us.push(value),
            "gpu_profile_latency_frames" => {
                self.hotspot.gpu_profile_latency_max_frames =
                    self.hotspot.gpu_profile_latency_max_frames.max(value)
            }
            "gpu_visible_commands" => {
                self.hotspot.gpu_visible_commands =
                    self.hotspot.gpu_visible_commands.saturating_add(value)
            }
            "gpu_visible_draw_items" => {
                self.hotspot.gpu_visible_draw_items =
                    self.hotspot.gpu_visible_draw_items.saturating_add(value)
            }
            "gpu_compiled_draw_items" => {
                self.hotspot.gpu_compiled_draw_items =
                    self.hotspot.gpu_compiled_draw_items.saturating_add(value)
            }
            "gpu_batch_layers" => {
                self.hotspot.gpu_batch_layers = self.hotspot.gpu_batch_layers.saturating_add(value)
            }
            "gpu_batch_dependencies" => {
                self.hotspot.gpu_batch_dependencies =
                    self.hotspot.gpu_batch_dependencies.saturating_add(value)
            }
            "gpu_batch_plan_builds" => {
                self.hotspot.gpu_batch_plan_build_count = self
                    .hotspot
                    .gpu_batch_plan_build_count
                    .saturating_add(value)
            }
            "gpu_batch_plan_cache_hits" => {
                self.hotspot.gpu_batch_plan_cache_hit_count = self
                    .hotspot
                    .gpu_batch_plan_cache_hit_count
                    .saturating_add(value)
            }
            "gpu_vertex_buffer_creates" => {
                self.hotspot.gpu_vertex_buffer_create_count = self
                    .hotspot
                    .gpu_vertex_buffer_create_count
                    .saturating_add(value)
            }
            "gpu_vertex_upload_bytes" => {
                self.hotspot.gpu_vertex_upload_bytes =
                    self.hotspot.gpu_vertex_upload_bytes.saturating_add(value)
            }
            "gpu_retained_cache_copy_bytes" => {
                self.hotspot.gpu_retained_cache_copy_bytes = self
                    .hotspot
                    .gpu_retained_cache_copy_bytes
                    .saturating_add(value)
            }
            _ => {}
        }
    }

    fn finish(mut self) -> UiScenarioHotspot {
        self.frame_durations_us.sort_unstable();
        self.hotspot.frame_count = self.frame_durations_us.len() as u64;
        self.hotspot.frame_p95_us = percentile(&self.frame_durations_us, 95);
        self.hotspot.frame_max_us = self.frame_durations_us.last().copied().unwrap_or(0);
        self.gpu_times_us.sort_unstable();
        self.hotspot.gpu_time_sample_count = self.gpu_times_us.len() as u64;
        self.hotspot.gpu_time_p50_us = percentile(&self.gpu_times_us, 50);
        self.hotspot.gpu_time_p95_us = percentile(&self.gpu_times_us, 95);
        self.hotspot.gpu_time_max_us = self.gpu_times_us.last().copied().unwrap_or(0);
        self.hotspot
    }
}

fn counter_value(counter: &ProfileCounterSnapshot) -> u64 {
    if !counter.value.is_finite() || counter.value <= 0.0 {
        return 0;
    }
    counter.value.round().min(u64::MAX as f64) as u64
}

fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let index = ((values.len() - 1) * percentile).div_ceil(100);
    values[index.min(values.len() - 1)]
}

fn alerts_for_scenarios(scenarios: &[UiScenarioHotspot]) -> Vec<UiHotspotAlert> {
    let mut alerts = Vec::new();
    for scenario in scenarios {
        let chrome_command_count = scenario
            .chrome_command_full_rebuild_count
            .saturating_add(scenario.chrome_command_patch_count);
        if scenario.software_fallback_present_count > 0 {
            alerts.push(alert(
                scenario,
                "gpu_presenter_fell_back_to_software",
                "Retained host should use the GPU presenter in normal editor profiles; softbuffer presents are fallback-only.",
            ));
        }
        if chrome_command_count > 0
            && scenario.software_fallback_present_count == 0
            && scenario.gpu_draw_calls == 0
        {
            alerts.push(alert(
                scenario,
                "gpu_presenter_recorded_no_draw_calls",
                "A command stream was generated for the GPU presenter, but no GPU draw calls were recorded.",
            ));
        }
        if scenario.scenario == "viewport_image"
            && chrome_command_count > 0
            && scenario.software_fallback_present_count == 0
            && scenario.gpu_upload_bytes == 0
        {
            alerts.push(alert(
                scenario,
                "viewport_image_missing_gpu_upload",
                "Viewport image updates should upload texture bytes through the GPU presenter instead of dirtying layout or repainting through software.",
            ));
        }
        if scenario.gpu_image_cache_admission_reject_count > 0 {
            alerts.push(alert(
                scenario,
                "gpu_image_cache_rejected_active_resources",
                "The GPU image cache rejected resources during the scenario; inspect unstable image identities and cache budgets.",
            ));
        }
        if scenario.gpu_image_invalid_payload_count > 0 {
            alerts.push(alert(
                scenario,
                "gpu_image_payload_was_invalid",
                "The GPU image cache rejected malformed or incomplete image payloads.",
            ));
        }
        if matches!(scenario.scenario.as_str(), "idle_hover" | "click")
            && scenario.gpu_batch_plan_cache_hit_count > 0
        {
            alerts.push(alert(
                scenario,
                "live_patch_reused_compiled_full_projection",
                "Live hover/click damage streams are partial projections and must not reuse a versioned full-projection batch plan.",
            ));
        }
        if matches!(
            scenario.scenario.as_str(),
            "idle_hover" | "click" | "window_resize"
        ) && scenario.visual_asset_full_invalidation_count > 0
        {
            alerts.push(alert(
                scenario,
                "non_asset_interaction_cleared_visual_asset_caches",
                "Pointer and resize scenarios must not clear SVG, raster, or icon-atlas caches.",
            ));
        }
        if matches!(scenario.scenario.as_str(), "idle_hover" | "click")
            && scenario.presentation_rebuild_count > 0
        {
            alerts.push(alert(
                scenario,
                "non_structural_interaction_rebuilt_presentation",
                "Hover/click should stay paint-only unless component structure changed.",
            ));
        }
        if matches!(scenario.scenario.as_str(), "idle_hover" | "click")
            && scenario.asset_editor_pane_presentation_build_count > 0
        {
            alerts.push(alert(
                scenario,
                "non_structural_interaction_rebuilt_asset_editor_pane_presentation",
                "Hover/click should not rebuild the asset editor pane presentation unless the asset editor state changed.",
            ));
        }
        if scenario.scenario == "idle_hover"
            && (scenario.chrome_snapshot_count > 0 || scenario.workbench_model_build_count > 0)
        {
            alerts.push(alert(
                scenario,
                "hover_rebuilt_chrome_snapshot_or_model",
                "Hover should use committed retained-host caches instead of pulling a fresh chrome snapshot or workbench model.",
            ));
        }
        if matches!(
            scenario.scenario.as_str(),
            "idle_hover" | "asset_refresh" | "viewport_image"
        ) && scenario.redraw_full_frame_count > 0
        {
            alerts.push(alert(
                scenario,
                "region_redraw_degenerated_to_full_frame",
                "This scenario requested a full-frame redraw where region damage is expected.",
            ));
        }
        if matches!(
            scenario.scenario.as_str(),
            "idle_hover" | "click" | "drag" | "viewport_image"
        ) && scenario.redraw_region_count > 0
            && scenario.full_paint_count > 0
        {
            alerts.push(alert(
                scenario,
                "region_request_repainted_full_frame",
                "A region redraw request still caused a full-frame paint; inspect presenter damage retention and backbuffer validity.",
            ));
        }
        if scenario.scenario == "drawer_resize" && scenario.slow_path_rebuild_count > 0 {
            alerts.push(alert(
                scenario,
                "resize_triggered_slow_path_rebuild",
                "Dragging a drawer splitter is still entering the slow presentation/layout path.",
            ));
        }
        if scenario.scenario == "viewport_image"
            && (scenario.dirty_layout_count > 0 || scenario.dirty_presentation_count > 0)
        {
            alerts.push(alert(
                scenario,
                "viewport_image_dirtied_layout_or_presentation",
                "Viewport image updates should not dirty layout or presentation data.",
            ));
        }
        if scenario.software_fallback_present_count == 0
            && scenario.gpu_batch_dependencies == 0
            && scenario.gpu_visible_draw_items >= 4
            && scenario.gpu_draw_calls >= scenario.gpu_visible_draw_items
        {
            alerts.push(alert(
                scenario,
                "gpu_ui_batching_degenerated_without_depth_dependencies",
                "The GPU UI presenter saw independent draw items but did not reduce draw calls; inspect depth batching and material grouping.",
            ));
        }
    }
    alerts
}

fn alert(scenario: &UiScenarioHotspot, rule: &str, message: &str) -> UiHotspotAlert {
    UiHotspotAlert {
        scenario: scenario.scenario.clone(),
        rule: rule.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
#[path = "ui_hotspot/tests/mod.rs"]
mod tests;
