use std::{hint::black_box, time::Instant};

use crate::ui::{
    layout::UiFrame,
    surface::{
        UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun, UiShapedGlyph,
        UiTextDirection, UiTextOverflow, UiTextRange, UiTextRenderMode, UiTextRunKind,
        UiTextWritingMode,
    },
};

use super::{
    resolved_text_run_frame, text_paint_runs_from_resolved_layout, text_paint_runs_from_shaped,
    text_run_frame, UiShapedText, UiShapedTextCluster, UiShapedTextLine, UiTextPaintRun,
    UiTextRunPaintStyle,
};

const LINE_COUNT: usize = 128;
const RUN_COUNT: usize = 128;
const ITERATION_COUNT: usize = 2_048;
const GEOMETRY_ITERATION_COUNT: usize = 512;
const SAMPLE_COUNT: usize = 11;

fn shaped_text() -> UiShapedText {
    let lines = (0..LINE_COUNT)
        .map(|line_index| UiShapedTextLine {
            text: "x".to_string(),
            frame: UiFrame::new(0.0, line_index as f32, 1.0, 1.0),
            source_range: UiTextRange {
                start: line_index,
                end: line_index + 1,
            },
            visual_range: UiTextRange { start: 0, end: 1 },
            measured_width: 1.0,
            baseline: 1.0,
            direction: UiTextDirection::LeftToRight,
            ellipsized: false,
            glyphs: Vec::new(),
            clusters: vec![UiShapedTextCluster {
                kind: UiTextRunKind::Plain,
                text: "x".to_string(),
                source_range: UiTextRange {
                    start: line_index,
                    end: line_index + 1,
                },
                visual_range: UiTextRange { start: 0, end: 1 },
                direction: UiTextDirection::LeftToRight,
            }],
        })
        .collect();
    UiShapedText {
        source_text: "x".repeat(LINE_COUNT),
        source_range: UiTextRange {
            start: 0,
            end: LINE_COUNT,
        },
        direction: UiTextDirection::LeftToRight,
        overflow: UiTextOverflow::Clip,
        font_size: 12.0,
        line_height: 14.0,
        measured_width: 1.0,
        measured_height: LINE_COUNT as f32,
        writing_mode: UiTextWritingMode::HorizontalTb,
        render_mode: UiTextRenderMode::Native,
        font_key: None,
        atlas_resource: None,
        ellipsis_range: None,
        lines,
    }
}

fn resolved_layout() -> UiResolvedTextLayout {
    let lines = (0..LINE_COUNT)
        .map(|line_index| UiResolvedTextLine {
            text: "x".to_string(),
            frame: UiFrame::new(0.0, line_index as f32, 1.0, 1.0),
            placement_frame: UiFrame::new(0.0, line_index as f32, 1.0, 1.0),
            source_range: UiTextRange {
                start: line_index,
                end: line_index + 1,
            },
            visual_range: UiTextRange { start: 0, end: 1 },
            measured_width: 1.0,
            glyph_advances: vec![1.0],
            baseline: 1.0,
            direction: UiTextDirection::LeftToRight,
            runs: vec![UiResolvedTextRun {
                kind: UiTextRunKind::Plain,
                text: "x".to_string(),
                source_range: UiTextRange {
                    start: line_index,
                    end: line_index + 1,
                },
                visual_range: UiTextRange { start: 0, end: 1 },
                direction: UiTextDirection::LeftToRight,
            }],
            ellipsized: false,
        })
        .collect();
    UiResolvedTextLayout {
        font_size: 12.0,
        line_height: 14.0,
        lines,
        ..UiResolvedTextLayout::default()
    }
}

