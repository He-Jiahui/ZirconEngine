//! 固定密集单行夹具的视觉簇投影对照与计时。
//!
//! 普通测试以线性实现对照顺序及逆序 run；带忽略标记的基准需显式选择执行。
//! “release-only” 是该基准的预期运行方式，忽略属性本身不检查编译配置。

use std::{hint::black_box, time::Instant};

use crate::ui::{
    layout::UiFrame,
    surface::{UiResolvedTextLine, UiResolvedTextRun, UiTextDirection, UiTextRange, UiTextRunKind},
};

use super::{visual_source_clusters, visual_source_clusters_linear};

const RUN_COUNT: usize = 2_048;
const ITERATION_COUNT: usize = 4;
const SAMPLE_COUNT: usize = 11;

/// 构造每个 ASCII 字节独占一个 run 的左到右行，源与视觉范围一一对应。
///
/// # 返回
/// 返回独立拥有文本、run 和等宽 advance 的合成夹具，供对照与计时复用。
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

/// 对照同一 ASCII 夹具的顺序与逆序 run，检查两条路径返回的完整簇序列相等。
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

/// 测量每次完整生成簇序列的成本；夹具构造在计时外，每个样本包含多次完整投影。
///
/// 每组测量交替两路径先后顺序，比较排序后的最大样本。
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
    // 排序后固定取最后一项，阈值比较的是本轮观测到的最大耗时。
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
