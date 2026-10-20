//! 家族去重的固定输入与容量策略契约；断言范围不覆盖全部 Unicode 或真实字体性能。
//! 忽略基准比较整数容器的预留策略，运行证据需由指定环境另行取得。

use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

use super::{dedupe_families, dedupe_scoped_families, FontFamilyCandidateScope};
use crate::text::FontFamilyName;

// 成对样本数量，用于交替两种策略的测量顺序。
const SAMPLE_PAIRS: usize = 21;
// 每个样本中重复构造容器的次数。
const BUILDS_PER_SAMPLE: usize = 2_048;
// 每次构造插入的不同整数数量；并未构造或哈希 FontFamilyName。
const FAMILIES_PER_BUILD: usize = 256;

/// 固定 ASCII 名称夹具同时检查首项顺序、空名过滤与输入下界预留。
/// `" Inter "` 的输出去空白来自 `FontFamilyName::from`，不是去重函数改写首项。
#[test]
fn optimization_batch_20260826fd_runtime199_capacity_preserves_font_family_deduplication() {
    let families = vec![
        FontFamilyName::from(" Inter "),
        FontFamilyName::from("inter"),
        FontFamilyName::from(""),
        FontFamilyName::from("Noto Sans"),
        FontFamilyName::from("Fira Mono"),
        FontFamilyName::from("NOTO SANS"),
    ];

    let deduped = dedupe_families(families);

    assert_eq!(deduped.len(), 3);
    assert_eq!(deduped[0].as_str(), "Inter");
    assert_eq!(deduped[1].as_str(), "Noto Sans");
    assert_eq!(deduped[2].as_str(), "Fira Mono");
    // 六个输入的 Vec 迭代器下界是 6；只要求结果容量至少为此下界。
    assert!(deduped.capacity() >= 6);
}

/// 原文契约只检查四个精确子串；它不执行去重算法或验证分配次数。
#[test]
fn optimization_batch_20260826fd_runtime199_font_family_dedupe_reserves_iterator_lower_bound() {
    // 下方 contains 是词法锚点；注释中的同名文本也会参与匹配。
    let source = include_str!("../../matching.rs");
    assert!(source.contains("let families = families.into_iter();"));
    assert!(source.contains("let (capacity, _) = families.size_hint();"));
    assert!(source.contains("HashSet::with_capacity(capacity)"));
    assert!(source.contains("Vec::with_capacity(capacity)"));
}

/// 检查单独的本地标签，以及后续同名 ASCII 候选把本地标签升级为允许全局查找。
/// 该夹具只检查标签合并，不查询 owner 字体面或验证字符覆盖回退。
#[test]
fn scoped_font_family_dedupe_preserves_or_upgrades_external_fallback_authority() {
    let local_only = dedupe_scoped_families([(
        FontFamilyName::from("Owner Typeface"),
        FontFamilyCandidateScope::OwnerLocalOnly,
    )]);
    let explicitly_external = dedupe_scoped_families([
        (
            FontFamilyName::from("Owner Typeface"),
            FontFamilyCandidateScope::OwnerLocalOnly,
        ),
        (
            FontFamilyName::from("owner typeface"),
            FontFamilyCandidateScope::OwnerThenGlobal,
        ),
    ]);

    assert_eq!(local_only.len(), 1);
    assert_eq!(
        local_only[0].scope,
        FontFamilyCandidateScope::OwnerLocalOnly
    );
    assert_eq!(explicitly_external.len(), 1);
    assert_eq!(
        explicitly_external[0].scope,
        FontFamilyCandidateScope::OwnerThenGlobal
    );
}

/// 手动环境基准比较 HashSet/Vec 是否预留容量，元素是不同整数。
/// 不调用家族去重函数；P95 门槛只约束这组受管基准结果，不证明字体查询加速比例。
#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fd_runtime199_font_family_dedupe_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    // 每对交替先测的策略，分别取得一个无预留与有预留样本。
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    // 对各策略的整组样本分别取排序分位值，不按每对样本计算比例。
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME199_FONT_FAMILY_DEDUPE_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} families_per_build={FAMILIES_PER_BUILD} \
legacy_reservations_per_build=0 optimized_reservations_per_build=2 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    // 门槛使用饱和乘法作百分比比较；测量包含整段循环与容器处理。
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

/// 测量 `reserve` 指定的容器构造策略，返回本次完整循环的耗时纳秒，最小为 1。
/// true 使用固定容量预留，false 从空容器增长；计时包含构造、插入与销毁。
/// 两分支插入相同整数并通过 black_box 保留结果；不访问字体或改变生产查询状态。
fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    // 每轮重新创建两容器，避免把前一轮已有容量带入下一轮。
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut identities = if reserve {
            HashSet::with_capacity(FAMILIES_PER_BUILD)
        } else {
            HashSet::new()
        };
        let mut families = if reserve {
            Vec::with_capacity(FAMILIES_PER_BUILD)
        } else {
            Vec::new()
        };
        for family in 0..FAMILIES_PER_BUILD {
            if identities.insert(black_box(family)) {
                families.push(family);
            }
        }
        // 把长度与容量送入优化屏障；该校验和不作为功能正确性断言。
        checksum ^= black_box(identities.len() ^ families.len() ^ families.capacity());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

/// 借入 `samples` 并排序副本，返回 `percentile` 对应的向上取整秩样本，不改原切片。
/// 调用约束由基准保证：样本非空、计算出的秩落在切片内且长度乘百分位可表示。
/// 当前调用固定使用 21 项与 50/95；返回数值不保留输入借用。
fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

/// 借入 `samples`，按原顺序生成逗号分隔的十进制样本串；返回 String 拥有全部文本。
/// 仅供基准输出，空切片会生成空串，不保留输入借用。
fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
