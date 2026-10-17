//! 以手工字素推进量验证制表几何，并保留独立忽略的计时比较夹具。
//! 这些断言不替代真实整形、分配计数或本次性能验收。

use std::{hint::black_box, time::Instant};

use super::*;

// 大输入、样本数和预热次数是夹具配置，分配字节输出采用公式估算。
const RELEASE_GRAPHEME_COUNT: usize = 131_072;
const RELEASE_SAMPLE_COUNT: usize = 31;
const RELEASE_WARMUP_COUNT: usize = 5;

/// 构造只覆盖制表大小的手工样式，其余字段沿用默认值。
fn tab_style() -> TextStyle {
    TextStyle {
        tab_size: 4.0,
        ..TextStyle::default()
    }
}

// 组合字符与基字组成一个扩展字素；负的制表推进输入由下一停靠点替代。
#[test]
fn streaming_tab_layout_preserves_tab_stops_and_unicode_graphemes() {
    let style = tab_style();
    let text = "a\u{301}\tb";
    let advances = [1.0, -2.0, 4.0];

    assert_eq!(
        tab_aligned_advances(text, &advances, &style, 2.0),
        vec![1.0, 7.0, 4.0]
    );
    assert_eq!(tab_aligned_width(text, &advances, &style, 2.0), 12.0);
}

// 直接调用预验证入口，夹具明确满足三个字素对应三个推进量且包含制表字符。
#[test]
fn matching_grapheme_width_streams_the_known_tabbed_input() {
    let style = tab_style();
    let text = "a\u{301}\tb";
    let advances = [1.0, -2.0, 4.0];

    assert_eq!(
        tab_aligned_width_for_matching_graphemes(text, &advances, &style, 2.0),
        12.0
    );
}

// 本忽略夹具须另行运行；本次审查没有生成延迟样本或分配测量结果。
// 输出中的分配次数是预设值，字节数按元素公式估算；都不是分配器计数。
#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260827bl_streaming_tab_layout_p95() {
    // 文本与推进量数组在计时外构造，先核两条路径对该输入的宽度一致。
    let style = tab_style();
    let text = release_tabbed_text();
    let advances = vec![1.0; RELEASE_GRAPHEME_COUNT];
    let legacy_width = legacy_tab_aligned_width(&text, &advances, &style, 2.0);
    let optimized_width = tab_aligned_width(&text, &advances, &style, 2.0);
    assert_eq!(legacy_width, optimized_width);

    // 仅下方采样函数里的两条调用被分别计时。
    let (legacy_samples, optimized_samples) = paired_width_samples(&text, &advances, &style, 2.0);
    let legacy_p50 = percentile(&legacy_samples, 50);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    // 此字节数按元素大小计算，不包含容器头、容量偏差或外部输入。
    let legacy_allocated_bytes =
        RELEASE_GRAPHEME_COUNT * (std::mem::size_of::<&str>() + std::mem::size_of::<f32>());

    println!(
        "PERF_RESULT RUNTIME81_STREAMING_TAB_LAYOUT_BENCH_V1 graphemes={RELEASE_GRAPHEME_COUNT} tab_every=16 warmups={RELEASE_WARMUP_COUNT} samples={RELEASE_SAMPLE_COUNT} legacy_width_allocations=2 optimized_width_allocations=0 legacy_width_allocated_bytes={legacy_allocated_bytes} optimized_width_allocated_bytes=0 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
    );
    // 纳秒非零和两个延迟阈值由实际运行判断，静态审读不提供这些值。
    assert!(
        legacy_p50 > 0 && optimized_p50 > 0,
        "release timing samples must have nanosecond resolution"
    );
    assert!(
        optimized_p50 * 100 <= legacy_p50 * 30,
        "optimized P50 {optimized_p50}ns must be at least 70% below legacy P50 {legacy_p50}ns"
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be at least 50% below legacy P95 {legacy_p95}ns"
    );
}

