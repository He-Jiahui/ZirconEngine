use std::hint::black_box;
use std::time::Instant;

use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use super::{frame_summary, presentation_summary};

const SAMPLE_PAIRS: usize = 31;
const SUMMARIES_PER_SAMPLE: usize = 10_000;

#[test]
fn optimization_batch_20260829z_editor245_presentation_summary_preserves_bytes() {
    let presentation = fixture();
    assert_eq!(
        presentation_summary(&presentation),
        legacy_presentation_summary(&presentation)
    );
    assert_eq!(
        frame_summary(&presentation.host_layout.center_band_frame),
        "8.0,32.0,1280.0,720.0"
    );
}

#[test]
fn optimization_batch_20260829z_editor245_presentation_summary_uses_one_buffer() {
    let source = include_str!("../summary.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let body = implementation
        .split("fn presentation_summary")
        .nth(1)
        .and_then(|body| {
            body.split("pub(in crate::ui::retained_host::host_contract) fn frame_summary")
                .next()
        })
        .expect("presentation summary builder");

    assert!(body.contains("String::with_capacity"));
    assert!(body.contains("write!("));
    assert!(body.contains("&mut summary"));
    assert!(body.contains("FrameSummary("));
    assert!(!body.contains("frame_summary(&"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829z_editor245_single_buffer_presentation_summary_bench() {
    let presentation = fixture();
    assert_eq!(
        presentation_summary(&presentation),
        legacy_presentation_summary(&presentation)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &presentation));
            optimized_samples.push(measure(true, &presentation));
        } else {
            optimized_samples.push(measure(true, &presentation));
            legacy_samples.push(measure(false, &presentation));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR245_SINGLE_BUFFER_PRESENTATION_SUMMARY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
summaries_per_sample={SUMMARIES_PER_SAMPLE} frame_count=7 \
legacy_result_allocations_per_summary=8 optimized_result_allocations_per_summary=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn fixture() -> HostWindowPresentationData {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_shell.project_path = "E:/Projects/Zircon/diagnostics-fixture".into();
    presentation.host_shell.viewport_label = "Scene Viewport".into();
    presentation.host_shell.status_secondary = "Ready for incremental presentation".into();
    presentation.host_layout.center_band_frame = frame(8.0, 32.0, 1280.0, 720.0);
    presentation.host_layout.status_bar_frame = frame(0.0, 752.0, 1296.0, 24.0);
    presentation.host_layout.document_region_frame = frame(248.0, 64.0, 800.0, 520.0);
    presentation.host_layout.viewport_content_frame = frame(256.0, 72.0, 784.0, 496.0);
    presentation.host_layout.left_region_frame = frame(0.0, 64.0, 240.0, 688.0);
    presentation.host_layout.right_region_frame = frame(1056.0, 64.0, 240.0, 688.0);
    presentation.host_layout.bottom_region_frame = frame(248.0, 592.0, 800.0, 160.0);
    presentation.host_scene_data.document_dock.pane.kind = "SceneDocument".into();
    presentation.host_scene_data.left_dock.pane.kind = "Hierarchy".into();
    presentation.host_scene_data.right_dock.pane.kind = "Inspector".into();
    presentation.host_scene_data.bottom_dock.pane.kind = "Diagnostics".into();
    presentation
}

fn frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

fn legacy_presentation_summary(presentation: &HostWindowPresentationData) -> String {
    let layout = &presentation.host_layout;
    let scene = &presentation.host_scene_data;
    format!(
        "project_path={} viewport_label={} status={} center={} status_bar={} document={} viewport={} left={} right={} bottom={} page_tabs={} document_tabs={} left_tabs={} right_tabs={} bottom_tabs={} floating_windows={} document_pane_kind={} left_pane_kind={} right_pane_kind={} bottom_pane_kind={}",
        presentation.host_shell.project_path,
        presentation.host_shell.viewport_label,
        presentation.host_shell.status_secondary,
        frame_summary(&layout.center_band_frame),
        frame_summary(&layout.status_bar_frame),
        frame_summary(&layout.document_region_frame),
        frame_summary(&layout.viewport_content_frame),
        frame_summary(&layout.left_region_frame),
        frame_summary(&layout.right_region_frame),
        frame_summary(&layout.bottom_region_frame),
        scene.page_chrome.tabs.row_count(),
        scene.document_dock.tabs.row_count(),
        scene.left_dock.tabs.row_count(),
        scene.right_dock.tabs.row_count(),
        scene.bottom_dock.tabs.row_count(),
        scene.floating_layer.floating_windows.row_count(),
        scene.document_dock.pane.kind,
        scene.left_dock.pane.kind,
        scene.right_dock.pane.kind,
        scene.bottom_dock.pane.kind,
    )
}

fn measure(optimized: bool, presentation: &HostWindowPresentationData) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..SUMMARIES_PER_SAMPLE {
        let summary = if optimized {
            presentation_summary(black_box(presentation))
        } else {
            legacy_presentation_summary(black_box(presentation))
        };
        checksum = checksum.wrapping_add(black_box(summary).len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
