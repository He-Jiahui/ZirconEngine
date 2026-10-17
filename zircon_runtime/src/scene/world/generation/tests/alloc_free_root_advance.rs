//! 根绑定代数的行为、源码形态与迭代器适配微基准。
//!
//! 忽略基准测量下方两个探针；生产代数的实际结果由普通行为测试核对。

use std::hint::black_box;
use std::time::Instant;

use super::SceneBindingGenerations;

// 31 对交错采样，每个样本重复 20 万次；仅供显式启用的忽略基准使用。
const SAMPLE_PAIRS: usize = 31;
const ADVANCES_PER_SAMPLE: usize = 200_000;

/// 核对空输入不推进修订，并让替换后的指定根越过旧世界已发布的代数。
#[test]
fn optimization_batch_20260829ag_runtime306_root_advances_preserve_generations() {
    let mut generations = SceneBindingGenerations::default();
    generations.advance_roots([]);
    assert_eq!(generations.catalog_generation(), 0);

    generations.advance_roots([3, 5, 8]);
    assert_eq!(generations.catalog_generation(), 1);
    assert_eq!(generations.for_root(3), 1);
    assert_eq!(generations.for_root(5), 1);
    assert_eq!(generations.for_root(8), 1);

    let previous = generations.clone();
    let mut replacement = SceneBindingGenerations::default();
    replacement.advance_roots_after(&previous, [5, 13]);
    assert_eq!(replacement.catalog_generation(), 2);
    assert_eq!(replacement.for_root(5), 2);
    assert_eq!(replacement.for_root(13), 2);
}

/// 此夹具截取生产源码检查迭代器形态；它不执行被截取的函数体，也不测量分配次数。
#[test]
fn optimization_batch_20260829ag_runtime306_root_advances_use_peekable_iterators() {
    let source = include_str!("../../compiled_binding/generation.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let root_advance = implementation
        .split("pub(super) fn advance_roots<I>")
        .nth(1)
        .expect("root advance")
        .split("pub(super) fn advance_roots_after<I>")
        .next()
        .expect("root advance body");
    let replacement_advance = implementation
        .split("pub(super) fn advance_roots_after<I>")
        .nth(1)
        .expect("replacement root advance")
        .split("pub(super) fn intern_path")
        .next()
        .expect("replacement root advance body");

    for body in [root_advance, replacement_advance] {
        assert!(body.contains("roots.into_iter().peekable()"));
        assert!(body.contains("roots.peek().is_none()"));
        assert!(!body.contains("collect::<Vec<_>>"));
    }
}

/// 只比较两个迭代器探针的计时分布，按交错顺序采样后检查 P95 阈值。
/// 输出中的分配次数为固定模型标签，本函数没有接入分配计数器。
#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829ag_runtime306_allocation_free_scene_binding_root_advance_bench() {
    let roots = [3u64, 5, 8, 13, 21, 34, 55, 89];
    assert_eq!(optimized_probe(&roots), legacy_probe(&roots));
    assert_eq!(optimized_probe(&[]), legacy_probe(&[]));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &roots));
            optimized_samples.push(measure(true, &roots));
        } else {
            optimized_samples.push(measure(true, &roots));
            legacy_samples.push(measure(false, &roots));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME306_ALLOCATION_FREE_SCENE_BINDING_ROOT_ADVANCE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
advances_per_sample={ADVANCES_PER_SAMPLE} roots_per_advance={} \
legacy_adapter_allocations_per_advance=1 optimized_adapter_allocations_per_advance=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        roots.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn legacy_probe(roots: &[u64]) -> (bool, u64) {
    let roots = roots.iter().copied().collect::<Vec<_>>();
    if roots.is_empty() {
        return (false, 0);
    }
    (true, roots.into_iter().sum())
}

fn optimized_probe(roots: &[u64]) -> (bool, u64) {
    let mut roots = roots.iter().copied().peekable();
    if roots.peek().is_none() {
        return (false, 0);
    }
    (true, roots.sum())
}

/// 重复运行指定探针并用校验和保留计算；返回至少为一的纳秒计时，不计入样本数组维护。
fn measure(optimized: bool, roots: &[u64]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0u64;
    for _ in 0..ADVANCES_PER_SAMPLE {
        let (_, sum) = if optimized {
            optimized_probe(black_box(roots))
        } else {
            legacy_probe(black_box(roots))
        };
        checksum = checksum.wrapping_add(sum);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

/// 复制并排序非空样本，按上整百分位秩取值；本文件调用处只传 50 或 95。
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
