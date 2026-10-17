//! 直接校验字符筛选与源码结构，并保留单独忽略的 ASCII 计时夹具。

use std::hint::black_box;
use std::time::Instant;

use super::allows_glyph_fallback;

// 性能夹具收集成对样本；此常量不代表本次审查已经计时。
const SAMPLE_PAIRS: usize = 17;

// 两例分别覆盖基本区和补充区选择器，不表示穷尽所有序列。
#[test]
fn rejects_glyph_fallback_for_variation_selector_sequences() {
    assert!(!allows_glyph_fallback("✈\u{fe0f}"));
    assert!(!allows_glyph_fallback("禰\u{e0100}"));
}

// 表驱动循环的同一断言检查四个指定连接字符。
#[test]
fn rejects_glyph_fallback_for_additional_glue_characters() {
    for text in ["a\u{2011}b", "a\u{202f}b", "a\u{2060}b", "a\u{feff}b"] {
        assert!(!allows_glyph_fallback(text), "{text:?} must be glue");
    }
}

// 只检查生产前缀的原始文本形状，不直接测量遍历次数或耗时。
#[test]
fn optimization_batch_dd_glue_scan_uses_ascii_fast_path_and_one_unicode_pass() {
    let source = include_str!("../glue.rs")
        .split_once("#[cfg(test)]")
        .expect("production source and tests must remain separated")
        .0;
    assert!(source.contains("if text.is_ascii()"));
    assert!(source.contains("return true;"));
    assert!(source.contains(".all(|character|"));
    assert!(!source.contains(".any(is_glue_character) && !text.chars().any"));
}

// 该忽略测试须另行运行；本次仅审读输入、采样顺序和比较阈值。
#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_dd_runtime410_glue_ascii_fast_path_p95() {
    // 长 ASCII 输入构造在计时外，先交替预热两条路径。
    const TEXT_LENGTH: usize = 65_536;
    let text = "a".repeat(TEXT_LENGTH);

    for _ in 0..3 {
        black_box(legacy_allows_glyph_fallback(black_box(&text)));
        black_box(allows_glyph_fallback(black_box(&text)));
    }

    // 每对交换执行先后；单样本仍包含闭包调用与计时开销。
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(|| legacy_allows_glyph_fallback(&text)));
            optimized.push(measure(|| allows_glyph_fallback(&text)));
        } else {
            optimized.push(measure(|| allows_glyph_fallback(&text)));
            legacy.push(measure(|| legacy_allows_glyph_fallback(&text)));
        }
    }

    // 各取第 95 百分位；17 个样本时当前索引公式选到最大值。
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME410_GLUE_ASCII_FAST_PATH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} text_length={TEXT_LENGTH} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

// 保留双遍历比较路径，仅供本文件的忽略计时夹具调用。
fn legacy_allows_glyph_fallback(text: &str) -> bool {
    !text.chars().any(super::is_glue_character) && !text.chars().any(super::is_variation_selector)
}

/// 消费一次闭包并计时，布尔结果仅交给优化屏障；耗时至少记为 1ns。
fn measure(run: impl FnOnce() -> bool) -> u128 {
    let started = Instant::now();
    black_box(run());
    started.elapsed().as_nanos().max(1)
}

/// 复制并排序非空样本，按向上取整的秩取值；本夹具只传入 95。
/// 调用方须保证秩落在样本范围，此处不另做参数校验。
fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

/// 按样本原顺序格式化为逗号分隔文本，供基准输出记录使用。
fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
