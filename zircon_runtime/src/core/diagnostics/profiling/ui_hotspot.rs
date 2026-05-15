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
}

impl UiScenarioAccumulator {
    fn new(scenario: &str) -> Self {
        Self {
            hotspot: UiScenarioHotspot::empty(scenario),
            frame_durations_us: Vec::new(),
        }
    }

    fn record(&mut self, metric: &str, counter: &ProfileCounterSnapshot) {
        let value = counter_value(counter);
        match metric {
            "frame_duration_us" => self.frame_durations_us.push(value),
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
            "gpu_draw_calls" => {
                self.hotspot.gpu_draw_calls = self.hotspot.gpu_draw_calls.saturating_add(value)
            }
            "gpu_visible_commands" => {
                self.hotspot.gpu_visible_commands =
                    self.hotspot.gpu_visible_commands.saturating_add(value)
            }
            "gpu_visible_draw_items" => {
                self.hotspot.gpu_visible_draw_items =
                    self.hotspot.gpu_visible_draw_items.saturating_add(value)
            }
            "gpu_batch_layers" => {
                self.hotspot.gpu_batch_layers = self.hotspot.gpu_batch_layers.saturating_add(value)
            }
            "gpu_batch_dependencies" => {
                self.hotspot.gpu_batch_dependencies =
                    self.hotspot.gpu_batch_dependencies.saturating_add(value)
            }
            _ => {}
        }
    }

