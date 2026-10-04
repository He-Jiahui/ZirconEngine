use std::{hint::black_box, time::Instant};

use crate::ui::{
    layout::UiFrame,
    surface::{
        UiResolvedTextLine, UiResolvedTextRun, UiTextCaret, UiTextCaretAffinity, UiTextDirection,
        UiTextRange, UiTextRunKind,
    },
};

use super::{UiTextLineSourceMap, SOURCE_EDGE_LINEAR_QUERY_LIMIT};

const CLUSTER_COUNT: usize = 65_536;
const LOOKUP_COUNT: usize = 64;
const SAMPLE_COUNT: usize = 11;

fn dense_line() -> UiResolvedTextLine {
    let text = "x".repeat(CLUSTER_COUNT);
    let runs = (0..CLUSTER_COUNT)
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
        frame: UiFrame::new(0.0, 0.0, CLUSTER_COUNT as f32, 1.0),
        placement_frame: UiFrame::new(0.0, 0.0, CLUSTER_COUNT as f32, 1.0),
        source_range: UiTextRange {
            start: 0,
            end: CLUSTER_COUNT,
        },
        visual_range: UiTextRange {
            start: 0,
            end: CLUSTER_COUNT,
        },
        measured_width: CLUSTER_COUNT as f32,
        glyph_advances: vec![1.0; CLUSTER_COUNT],
        baseline: 1.0,
        direction: UiTextDirection::LeftToRight,
        runs,
        ellipsized: false,
    }
}

fn repeated_source_line() -> UiResolvedTextLine {
    let mut line = dense_line();
    for run in &mut line.runs {
        run.source_range = UiTextRange {
            start: 0,
            end: CLUSTER_COUNT,
        };
    }
    line
}

#[test]
fn runtime_interface03_batch14_binary_caret_source_lookup_preserves_affinity() {
    let line = dense_line();
    let map = UiTextLineSourceMap::new(&line);
    for offset in [0, 1, CLUSTER_COUNT / 2, CLUSTER_COUNT - 1, CLUSTER_COUNT] {
        for affinity in [
            UiTextCaretAffinity::Upstream,
            UiTextCaretAffinity::Downstream,
        ] {
            let caret = UiTextCaret { offset, affinity };
            assert_eq!(
                map.visual_offset_for_caret(&caret),
                map.visual_offset_for_caret_linear(&caret),
                "offset={offset} affinity={affinity:?}",
            );
        }
    }
}

#[test]
fn runtime_interface03_batch15_binary_source_span_range_preserves_projection() {
    let line = dense_line();
    let map = UiTextLineSourceMap::new(&line);
    for range in [
        UiTextRange { start: 0, end: 0 },
        UiTextRange { start: 0, end: 1 },
        UiTextRange {
            start: CLUSTER_COUNT / 2,
            end: CLUSTER_COUNT / 2 + 8,
        },
        UiTextRange {
            start: CLUSTER_COUNT - 1,
            end: CLUSTER_COUNT,
        },
        UiTextRange {
            start: CLUSTER_COUNT,
            end: CLUSTER_COUNT + 1,
        },
    ] {
        assert_eq!(
            map.visual_spans_for_source_range(range),
            map.visual_spans_for_source_range_linear(range),
            "range={range:?}",
        );
    }
}

#[test]
fn runtime_interface03_batch16_cached_source_edges_preserve_projection() {
    let line = repeated_source_line();
    let map = UiTextLineSourceMap::new(&line);
    let cluster = &map.clusters[CLUSTER_COUNT / 2];
    let expected = map.source_edge_visual_bounds_linear(cluster);

    assert_eq!(expected, (0, CLUSTER_COUNT));
    for _ in 0..8 {
        assert_eq!(map.source_edge_visual_bounds(cluster), expected);
    }
    assert!(map
        .source_edge_cache
        .as_ref()
        .and_then(|cache| cache.bounds.get())
        .is_some());
}

