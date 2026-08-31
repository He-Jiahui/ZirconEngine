use zircon_runtime_interface::ui::surface::{
    UiDebugTimelineFrameHandle, UiDebugTimelineFrameSummary, UiDebugTimelineSnapshot,
    UiSurfaceDebugSnapshot,
};

use super::model::EditorUiDebugReflectorModel;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct EditorUiDebugTimelineModel {
    pub retention: String,
    pub selected: String,
    pub latest: String,
    pub previous_frame: Option<UiDebugTimelineFrameHandle>,
    pub next_frame: Option<UiDebugTimelineFrameHandle>,
    pub frame_rows: Vec<String>,
    pub selected_reflector: EditorUiDebugReflectorModel,
}

#[derive(Clone, Copy)]
struct TimelineProjection<'a> {
    selected_summary: Option<&'a UiDebugTimelineFrameSummary>,
    selected_snapshot: Option<&'a UiSurfaceDebugSnapshot>,
    previous_frame: Option<UiDebugTimelineFrameHandle>,
    next_frame: Option<UiDebugTimelineFrameHandle>,
    latest_summary: Option<&'a UiDebugTimelineFrameSummary>,
}

impl EditorUiDebugTimelineModel {
    pub(crate) fn from_timeline(timeline: &UiDebugTimelineSnapshot) -> Self {
        let projection = resolve_timeline_projection(timeline);

        Self {
            retention: retention_label(timeline),
            selected: projection
                .selected_summary
                .map(selected_label)
                .unwrap_or_else(|| "Selected frame: none".to_string()),
            latest: projection
                .latest_summary
                .map(latest_label)
                .unwrap_or_else(|| "Latest frame: none".to_string()),
            previous_frame: projection.previous_frame,
            next_frame: projection.next_frame,
            frame_rows: timeline.summaries.iter().map(frame_row).collect(),
            selected_reflector: projection
                .selected_snapshot
                .map(EditorUiDebugReflectorModel::from_snapshot)
                .unwrap_or_else(EditorUiDebugReflectorModel::no_active_surface),
        }
    }
}

fn resolve_timeline_projection(timeline: &UiDebugTimelineSnapshot) -> TimelineProjection<'_> {
    let selected = timeline.selected_frame.and_then(|handle| {
        summary_index(timeline, handle).map(|index| (Some(handle), Some(index)))
    });
    let (selected_handle, selected_index) = selected.unwrap_or_else(|| {
        let handle = timeline.retention.latest_frame;
        let index = handle.and_then(|handle| summary_index(timeline, handle));
        (handle, index)
    });
    let selected_summary = selected_index.and_then(|index| timeline.summaries.get(index));
    let selected_snapshot = selected_index.and_then(|index| timeline.frames.get(index));
    let (previous_frame, next_frame) = selected_index
        .map(|index| neighbors_at(timeline, index))
        .unwrap_or((None, None));
    let latest_summary = match timeline.retention.latest_frame {
        Some(handle) if Some(handle) == selected_handle => selected_summary,
        Some(handle) => summary_for_handle(timeline, handle),
        None => None,
    };
    TimelineProjection {
        selected_summary,
        selected_snapshot,
        previous_frame,
        next_frame,
        latest_summary,
    }
}

fn summary_for_handle(
    timeline: &UiDebugTimelineSnapshot,
    handle: UiDebugTimelineFrameHandle,
) -> Option<&UiDebugTimelineFrameSummary> {
    summary_index(timeline, handle).and_then(|index| timeline.summaries.get(index))
}

fn summary_index(
    timeline: &UiDebugTimelineSnapshot,
    handle: UiDebugTimelineFrameHandle,
) -> Option<usize> {
    timeline
        .summaries
        .iter()
        .position(|summary| summary.handle == handle)
}

fn neighbors_at(
    timeline: &UiDebugTimelineSnapshot,
    index: usize,
) -> (
    Option<UiDebugTimelineFrameHandle>,
    Option<UiDebugTimelineFrameHandle>,
) {
    let previous = index
        .checked_sub(1)
        .and_then(|previous| timeline.summaries.get(previous))
        .map(|summary| summary.handle);
    let next = timeline
        .summaries
        .get(index + 1)
        .map(|summary| summary.handle);
    (previous, next)
}