fn shaped_multi_run_text() -> UiShapedText {
    let text = "x".repeat(RUN_COUNT);
    let clusters = (0..RUN_COUNT)
        .map(|index| UiShapedTextCluster {
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
    let glyphs = (0..RUN_COUNT)
        .map(|index| {
            UiShapedGlyph::new(
                index as u32,
                UiTextRange {
                    start: index,
                    end: index + 1,
                },
                UiFrame::new(index as f32, 0.0, 1.0, 1.0),
                1.0,
            )
        })
        .collect();
    UiShapedText {
        source_text: text.clone(),
        source_range: UiTextRange {
            start: 0,
            end: RUN_COUNT,
        },
        direction: UiTextDirection::LeftToRight,
        overflow: UiTextOverflow::Clip,
        font_size: 12.0,
        line_height: 14.0,
        measured_width: RUN_COUNT as f32,
        measured_height: 1.0,
        writing_mode: UiTextWritingMode::HorizontalTb,
        render_mode: UiTextRenderMode::Native,
        font_key: None,
        atlas_resource: None,
        ellipsis_range: None,
        lines: vec![UiShapedTextLine {
            text,
            frame: UiFrame::new(0.0, 0.0, RUN_COUNT as f32, 1.0),
            source_range: UiTextRange {
                start: 0,
                end: RUN_COUNT,
            },
            visual_range: UiTextRange {
                start: 0,
                end: RUN_COUNT,
            },
            measured_width: RUN_COUNT as f32,
            baseline: 1.0,
            direction: UiTextDirection::LeftToRight,
            ellipsized: false,
            glyphs,
            clusters,
        }],
    }
}

fn resolved_multi_run_layout() -> UiResolvedTextLayout {
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
    UiResolvedTextLayout {
        font_size: 12.0,
        line_height: 14.0,
        measured_width: RUN_COUNT as f32,
        measured_height: 1.0,
        source_range: UiTextRange {
            start: 0,
            end: RUN_COUNT,
        },
        lines: vec![UiResolvedTextLine {
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
        }],
        ..UiResolvedTextLayout::default()
    }
}

fn unreserved_shaped_runs(shaped: &UiShapedText) -> Vec<UiTextPaintRun> {
    let mut runs = Vec::new();
    for line in &shaped.lines {
        for cluster in &line.clusters {
            if cluster.text.is_empty() {
                continue;
            }
            runs.push(UiTextPaintRun {
                kind: cluster.kind,
                text: cluster.text.clone(),
                source_range: cluster.source_range,
                visual_range: cluster.visual_range,
                frame: text_run_frame(shaped.writing_mode, line, cluster.visual_range),
                color: None,
                font: None,
                font_family: None,
                font_weight: 400,
                font_size: shaped.font_size,
                line_height: shaped.line_height,
                style: UiTextRunPaintStyle::from_run_kind(cluster.kind),
            });
        }
    }
    runs
}

fn unreserved_resolved_runs(layout: &UiResolvedTextLayout) -> Vec<UiTextPaintRun> {
    let mut runs = Vec::new();
    for line in &layout.lines {
        let mut expected_visual_start = line.visual_range.start;
        let mut has_nonempty_run = false;
        for run in &line.runs {
            if run.text.is_empty() {
                continue;
            }
            if run.visual_range.start != expected_visual_start
                || line.text.get(run.visual_range.start..run.visual_range.end)
                    != Some(run.text.as_str())
            {
                return Vec::new();
            }
            has_nonempty_run = true;
            expected_visual_start = run.visual_range.end;
            let Some(frame) = resolved_text_run_frame(layout.writing_mode, line, run.visual_range)
            else {
                return Vec::new();
            };
            runs.push(UiTextPaintRun {
                kind: run.kind,
                text: run.text.clone(),
                source_range: run.source_range,
                visual_range: run.visual_range,
                frame,
                color: None,
                font: None,
                font_family: None,
                font_weight: 400,
                font_size: layout.font_size,
                line_height: layout.line_height,
                style: UiTextRunPaintStyle::from_run_kind(run.kind),
            });
        }
        if has_nonempty_run && expected_visual_start != line.visual_range.end {
            return Vec::new();
        }
    }
    runs
}

fn legacy_shaped_geometry_runs(shaped: &UiShapedText) -> Vec<UiTextPaintRun> {
    let run_capacity = shaped.lines.iter().map(|line| line.clusters.len()).sum();
    let mut runs = Vec::with_capacity(run_capacity);
    for line in &shaped.lines {
        for cluster in &line.clusters {
            if cluster.text.is_empty() {
                continue;
            }
            runs.push(UiTextPaintRun {
                kind: cluster.kind,
                text: cluster.text.clone(),
                source_range: cluster.source_range,
                visual_range: cluster.visual_range,
                frame: text_run_frame(shaped.writing_mode, line, cluster.visual_range),
                color: None,
                font: None,
                font_family: None,
                font_weight: 400,
                font_size: shaped.font_size,
                line_height: shaped.line_height,
                style: UiTextRunPaintStyle::from_run_kind(cluster.kind),
            });
        }
    }
    runs
}

fn legacy_resolved_geometry_runs(layout: &UiResolvedTextLayout) -> Vec<UiTextPaintRun> {
    let run_capacity = layout.lines.iter().map(|line| line.runs.len()).sum();
    let mut runs = Vec::with_capacity(run_capacity);
    for line in &layout.lines {
        let mut expected_visual_start = line.visual_range.start;
        let mut has_nonempty_run = false;
        for run in &line.runs {
            if run.text.is_empty() {
                continue;
            }
            if run.visual_range.start != expected_visual_start
                || line.text.get(run.visual_range.start..run.visual_range.end)
                    != Some(run.text.as_str())
            {
                return Vec::new();
            }
            has_nonempty_run = true;
            expected_visual_start = run.visual_range.end;
            let Some(frame) = resolved_text_run_frame(layout.writing_mode, line, run.visual_range)
            else {
                return Vec::new();
            };
            runs.push(UiTextPaintRun {
                kind: run.kind,
                text: run.text.clone(),
                source_range: run.source_range,
                visual_range: run.visual_range,
                frame,
                color: None,
                font: None,
                font_family: None,
                font_weight: 400,
                font_size: layout.font_size,
                line_height: layout.line_height,
                style: UiTextRunPaintStyle::from_run_kind(run.kind),
            });
        }
        if has_nonempty_run && expected_visual_start != line.visual_range.end {
            return Vec::new();
        }
    }
    runs
}

fn p95(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[SAMPLE_COUNT - 1]
}

#[test]
fn runtime_interface03_batch9_presized_text_run_projections_preserve_output() {
    let shaped = shaped_text();
    let layout = resolved_layout();
    let none = None;

    assert_eq!(
        text_paint_runs_from_shaped(&shaped, &none, &none, &none, 400, 12.0, 14.0),
        unreserved_shaped_runs(&shaped),
    );
    assert_eq!(
        text_paint_runs_from_resolved_layout(&layout, &none, &none, &none, 400, 12.0, 14.0,),
        unreserved_resolved_runs(&layout),
    );
}

#[test]
fn runtime_interface03_batch10_indexed_text_run_geometry_preserves_output() {
    let shaped = shaped_multi_run_text();
    let layout = resolved_multi_run_layout();
    let none = None;

    assert_eq!(
        text_paint_runs_from_shaped(&shaped, &none, &none, &none, 400, 12.0, 14.0),
        legacy_shaped_geometry_runs(&shaped),
    );
    assert_eq!(
        text_paint_runs_from_resolved_layout(&layout, &none, &none, &none, 400, 12.0, 14.0,),
        legacy_resolved_geometry_runs(&layout),
    );
}

#[test]
#[ignore = "release-only indexed shaped-text run-geometry benchmark"]
fn runtime_interface03_batch10_indexed_shaped_text_run_geometry_release_benchmark() {
    let shaped = shaped_multi_run_text();
    let none = None;
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..GEOMETRY_ITERATION_COUNT {
                black_box(legacy_shaped_geometry_runs(black_box(&shaped)));
            }
            started.elapsed().as_nanos()
        };
        let measure_indexed = || {
            let started = Instant::now();
            for _ in 0..GEOMETRY_ITERATION_COUNT {
                black_box(text_paint_runs_from_shaped(
                    black_box(&shaped),
                    &none,
                    &none,
                    &none,
                    400,
                    12.0,
                    14.0,
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy());
            indexed_samples.push(measure_indexed());
        } else {
            indexed_samples.push(measure_indexed());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95 = p95(legacy_samples);
    let indexed_p95 = p95(indexed_samples);
    eprintln!(
        "RUNTIME_INTERFACE03_INDEXED_SHAPED_TEXT_RUN_GEOMETRY_BENCH_V1 runs={RUN_COUNT} iterations={GEOMETRY_ITERATION_COUNT} samples={SAMPLE_COUNT} legacy_p95_ns={legacy_p95} indexed_p95_ns={indexed_p95}"
    );
    assert!(
        indexed_p95.saturating_mul(10) <= legacy_p95.saturating_mul(7),
        "indexed shaped geometry must improve P95 by at least 30%: legacy={legacy_p95}ns indexed={indexed_p95}ns",
    );
}

#[test]
#[ignore = "release-only indexed resolved-text run-geometry benchmark"]
fn runtime_interface03_batch10_indexed_resolved_text_run_geometry_release_benchmark() {
    let layout = resolved_multi_run_layout();
    let none = None;
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..GEOMETRY_ITERATION_COUNT {
                black_box(legacy_resolved_geometry_runs(black_box(&layout)));
            }
            started.elapsed().as_nanos()
        };
        let measure_indexed = || {
            let started = Instant::now();
            for _ in 0..GEOMETRY_ITERATION_COUNT {
                black_box(text_paint_runs_from_resolved_layout(
                    black_box(&layout),
                    &none,
                    &none,
                    &none,
                    400,
                    12.0,
                    14.0,
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy());
            indexed_samples.push(measure_indexed());
        } else {
            indexed_samples.push(measure_indexed());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95 = p95(legacy_samples);
    let indexed_p95 = p95(indexed_samples);
    eprintln!(
        "RUNTIME_INTERFACE03_INDEXED_RESOLVED_TEXT_RUN_GEOMETRY_BENCH_V1 runs={RUN_COUNT} iterations={GEOMETRY_ITERATION_COUNT} samples={SAMPLE_COUNT} legacy_p95_ns={legacy_p95} indexed_p95_ns={indexed_p95}"
    );
    assert!(
        indexed_p95.saturating_mul(10) <= legacy_p95.saturating_mul(7),
        "indexed resolved geometry must improve P95 by at least 30%: legacy={legacy_p95}ns indexed={indexed_p95}ns",
    );
}

#[test]
#[ignore = "release-only presized shaped-text paint-run benchmark"]
fn runtime_interface03_batch9_presized_shaped_text_runs_release_benchmark() {
    let shaped = shaped_text();
    let none = None;
    let mut unreserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut presized_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_unreserved = || {
            let started = Instant::now();
            for _ in 0..ITERATION_COUNT {
                black_box(unreserved_shaped_runs(black_box(&shaped)));
            }
            started.elapsed().as_nanos()
        };
        let measure_presized = || {
            let started = Instant::now();
            for _ in 0..ITERATION_COUNT {
                black_box(text_paint_runs_from_shaped(
                    black_box(&shaped),
                    &none,
                    &none,
                    &none,
                    400,
                    12.0,
                    14.0,
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            unreserved_samples.push(measure_unreserved());
            presized_samples.push(measure_presized());
        } else {
            presized_samples.push(measure_presized());
            unreserved_samples.push(measure_unreserved());
        }
    }

    let unreserved_p95 = p95(unreserved_samples);
    let presized_p95 = p95(presized_samples);
    eprintln!(
        "RUNTIME_INTERFACE03_PRESIZED_SHAPED_TEXT_RUNS_BENCH_V1 lines={LINE_COUNT} iterations={ITERATION_COUNT} samples={SAMPLE_COUNT} unreserved_p95_ns={unreserved_p95} presized_p95_ns={presized_p95}"
    );
    assert!(
        presized_p95.saturating_mul(20) <= unreserved_p95.saturating_mul(19),
        "presized shaped projection must improve P95 by at least 5%: unreserved={unreserved_p95}ns presized={presized_p95}ns",
    );
}

#[test]
#[ignore = "release-only presized resolved-text paint-run benchmark"]
fn runtime_interface03_batch9_presized_resolved_text_runs_release_benchmark() {
    let layout = resolved_layout();
    let none = None;
    let mut unreserved_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut presized_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_unreserved = || {
            let started = Instant::now();
            for _ in 0..ITERATION_COUNT {
                black_box(unreserved_resolved_runs(black_box(&layout)));
            }
            started.elapsed().as_nanos()
        };
        let measure_presized = || {
            let started = Instant::now();
            for _ in 0..ITERATION_COUNT {
                black_box(text_paint_runs_from_resolved_layout(
                    black_box(&layout),
                    &none,
                    &none,
                    &none,
                    400,
                    12.0,
                    14.0,
                ));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            unreserved_samples.push(measure_unreserved());
            presized_samples.push(measure_presized());
        } else {
            presized_samples.push(measure_presized());
            unreserved_samples.push(measure_unreserved());
        }
    }

    let unreserved_p95 = p95(unreserved_samples);
    let presized_p95 = p95(presized_samples);
    eprintln!(
        "RUNTIME_INTERFACE03_PRESIZED_RESOLVED_TEXT_RUNS_BENCH_V1 lines={LINE_COUNT} iterations={ITERATION_COUNT} samples={SAMPLE_COUNT} unreserved_p95_ns={unreserved_p95} presized_p95_ns={presized_p95}"
    );
    assert!(
        presized_p95.saturating_mul(20) <= unreserved_p95.saturating_mul(19),
        "presized resolved projection must improve P95 by at least 5%: unreserved={unreserved_p95}ns presized={presized_p95}ns",
    );
}
