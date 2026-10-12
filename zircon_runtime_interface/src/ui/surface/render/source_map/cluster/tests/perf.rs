use std::{hint::black_box, time::Instant};

use crate::ui::{
    layout::UiFrame,
    surface::{UiResolvedTextLine, UiResolvedTextRun, UiTextDirection, UiTextRange, UiTextRunKind},
};

use super::{visual_source_clusters, visual_source_clusters_linear};

const RUN_COUNT: usize = 2_048;
const ITERATION_COUNT: usize = 4;
const SAMPLE_COUNT: usize = 11;

fn dense_line() -> UiResolvedTextLine {
    let text = "x".repeat(RUN_COUNT);
    let runs = (0..RUN_COUNT)
        .map(|index| UiResolvedTextRun {
            kind: UiTextRunKind::Plain,
            text: "x".to_string(),
            source_range: UiTextRange {
                start: index,
                end: index + 1,
            },
            visual_range: UiTextRange {
                start: index,
                end: index + 1,
            },
            direction: UiTextDirection::LeftToRight,
        })
        .collect();
    UiResolvedTextLine {
        text,
        frame: UiFrame::new(0.0, 0.0, RUN_COUNT as f32, 1.0),
        placement_frame: UiFrame::new(0.0, 0.0, RUN_COUNT as f32, 1.0),
        source_range: UiTextRange {
            start: 0,
            end: RUN_COUNT,
        },
        visual_range: UiTextRange {
            start: 0,
            end: RUN_COUNT,
        },
        measured_width: RUN_COUNT as f32,
        glyph_advances: vec![1.0; RUN_COUNT],
        baseline: 1.0,
        direction: UiTextDirection::LeftToRight,
        runs,
        ellipsized: false,
    }
}

#[test]
fn runtime_interface03_batch13_visual_cluster_cursor_preserves_projection() {
    let ordered = dense_line();
    assert_eq!(
        visual_source_clusters(&ordered),
        visual_source_clusters_linear(&ordered),
    );

    let mut unordered = ordered;
    unordered.runs.reverse();
    assert_eq!(
        visual_source_clusters(&unordered),
        visual_source_clusters_linear(&unordered),
    );
}

#[test]
#[ignore = "release-only visual-cluster run-cursor benchmark"]
fn runtime_interface03_batch13_visual_cluster_run_cursor_release_benchmark() {
    let line = dense_line();
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut cursor_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for _ in 0..ITERATION_COUNT {
                black_box(visual_source_clusters_linear(black_box(&line)));
            }
            started.elapsed().as_nanos()
        };
        let measure_cursor = || {
            let started = Instant::now();
            for _ in 0..ITERATION_COUNT {
                black_box(visual_source_clusters(black_box(&line)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            cursor_samples.push(measure_cursor());
        } else {
            cursor_samples.push(measure_cursor());
            linear_samples.push(measure_linear());
        }
    }

    linear_samples.sort_unstable();
    cursor_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_VISUAL_CLUSTER_RUN_CURSOR_BENCH_V1 runs={RUN_COUNT} iterations={ITERATION_COUNT} samples={SAMPLE_COUNT} linear_p95_ns={} cursor_p95_ns={}",
        linear_samples[p95],
        cursor_samples[p95],
    );
    assert!(
        cursor_samples[p95].saturating_mul(10) <= linear_samples[p95],
        "visual-cluster cursor must improve P95 by at least 90%: linear={}ns cursor={}ns",
        linear_samples[p95],
        cursor_samples[p95],
    );
}