    fn finish(mut self) -> UiScenarioHotspot {
        self.frame_durations_us.sort_unstable();
        self.hotspot.frame_count = self.frame_durations_us.len() as u64;
        self.hotspot.frame_p95_us = percentile(&self.frame_durations_us, 95);
        self.hotspot.frame_max_us = self.frame_durations_us.last().copied().unwrap_or(0);
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
        if matches!(scenario.scenario.as_str(), "idle_hover" | "click")
            && scenario.presentation_rebuild_count > 0
        {
            alerts.push(alert(
                scenario,
                "non_structural_interaction_rebuilt_presentation",
                "Hover/click should stay paint-only unless component structure changed.",
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
mod tests {
    use zircon_runtime_interface::{ProfileCounterSnapshot, ProfileSnapshot};

    fn counter(name: &str, value: f64) -> ProfileCounterSnapshot {
        ProfileCounterSnapshot {
            stream: "editor".to_string(),
            name: name.to_string(),
            value,
            timestamp_us: 0,
            frame_index: None,
        }
    }

    #[test]
    fn ui_hotspots_group_counters_by_scenario() {
        let mut snapshot = ProfileSnapshot::default();
        snapshot
            .counters
            .push(counter("ui.idle_hover.redraw_region", 2.0));
        snapshot
            .counters
            .push(counter("ui.idle_hover.chrome_command_patch_count", 3.0));
        snapshot
            .counters
            .push(counter("ui.idle_hover.painted_pixels", 120.0));
        snapshot
            .counters
            .push(counter("ui.drawer_resize.slow_path_rebuild_count", 1.0));
        snapshot
            .counters
            .push(counter("runtime.unrelated.counter", 1.0));

        let report = super::analyze_ui_hotspots(&snapshot);

        assert_eq!(report.generated_from_counter_count, 4);
        let idle = report
            .scenarios
            .iter()
            .find(|scenario| scenario.scenario == "idle_hover")
            .expect("idle hover scenario");
        assert_eq!(idle.redraw_region_count, 2);
        assert_eq!(idle.chrome_command_patch_count, 3);
        assert_eq!(idle.painted_pixels, 120);
        assert!(report
            .alerts
            .iter()
            .any(|alert| alert.rule == "resize_triggered_slow_path_rebuild"));
    }

    #[test]
    fn hover_presentation_rebuild_is_flagged() {
        let mut snapshot = ProfileSnapshot::default();
        snapshot
            .counters
            .push(counter("ui.idle_hover.presentation_rebuild_count", 1.0));

        let report = super::analyze_ui_hotspots(&snapshot);

        assert!(report
            .alerts
            .iter()
            .any(|alert| alert.rule == "non_structural_interaction_rebuilt_presentation"));
    }

    #[test]
    fn hover_snapshot_or_model_rebuild_is_flagged() {
        let mut snapshot = ProfileSnapshot::default();
        snapshot
            .counters
            .push(counter("ui.idle_hover.chrome_snapshot_count", 1.0));
        snapshot
            .counters
            .push(counter("ui.idle_hover.workbench_model_build_count", 1.0));

        let report = super::analyze_ui_hotspots(&snapshot);

        let idle = report
            .scenarios
            .iter()
            .find(|scenario| scenario.scenario == "idle_hover")
            .expect("idle hover scenario");
        assert_eq!(idle.chrome_snapshot_count, 1);
        assert_eq!(idle.workbench_model_build_count, 1);
        assert!(report
            .alerts
            .iter()
            .any(|alert| alert.rule == "hover_rebuilt_chrome_snapshot_or_model"));
    }

    #[test]
    fn region_request_that_repaints_full_frame_is_flagged() {
        let mut snapshot = ProfileSnapshot::default();
        snapshot
            .counters
            .push(counter("ui.idle_hover.redraw_region", 1.0));
        snapshot
            .counters
            .push(counter("ui.idle_hover.full_paint_count", 1.0));

        let report = super::analyze_ui_hotspots(&snapshot);

        assert!(report
            .alerts
            .iter()
            .any(|alert| alert.rule == "region_request_repainted_full_frame"));
    }

    #[test]
    fn ui_hotspots_collect_gpu_presenter_counters() {
        let mut snapshot = ProfileSnapshot::default();
        snapshot
            .counters
            .push(counter("ui.viewport_image.gpu_upload_bytes", 1024.0));
        snapshot
            .counters
            .push(counter("ui.viewport_image.gpu_draw_calls", 7.0));
        snapshot
            .counters
            .push(counter("ui.viewport_image.gpu_visible_commands", 11.0));
        snapshot
            .counters
            .push(counter("ui.viewport_image.gpu_visible_draw_items", 13.0));
        snapshot
            .counters
            .push(counter("ui.viewport_image.gpu_batch_layers", 2.0));
        snapshot
            .counters
            .push(counter("ui.viewport_image.gpu_batch_dependencies", 3.0));

        let report = super::analyze_ui_hotspots(&snapshot);

        let viewport = report
            .scenarios
            .iter()
            .find(|scenario| scenario.scenario == "viewport_image")
            .expect("viewport image scenario");
        assert_eq!(viewport.gpu_upload_bytes, 1024);
        assert_eq!(viewport.gpu_draw_calls, 7);
        assert_eq!(viewport.gpu_visible_commands, 11);
        assert_eq!(viewport.gpu_visible_draw_items, 13);
        assert_eq!(viewport.gpu_batch_layers, 2);
        assert_eq!(viewport.gpu_batch_dependencies, 3);
    }

    #[test]
    fn software_fallback_present_is_flagged_for_gpu_profile() {
        let mut snapshot = ProfileSnapshot::default();
        snapshot.counters.push(counter(
            "ui.idle_hover.software_fallback_present_count",
            1.0,
        ));

        let report = super::analyze_ui_hotspots(&snapshot);

        assert!(report
            .alerts
            .iter()
            .any(|alert| alert.rule == "gpu_presenter_fell_back_to_software"));
    }

    #[test]
    fn gpu_command_stream_without_draw_calls_is_flagged() {
        let mut snapshot = ProfileSnapshot::default();
        snapshot
            .counters
            .push(counter("ui.idle_hover.chrome_command_patch_count", 1.0));

        let report = super::analyze_ui_hotspots(&snapshot);

        assert!(report
            .alerts
            .iter()
            .any(|alert| alert.rule == "gpu_presenter_recorded_no_draw_calls"));
    }

    #[test]
    fn viewport_image_command_without_gpu_upload_is_flagged() {
        let mut snapshot = ProfileSnapshot::default();
        snapshot
            .counters
            .push(counter("ui.viewport_image.chrome_command_patch_count", 1.0));
        snapshot
            .counters
            .push(counter("ui.viewport_image.gpu_draw_calls", 1.0));

        let report = super::analyze_ui_hotspots(&snapshot);

        assert!(report
            .alerts
            .iter()
            .any(|alert| alert.rule == "viewport_image_missing_gpu_upload"));
    }

    #[test]
    fn independent_gpu_items_without_batch_reduction_are_flagged() {
        let mut snapshot = ProfileSnapshot::default();
        snapshot
            .counters
            .push(counter("ui.idle_hover.gpu_draw_calls", 5.0));
        snapshot
            .counters
            .push(counter("ui.idle_hover.gpu_visible_draw_items", 5.0));

        let report = super::analyze_ui_hotspots(&snapshot);

        assert!(report.alerts.iter().any(|alert| {
            alert.rule == "gpu_ui_batching_degenerated_without_depth_dependencies"
        }));
    }
}
