//! 覆盖规范化的值结果、源码形态和显式忽略的计时夹具。
//! 普通测试不提供大输入性能结论；计时夹具需要被显式选择才运行。

use std::hint::black_box;
use std::time::Instant;

use super::{normalize_codepoint_values, FontCoverage};

/// 仅供打印结果定位的静态字符串，不能作为本次已运行的证据。
const MARKER: &str = "RUNTIME245_FONT_COVERAGE_HASH_DEDUP_BENCH_V1";
/// 两条路径各采集的样本数；交替先后次序以减少固定顺序影响。
const SAMPLE_PAIRS: usize = 17;
/// 每个纳秒样本内重复调用实现的次数，包含结果构造与释放。
const REPEATS: usize = 512;

/// 用少量乱序重复值核对规范化后的闭区间结果。
/// 本夹具低于分流阈值，覆盖直接排序路径。
#[test]
fn optimization_batch_20260826gy_runtime245_font_coverage_stays_unique_and_compact() {
    assert_eq!(
        FontCoverage::from_codepoint_values(vec![40, 33, 32, 33, 34, 35, 40]),
        FontCoverage::Known(vec![(32, 35), (40, 40)])
    );
}

/// 在实现原文中检查散列去重路径的四个锚点，不调用较大输入路径。
/// 选择器和包含判词都读未遮罩文本，新增注释不能代替实现满足条件。
#[test]
fn optimization_batch_20260826gy_runtime245_font_coverage_dedups_large_inputs_before_sorting() {
    let source = include_str!("../../coverage.rs");
    let implementation = source
        .split("fn normalize_codepoint_values")
        .nth(1)
        .and_then(|tail| tail.split("#[cfg(test)]").next())
        .expect("font coverage normalization implementation");
    assert!(implementation.contains("HASH_DEDUP_CODEPOINT_THRESHOLD"));
    assert!(implementation.contains("HashSet::with_capacity"));
    assert!(implementation.contains("unique.insert(codepoint)"));
    assert!(implementation.contains("unique.into_iter().collect::<Vec<_>>()"));
}

/// 对重复密集输入交替采样两种实现，并比较本批纳秒样本的 p95。
/// 4096 项只有 16 个不同值；该形态不能代表所有字体，忽略属性也不等于性能证据。
#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gy_runtime245_font_coverage_hash_dedup_bench() {
    let codepoints = (0..4_096)
        .map(|index| 0x20 + (index % 16) as u32)
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&codepoints, legacy_font_coverage));
            optimized_samples.push(measure(&codepoints, optimized_font_coverage));
        } else {
            optimized_samples.push(measure(&codepoints, optimized_font_coverage));
            legacy_samples.push(measure(&codepoints, legacy_font_coverage));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "hash deduplication must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

/// 对照路径：借用 `codepoints`，复制后先排序去重，再压缩为自有覆盖值。
fn legacy_font_coverage(codepoints: &[u32]) -> FontCoverage {
    let mut codepoints = codepoints.to_vec();
    codepoints.sort_unstable();
    codepoints.dedup();
    FontCoverage::from_sorted_unique_codepoints(codepoints)
}

/// 待比较路径：借用 `codepoints`，复制后交给规范化 helper，再压缩为自有覆盖值。
fn optimized_font_coverage(codepoints: &[u32]) -> FontCoverage {
    FontCoverage::from_sorted_unique_codepoints(normalize_codepoint_values(codepoints.to_vec()))
}

/// 对同一借用输入重复调用指定实现，返回总耗时，单位为纳秒。
///
/// # 参数
/// - `codepoints`：调用期间只读借用的整数切片，具体实现各自创建工作副本。
/// - `implementation`：Rust 函数指针，实际传入上述两条路径；不是 FFI 或注册回调。
///
/// # 返回与约束
/// 返回 u64，超过上限的纳秒值饱和到最大值；计时含复制、规范化、压缩与观察。
/// 每轮 coverage 为自有值，轮末释放；校验和只抑制工作消除，不验证所有范围内容。
fn measure(codepoints: &[u32], implementation: fn(&[u32]) -> FontCoverage) -> u64 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..REPEATS {
        let coverage = implementation(black_box(codepoints));
        checksum = checksum.wrapping_add(match &coverage {
            FontCoverage::Known(ranges) => ranges.len(),
            FontCoverage::Unknown => 0,
        });
        black_box(&coverage);
    }
    black_box(checksum);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

/// 原地排序非空纳秒样本，返回向上取整的 95% 秩位置值。
/// `samples` 在调用期间独占借用，返回的是复制值；两处实际调用均传入 17 项。
fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
