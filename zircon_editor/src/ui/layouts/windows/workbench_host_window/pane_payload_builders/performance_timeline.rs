use zircon_runtime::core::diagnostics::{
    analyze_hotspots, ProfileSnapshot, RuntimeDiagnosticsSnapshot,
};

use super::super::pane_payload::{
    PanePayload, PerformanceTimelineCaptureControlPayload, PerformanceTimelineFrameRowPayload,
    PerformanceTimelineHotspotRowPayload, PerformanceTimelinePanePayload,
    PerformanceTimelineSpanRowPayload,
};
use super::super::pane_presentation::PanePayloadBuildContext;

const MICROS_PER_MILLI: f64 = 1_000.0;
const MAX_TIMELINE_ROWS: usize = 12;
const MIN_FRAME_BUDGET_MS: f64 = 0.001;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    let diagnostics = context
        .runtime_diagnostics
        .cloned()
        .unwrap_or_else(RuntimeDiagnosticsSnapshot::default);
    let profile = diagnostics.profile;
    let hotspot_report = analyze_hotspots(&profile);

    PanePayload::PerformanceTimelineV1(PerformanceTimelinePanePayload {
        summary: summary(&profile),
        session_label: format!("Session {}", profile.session_id),
        output_label: format!("Output {}/{}", profile.output_root, profile.session_id),
        frame_rows: frame_rows(&profile),
        span_summary_rows: span_rows(&profile),
        hotspot_rows: hotspot_report
            .hotspots
            .iter()
            .take(MAX_TIMELINE_ROWS)
            .map(|hotspot| PerformanceTimelineHotspotRowPayload {
                stream: hotspot.stream.clone(),
                category: hotspot.category.clone(),
                name: hotspot.name.clone(),
                path: hotspot.path.clone(),
                total_label: format!("{} total", duration_label(hotspot.total_us)),
                average_label: format!("{} avg", duration_label(hotspot.avg_us)),
                count_label: format!("{} spans", hotspot.count),
            })
            .collect(),
        capture_controls: capture_controls(&profile),
    })
}

fn summary(profile: &ProfileSnapshot) -> String {
    let state = if profile.feature_enabled {
        if profile.active {
            "active"
        } else {
            "ready"
        }
    } else {
        "disabled"
    };
    format!(
        "Profiling {state}: {}, {}, {}",
        plural(profile.frames.len(), "frame"),
        plural(profile.spans.len(), "span"),
        plural(profile.counters.len(), "counter"),
    )
}

fn frame_rows(profile: &ProfileSnapshot) -> Vec<PerformanceTimelineFrameRowPayload> {
    profile
        .frames
        .iter()
        .rev()
        .take(MAX_TIMELINE_ROWS)
        .map(|frame| {
            let bar = frame_bar_metadata(frame.duration_us, frame.budget_ms);
            PerformanceTimelineFrameRowPayload {
                stream: frame.stream.clone(),
                name: frame.name.clone(),
                frame_index: frame.frame_index,
                duration_label: duration_label(frame.duration_us),
                budget_label: format!("{:.2} ms budget", frame.budget_ms),
                budget_usage_label: bar.budget_usage_label,
                duration_ratio: bar.duration_ratio,
                bar_fill_ratio: bar.bar_fill_ratio,
                budget_marker_ratio: bar.budget_marker_ratio,
                over_budget: frame.over_budget,
            }
        })
        .collect()
}

struct FrameBarMetadata {
    budget_usage_label: String,
    duration_ratio: f32,
    bar_fill_ratio: f32,
    budget_marker_ratio: f32,
}

fn frame_bar_metadata(duration_us: u64, budget_ms: f64) -> FrameBarMetadata {
    let duration_ms = duration_us as f64 / MICROS_PER_MILLI;
    let budget_ms = finite_positive_budget_ms(budget_ms);
    let scale_ms = duration_ms.max(budget_ms).max(MIN_FRAME_BUDGET_MS);
    let duration_ratio = (duration_ms / budget_ms).max(0.0);

    FrameBarMetadata {
        budget_usage_label: format!("{:.0}% budget", duration_ratio * 100.0),
        duration_ratio: duration_ratio as f32,
        bar_fill_ratio: ratio_for_ui(duration_ms / scale_ms),
        budget_marker_ratio: ratio_for_ui(budget_ms / scale_ms),
    }
}

fn finite_positive_budget_ms(budget_ms: f64) -> f64 {
    if budget_ms.is_finite() && budget_ms > 0.0 {
        budget_ms
    } else {
        MIN_FRAME_BUDGET_MS
    }
}

fn ratio_for_ui(value: f64) -> f32 {
    value.clamp(0.0, 1.0) as f32
}

fn span_rows(profile: &ProfileSnapshot) -> Vec<PerformanceTimelineSpanRowPayload> {
    profile
        .spans
        .iter()
        .rev()
        .take(MAX_TIMELINE_ROWS)
        .map(|span| PerformanceTimelineSpanRowPayload {
            stream: span.stream.clone(),
            category: span.category.clone(),
            name: span.name.clone(),
            path: span.path.clone(),
            duration_label: duration_label(span.duration_us),
            depth: span.depth,
        })
        .collect()
}

fn capture_controls(profile: &ProfileSnapshot) -> Vec<PerformanceTimelineCaptureControlPayload> {
    vec![
        PerformanceTimelineCaptureControlPayload {
            label: if profile.active {
                "Stop Capture".to_string()
            } else {
                "Start Capture".to_string()
            },
            action_id: if profile.active {
                "PerformanceTimeline.StopCapture".to_string()
            } else {
                "PerformanceTimeline.StartCapture".to_string()
            },
            enabled: profile.feature_enabled,
        },
        PerformanceTimelineCaptureControlPayload {
            label: "Export Report".to_string(),
            action_id: "PerformanceTimeline.ExportReport".to_string(),
            enabled: profile.feature_enabled && has_samples(profile),
        },
        PerformanceTimelineCaptureControlPayload {
            label: "Reset".to_string(),
            action_id: "PerformanceTimeline.Reset".to_string(),
            enabled: profile.feature_enabled && has_samples(profile),
        },
    ]
}

fn has_samples(profile: &ProfileSnapshot) -> bool {
    !profile.frames.is_empty() || !profile.spans.is_empty() || !profile.counters.is_empty()
}

fn duration_label(duration_us: u64) -> String {
    format!("{:.2} ms", duration_us as f64 / MICROS_PER_MILLI)
}

fn plural(count: usize, singular: &str) -> String {
    if count == 1 {
        format!("1 {singular}")
    } else {
        format!("{count} {singular}s")
    }
}
