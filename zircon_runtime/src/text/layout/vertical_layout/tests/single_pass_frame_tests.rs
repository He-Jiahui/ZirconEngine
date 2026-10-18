use std::hint::black_box;
use std::time::Instant;

use super::{finite_coordinate, finite_non_negative, finite_positive, layout_vertical_rl_columns};

const CHECKS_PER_SAMPLE: usize = 128;
const COLUMN_COUNT: usize = 16_384;
const SAMPLE_PAIRS: usize = 31;

// 双遍参照只用于下列常规范围夹具；直接 f32 运算不覆盖生产路径的溢出恢复。
fn legacy_layout(
    frame_x: f32,
    frame_y: f32,
    frame_width: f32,
    column_width: f32,
    column_advance: f32,
    column_heights: &[f32],
) -> super::VerticalColumnLayout {
    let column_width = finite_non_negative(column_width);
    let column_advance = finite_positive(column_advance).unwrap_or(column_width.max(1.0));
    let frame_width = finite_non_negative(frame_width);
    let column_capacity = (frame_width.max(column_advance) / column_advance)
        .floor()
        .max(1.0) as usize;
    let frame_right = finite_coordinate(frame_x) + frame_width;
    let frame_y = finite_coordinate(frame_y);
    let frames = column_heights
        .iter()
        .enumerate()
        .map(|(index, height)| super::VerticalColumnFrame {
            x: frame_right - (index + 1) as f32 * column_advance,
            y: frame_y,
            width: column_width,
            height: finite_non_negative(*height),
        })
        .collect::<Vec<_>>();
    let measured_height = frames
        .iter()
        .map(|frame| frame.height)
        .fold(0.0_f32, f32::max);

    super::VerticalColumnLayout {
        column_capacity,
        measured_width: frames.len() as f32 * column_advance,
        measured_height,
        frames,
    }
}

// 返回整批检查的纳秒数；计时包括建框、结果消费与析构、分支及循环开销。
// 固定高度夹具的构造和成对样本安排在调用此函数之外。
fn measure(column_heights: &[f32], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut evidence = 0.0_f32;
    for _ in 0..CHECKS_PER_SAMPLE {
        let layout = if optimized {
            layout_vertical_rl_columns(8.0, 4.0, 524_288.0, 20.0, 24.0, black_box(column_heights))
        } else {
            legacy_layout(8.0, 4.0, 524_288.0, 20.0, 24.0, black_box(column_heights))
        };
        evidence += layout.measured_height + layout.frames.len() as f32;
        black_box(layout);
    }
    black_box(evidence);
    started.elapsed().as_nanos().max(1)
}

// 本地调用固定使用非空的 31 份样本和 50/95 分位；不提供任意参数的边界校验。
fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

// 生成拥有的样本文本供计时结束后输出；分配与串接不在被测循环内。
fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

// 仅比较这些手工输入的结构结果，不能据此推断任意数值范围都与双遍参照等价。
#[test]
fn optimization_batch_20260829bt_runtime347_vertical_layout_preserves_results() {
    for heights in [
        Vec::new(),
        vec![50.0, 36.0, 72.0],
        vec![f32::NAN, -4.0, f32::INFINITY, 12.0],
    ] {
        let baseline = legacy_layout(f32::NAN, 4.0, 72.0, 20.0, 24.0, &heights);
        let candidate = layout_vertical_rl_columns(f32::NAN, 4.0, 72.0, 20.0, 24.0, &heights);
        assert_eq!(candidate, baseline, "{heights:?}");
    }
}

// 直接读取生产源码的指定前缀和函数后缀；这是文本结构断言，不是性能计数。
#[test]
fn optimization_batch_20260829bt_runtime347_vertical_layout_builds_frames_once() {
    let source = include_str!("../../vertical_layout.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    let function = production
        .split_once("fn layout_vertical_rl_columns")
        .expect("layout function")
        .1;
    assert!(function.contains("Vec::with_capacity(column_heights.len())"));
    assert!(function.contains("for (index, height) in column_heights.iter().copied().enumerate()"));
    assert!(function.contains("measured_height = measured_height.max(height)"));
    assert!(!function.contains("let measured_height = frames"));
}

// 极大值夹具核对有限发布及宽端点恢复后的两列位置。
#[test]
fn vertical_layout_keeps_geometry_finite_when_intermediates_overflow_f32() {
    let layout = layout_vertical_rl_columns(
        f32::MAX,
        f32::MAX,
        f32::MAX,
        f32::MAX,
        f32::MAX,
        &[f32::MAX, f32::MAX],
    );

    assert!(layout.measured_width.is_finite());
    assert!(layout.measured_height.is_finite());
    assert!(layout.frames.iter().all(|frame| {
        frame.x.is_finite()
            && frame.y.is_finite()
            && frame.width.is_finite()
            && frame.height.is_finite()
    }));
    assert_eq!(layout.frames[0].x, f32::MAX);
    assert_eq!(layout.frames[1].x, 0.0);
}

// 默认跳过的采样入口；忽略原因只是说明文字，不强制平台或编译配置。
// 打印的遍历次数是夹具常量，性能门槛只比较此次样本的高分位。
#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bt_runtime347_single_pass_vertical_frame_bench() {
    let column_heights = (0..COLUMN_COUNT)
        .map(|index| (index % 997) as f32)
        .collect::<Vec<_>>();
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&column_heights, false));
            candidate.push(measure(&column_heights, true));
        } else {
            candidate.push(measure(&column_heights, true));
            baseline.push(measure(&column_heights, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME347_SINGLE_PASS_VERTICAL_FRAME_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} column_count={COLUMN_COUNT} baseline_frame_passes=2 candidate_frame_passes=1 baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