fn retention_label(timeline: &UiDebugTimelineSnapshot) -> String {
    format!(
        "Timeline: {}/{} frames retained, dropped {}, first={}, latest={}, selected={}",
        timeline.retention.len,
        timeline.retention.capacity,
        timeline.retention.dropped_frame_count,
        handle_label(timeline.retention.first_frame),
        handle_label(timeline.retention.latest_frame),
        handle_label(timeline.retention.selected_frame),
    )
}

fn selected_label(summary: &UiDebugTimelineFrameSummary) -> String {
    format!(
        "Selected frame: handle={} frame={} source={} nodes={} commands={}",
        summary.handle.0,
        summary.frame_index,
        summary.source_label,
        summary.node_count,
        summary.render_command_count,
    )
}

fn latest_label(summary: &UiDebugTimelineFrameSummary) -> String {
    format!(
        "Latest frame: handle={} frame={} source={}",
        summary.handle.0, summary.frame_index, summary.source_label,
    )
}

fn frame_row(summary: &UiDebugTimelineFrameSummary) -> String {
    format!(
        "handle={} frame={} source={} nodes={} commands={} hit_cells={} dirty={} warnings={} selected_node={:?}",
        summary.handle.0,
        summary.frame_index,
        summary.source_label,
        summary.node_count,
        summary.render_command_count,
        summary.hit_grid_cell_count,
        summary.invalidation_dirty_count,
        summary.warning_count,
        summary.selected_node.map(|node| node.0),
    )
}