#[test]
#[ignore = "release-only repeated nonisomorphic source-edge cache benchmark"]
fn runtime_interface03_batch16_cached_source_edges_release_benchmark() {
    let line = repeated_source_line();
    let map = UiTextLineSourceMap::new(&line);
    let cluster = &map.clusters[CLUSTER_COUNT / 2];
    for _ in 0..=SOURCE_EDGE_LINEAR_QUERY_LIMIT {
        black_box(map.source_edge_visual_bounds(black_box(cluster)));
    }
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut cached_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(map.source_edge_visual_bounds_linear(black_box(cluster)));
            }
            started.elapsed().as_nanos()
        };
        let measure_cached = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(map.source_edge_visual_bounds(black_box(cluster)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            cached_samples.push(measure_cached());
        } else {
            cached_samples.push(measure_cached());
            linear_samples.push(measure_linear());
        }
    }

    linear_samples.sort_unstable();
    cached_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_SOURCE_EDGE_CACHE_BENCH_V1 clusters={CLUSTER_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} linear_p95_ns={} cached_p95_ns={}",
        linear_samples[p95],
        cached_samples[p95],
    );
    assert!(
        cached_samples[p95].saturating_mul(10) <= linear_samples[p95],
        "cached source-edge bounds must improve P95 by at least 90%: linear={}ns cached={}ns",
        linear_samples[p95],
        cached_samples[p95],
    );
}

#[test]
#[ignore = "release-only binary source-span cluster-range benchmark"]
fn runtime_interface03_batch15_binary_source_span_range_release_benchmark() {
    let line = dense_line();
    let map = UiTextLineSourceMap::new(&line);
    let range = UiTextRange {
        start: CLUSTER_COUNT - 2,
        end: CLUSTER_COUNT - 1,
    };
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut binary_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(map.visual_spans_for_source_range_linear(black_box(range)));
            }
            started.elapsed().as_nanos()
        };
        let measure_binary = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(map.visual_spans_for_source_range(black_box(range)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            binary_samples.push(measure_binary());
        } else {
            binary_samples.push(measure_binary());
            linear_samples.push(measure_linear());
        }
    }

    linear_samples.sort_unstable();
    binary_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_BINARY_SOURCE_SPAN_RANGE_BENCH_V1 clusters={CLUSTER_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} linear_p95_ns={} binary_p95_ns={}",
        linear_samples[p95],
        binary_samples[p95],
    );
    assert!(
        binary_samples[p95].saturating_mul(10) <= linear_samples[p95],
        "binary source-span range must improve P95 by at least 90%: linear={}ns binary={}ns",
        linear_samples[p95],
        binary_samples[p95],
    );
}

#[test]
#[ignore = "release-only binary caret source-cluster lookup benchmark"]
fn runtime_interface03_batch14_binary_caret_source_lookup_release_benchmark() {
    let line = dense_line();
    let map = UiTextLineSourceMap::new(&line);
    let caret = UiTextCaret {
        offset: CLUSTER_COUNT - 2,
        affinity: UiTextCaretAffinity::Downstream,
    };
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut binary_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(map.visual_offset_for_caret_linear(black_box(&caret)));
            }
            started.elapsed().as_nanos()
        };
        let measure_binary = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(map.visual_offset_for_caret(black_box(&caret)));
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            linear_samples.push(measure_linear());
            binary_samples.push(measure_binary());
        } else {
            binary_samples.push(measure_binary());
            linear_samples.push(measure_linear());
        }
    }

    linear_samples.sort_unstable();
    binary_samples.sort_unstable();
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_BINARY_CARET_SOURCE_LOOKUP_BENCH_V1 clusters={CLUSTER_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} linear_p95_ns={} binary_p95_ns={}",
        linear_samples[p95],
        binary_samples[p95],
    );
    assert!(
        binary_samples[p95].saturating_mul(10) <= linear_samples[p95],
        "binary caret source lookup must improve P95 by at least 90%: linear={}ns binary={}ns",
        linear_samples[p95],
        binary_samples[p95],
    );
}