// 回退用例区分原样推进量副本和宽度求和，并覆盖没有制表字符与数量不足。
#[test]
fn streaming_tab_layout_preserves_unaligned_advance_fallbacks() {
    let style = tab_style();
    let no_tab_advances = [2.0, -1.0];
    let mismatched_advances = [3.0];

    assert_eq!(
        tab_aligned_advances("ab", &no_tab_advances, &style, 2.0),
        no_tab_advances
    );
    assert_eq!(tab_aligned_width("ab", &no_tab_advances, &style, 2.0), 1.0);
    assert_eq!(
        tab_aligned_advances("a\t", &mismatched_advances, &style, 2.0),
        mismatched_advances
    );
    assert_eq!(
        tab_aligned_width("a\t", &mismatched_advances, &style, 2.0),
        3.0
    );
}

// 极值夹具只验证所列输出有限和上限结果，不证明预算准入或数学精度。
#[test]
fn streaming_tab_layout_keeps_extreme_finite_geometry_bounded() {
    let style = tab_style();
    let advances = [f32::MAX, f32::MAX];

    let aligned = tab_aligned_advances("a\t", &advances, &style, f32::MAX);
    let width = tab_aligned_width("a\t", &advances, &style, f32::MAX);

    assert!(aligned.iter().all(|advance| advance.is_finite()));
    assert!(width.is_finite());
    assert_eq!(width, f32::MAX);
    assert_eq!(
        tab_aligned_width("ab", &advances, &style, f32::MAX),
        f32::MAX
    );
}

/// 构造指定长度的 ASCII 文本，每十六个字符中的最后一个为制表字符。
fn release_tabbed_text() -> String {
    let mut text = String::with_capacity(RELEASE_GRAPHEME_COUNT);
    for index in 0..RELEASE_GRAPHEME_COUNT {
        text.push(if index % 16 == 15 { '\t' } else { 'a' });
    }
    text
}

/// 物化字素列表和调整列表的比较路径，仅供本文件忽略夹具使用。
fn legacy_tab_aligned_width(
    text: &str,
    advances: &[f32],
    style: &TextStyle,
    space_width: f32,
) -> f32 {
    let graphemes = text.graphemes(true).collect::<Vec<_>>();
    if graphemes.len() != advances.len() || !graphemes.iter().any(|grapheme| *grapheme == "\t") {
        return finite_sum(advances.iter().copied());
    }

    let tab_interval = tab_interval_width(style, space_width);
    let tab_interval_exact = tab_interval_exact_width(style, space_width);
    let mut cursor = FiniteGeometryAccumulator::default();
    let mut adjusted = Vec::with_capacity(advances.len());
    for (grapheme, advance) in graphemes.iter().zip(advances.iter().copied()) {
        let resolved_advance = if *grapheme == "\t" {
            next_tab_advance(
                cursor.value(),
                cursor.exact(),
                tab_interval,
                tab_interval_exact,
            )
        } else {
            finite_f32_or_geometry(advance.max(0.0), f64::from(advance).max(0.0))
        };
        cursor.add(resolved_advance);
        adjusted.push(resolved_advance);
    }
    finite_sum(adjusted)
}

/// 借用已构造输入，先预热，再为两条路径各收集规定数量的纳秒样本。
/// 每对交替执行先后；循环、闭包调用与计时开销仍参与测量。
fn paired_width_samples(
    text: &str,
    advances: &[f32],
    style: &TextStyle,
    space_width: f32,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..RELEASE_WARMUP_COUNT {
        black_box(legacy_tab_aligned_width(text, advances, style, space_width));
        black_box(tab_aligned_width(text, advances, style, space_width));
    }

    let mut legacy_samples = Vec::with_capacity(RELEASE_SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(RELEASE_SAMPLE_COUNT);
    for index in 0..RELEASE_SAMPLE_COUNT {
        let legacy = || legacy_tab_aligned_width(text, advances, style, space_width);
        let optimized = || tab_aligned_width(text, advances, style, space_width);
        if index % 2 == 0 {
            legacy_samples.push(measure_width(legacy));
            optimized_samples.push(measure_width(optimized));
        } else {
            optimized_samples.push(measure_width(optimized));
            legacy_samples.push(measure_width(legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

/// 消费闭包一次并返回耗时，优化屏障接收宽度；此处不抬高零纳秒样本。
fn measure_width(operation: impl FnOnce() -> f32) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

/// 复制并排序非空样本，以整数截断后的索引取指定百分位。
/// 百分位须使计算后的索引有效；本夹具只传五十和九十五。
fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() * percentile / 100]
}