fn handle_label(handle: Option<UiDebugTimelineFrameHandle>) -> String {
    handle
        .map(|handle| handle.0.to_string())
        .unwrap_or_else(|| "none".to_string())
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    use super::*;
    use zircon_runtime_interface::ui::surface::{UiDebugTimelineRetention, UiSurfaceDebugOptions};

    const BENCHMARK_FRAME_COUNT: usize = 4_096;
    const BENCHMARK_SAMPLES: usize = 11;
    const BENCHMARK_ITERATIONS: usize = 256;

    #[test]
    fn single_pass_timeline_selection_preserves_resolution_semantics() {
        for timeline in [
            timeline_fixture(Some(3), Some(3), 4),
            timeline_fixture(Some(99), Some(3), 4),
            timeline_fixture(None, Some(99), 4),
            timeline_fixture(None, None, 0),
        ] {
            assert_eq!(
                projection_signature(resolve_timeline_projection(&timeline)),
                retired_projection_signature(&timeline)
            );
        }
    }

    #[test]
    fn single_pass_timeline_selection_source_contract() {
        let source = include_str!("timeline.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .expect("production module end")
            .0;
        let constructor = production
            .split_once("pub(crate) fn from_timeline")
            .expect("timeline constructor")
            .1
            .split_once("\n    }\n}")
            .expect("timeline constructor end")
            .0;

        assert_eq!(
            constructor.matches("resolve_timeline_projection").count(),
            1
        );
        assert!(!production.contains("fn selected_handle("));
        assert!(!production.contains("fn snapshot_for_handle("));
        assert!(!production.contains("fn timeline_neighbors("));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn single_pass_timeline_selection_release_benchmark() {
        let timeline = timeline_fixture(
            Some(BENCHMARK_FRAME_COUNT as u64),
            Some(BENCHMARK_FRAME_COUNT as u64),
            BENCHMARK_FRAME_COUNT,
        );
        let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
        let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

        for sample in 0..BENCHMARK_SAMPLES {
            if sample % 2 == 0 {
                retired_samples.push(measure_projection(|| {
                    retired_projection_signature(&timeline)
                }));
                optimized_samples.push(measure_projection(|| {
                    projection_signature(resolve_timeline_projection(&timeline))
                }));
            } else {
                optimized_samples.push(measure_projection(|| {
                    projection_signature(resolve_timeline_projection(&timeline))
                }));
                retired_samples.push(measure_projection(|| {
                    retired_projection_signature(&timeline)
                }));
            }
        }

        let retired_p95 = percentile_95(&mut retired_samples);
        let optimized_p95 = percentile_95(&mut optimized_samples);
        let reduction_basis_points = 10_000_u128.saturating_sub(
            optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
        );
        eprintln!(
            "EDITOR25_SINGLE_PASS_TIMELINE_SELECTION_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} \
frames={BENCHMARK_FRAME_COUNT} retired_handle_comparisons_per_projection=20480 \
optimized_handle_comparisons_per_projection=4096 retired_p95_ns={} optimized_p95_ns={} \
reduction_basis_points={reduction_basis_points}",
            retired_p95.as_nanos(),
            optimized_p95.as_nanos(),
        );
        assert!(
            optimized_p95.as_nanos().saturating_mul(100)
                <= retired_p95.as_nanos().saturating_mul(35),
            "single-pass timeline selection must reduce projection P95 by at least 65%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
        );
    }

    type ProjectionSignature = (Option<u64>, bool, Option<u64>, Option<u64>, Option<u64>);

    fn projection_signature(projection: TimelineProjection<'_>) -> ProjectionSignature {
        (
            projection.selected_summary.map(|summary| summary.handle.0),
            projection.selected_snapshot.is_some(),
            projection.previous_frame.map(|handle| handle.0),
            projection.next_frame.map(|handle| handle.0),
            projection.latest_summary.map(|summary| summary.handle.0),
        )
    }

    fn retired_projection_signature(timeline: &UiDebugTimelineSnapshot) -> ProjectionSignature {
        let selected_handle = timeline
            .selected_frame
            .filter(|handle| retired_summary(timeline, *handle).is_some())
            .or(timeline.retention.latest_frame);
        let selected_summary = selected_handle.and_then(|handle| retired_summary(timeline, handle));
        let selected_snapshot = selected_handle
            .and_then(|handle| retired_index(timeline, handle))
            .and_then(|index| timeline.frames.get(index));
        let (previous_frame, next_frame) = selected_handle
            .and_then(|handle| retired_index(timeline, handle))
            .map(|index| neighbors_at(timeline, index))
            .unwrap_or((None, None));
        let latest_summary = timeline
            .retention
            .latest_frame
            .and_then(|handle| retired_summary(timeline, handle));

        (
            selected_summary.map(|summary| summary.handle.0),
            selected_snapshot.is_some(),
            previous_frame.map(|handle| handle.0),
            next_frame.map(|handle| handle.0),
            latest_summary.map(|summary| summary.handle.0),
        )
    }

    fn retired_summary(
        timeline: &UiDebugTimelineSnapshot,
        handle: UiDebugTimelineFrameHandle,
    ) -> Option<&UiDebugTimelineFrameSummary> {
        timeline
            .summaries
            .iter()
            .find(|summary| summary.handle == handle)
    }

    fn retired_index(
        timeline: &UiDebugTimelineSnapshot,
        handle: UiDebugTimelineFrameHandle,
    ) -> Option<usize> {
        timeline
            .summaries
            .iter()
            .position(|summary| summary.handle == handle)
    }

    fn timeline_fixture(
        selected: Option<u64>,
        latest: Option<u64>,
        frame_count: usize,
    ) -> UiDebugTimelineSnapshot {
        UiDebugTimelineSnapshot {
            selected_frame: selected.map(UiDebugTimelineFrameHandle),
            summaries: (1..=frame_count as u64).map(frame_summary).collect(),
            frames: vec![UiSurfaceDebugSnapshot::default(); frame_count],
            retention: UiDebugTimelineRetention {
                capacity: frame_count,
                len: frame_count,
                first_frame: (frame_count != 0).then_some(UiDebugTimelineFrameHandle(1)),
                latest_frame: latest.map(UiDebugTimelineFrameHandle),
                selected_frame: selected.map(UiDebugTimelineFrameHandle),
                dropped_frame_count: 0,
            },
        }
    }

    fn frame_summary(handle: u64) -> UiDebugTimelineFrameSummary {
        UiDebugTimelineFrameSummary {
            handle: UiDebugTimelineFrameHandle(handle),
            frame_index: handle,
            captured_at_millis: None,
            source_target_id: String::new(),
            source_label: String::new(),
            schema_version: 1,
            node_count: 0,
            render_command_count: 0,
            hit_grid_cell_count: 0,
            invalidation_dirty_count: 0,
            has_damage_region: false,
            warning_count: 0,
            selected_node: None,
            capture_options: UiSurfaceDebugOptions::default(),
        }
    }

    fn measure_projection(mut project: impl FnMut() -> ProjectionSignature) -> Duration {
        let started = Instant::now();
        for _ in 0..BENCHMARK_ITERATIONS {
            black_box(project());
        }
        started.elapsed()
    }

    fn percentile_95(samples: &mut [Duration]) -> Duration {
        samples.sort_unstable();
        samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
    }
}
