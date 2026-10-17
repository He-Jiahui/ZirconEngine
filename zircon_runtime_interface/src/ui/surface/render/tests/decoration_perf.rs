//! 对照范围候选行选择与无序行回退，使用合成的单字节水平行。
//! 显式忽略的计时用例不包含夹具构造，忽略属性不强制构建模式。
//! 线性分支收集结果向量，优化分支只返回下标范围，计时包含这一输出差别。

use std::{hint::black_box, time::Instant};

use crate::ui::{
    layout::UiFrame,
    surface::{UiResolvedTextLine, UiResolvedTextRun, UiTextDirection, UiTextRange, UiTextRunKind},
};

use super::{
    append_range_decorations_with_source_maps, intersecting_line_range,
    TextDecorationLineSourceMaps, TextRangeDecoration, UiResolvedTextLayout,
    UiTextPaintDecorationKind,
};

const LINE_COUNT: usize = 65_536;
const LOOKUP_COUNT: usize = 64;
const SAMPLE_COUNT: usize = 11;

/// 返回拥有文本与运行数据的单字节行；源范围按行号递增，视觉范围从行内零点开始。
fn line(index: usize) -> UiResolvedTextLine {
    UiResolvedTextLine {
        text: "x".to_string(),
        frame: UiFrame::new(0.0, index as f32, 1.0, 1.0),
        placement_frame: UiFrame::new(0.0, index as f32, 1.0, 1.0),
        source_range: UiTextRange {
            start: index,
            end: index + 1,
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
                start: index,
                end: index + 1,
            },
            visual_range: UiTextRange { start: 0, end: 1 },
            direction: UiTextDirection::LeftToRight,
        }],
        ellipsized: false,
    }
}

/// 逐行用相交谓词收集下标向量，作为本文件有限夹具的对照。
fn linear_intersections(lines: &[UiResolvedTextLine], range: UiTextRange) -> Vec<usize> {
    lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            (range.start < line.source_range.end && line.source_range.start < range.end)
                .then_some(index)
        })
        .collect()
}

/// 交换两行后，核选区仍生成一次、范围保留且只构造一个映射。
#[test]
fn unsorted_line_ranges_fall_back_without_losing_a_decoration() {
    let mut lines = (0..8).map(line).collect::<Vec<_>>();
    lines.swap(1, 6);
    let layout = UiResolvedTextLayout {
        lines,
        ..Default::default()
    };
    let range = UiTextRange { start: 6, end: 7 };
    let mut decorations = Vec::new();
    let mut source_maps = TextDecorationLineSourceMaps::new(&layout.lines);

    append_range_decorations_with_source_maps(
        &mut decorations,
        &layout,
        &[TextRangeDecoration::selection(range)],
        &mut source_maps,
    );

    assert_eq!(
        decorations
            .iter()
            .map(|decoration| decoration.kind)
            .collect::<Vec<_>>(),
        vec![UiTextPaintDecorationKind::Selection]
    );
    assert_eq!(decorations[0].range, range);
    assert_eq!(source_maps.initialized_count(), 1);
}

/// 用六个范围对照有序行的候选区间，包含空范围与行尾外位置。
#[test]
fn runtime_interface03_batch12_binary_decoration_line_range_preserves_intersections() {
    let lines = (0..8).map(line).collect::<Vec<_>>();
    for range in [
        UiTextRange { start: 0, end: 0 },
        UiTextRange { start: 0, end: 1 },
        UiTextRange { start: 2, end: 6 },
        UiTextRange { start: 7, end: 8 },
        UiTextRange { start: 8, end: 9 },
        UiTextRange { start: 10, end: 12 },
    ] {
        let binary = intersecting_line_range(&lines, range).collect::<Vec<_>>();
        assert_eq!(
            binary,
            linear_intersections(&lines, range),
            "range={range:?}"
        );
    }
}

/// 在行尾附近固定查询；每样本六十四次，交替两路径并排除夹具构造。
/// 此处比较完整调用成本，线性结果向量的构造也在计时内。
#[test]
#[ignore = "release-only binary decoration-line range benchmark"]
fn runtime_interface03_batch12_binary_decoration_line_range_release_benchmark() {
    let lines = (0..LINE_COUNT).map(line).collect::<Vec<_>>();
    let range = UiTextRange {
        start: LINE_COUNT - 2,
        end: LINE_COUNT - 1,
    };
    let mut linear_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut binary_samples = Vec::with_capacity(SAMPLE_COUNT);

    for sample in 0..SAMPLE_COUNT {
        let measure_linear = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(linear_intersections(black_box(&lines), black_box(range)));
            }
            started.elapsed().as_nanos()
        };
        let measure_binary = || {
            let started = Instant::now();
            for _ in 0..LOOKUP_COUNT {
                black_box(intersecting_line_range(black_box(&lines), black_box(range)));
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
    // 十一个样本排序后取末项，即这批样本的最大观测值。
    let p95 = SAMPLE_COUNT - 1;
    eprintln!(
        "RUNTIME_INTERFACE03_BINARY_DECORATION_LINE_RANGE_BENCH_V1 lines={LINE_COUNT} lookups={LOOKUP_COUNT} samples={SAMPLE_COUNT} linear_p95_ns={} binary_p95_ns={}",
        linear_samples[p95],
        binary_samples[p95],
    );
    assert!(
        binary_samples[p95].saturating_mul(10) <= linear_samples[p95],
        "binary decoration range must improve P95 by at least 90%: linear={}ns binary={}ns",
        linear_samples[p95],
        binary_samples[p95],
    );
}
