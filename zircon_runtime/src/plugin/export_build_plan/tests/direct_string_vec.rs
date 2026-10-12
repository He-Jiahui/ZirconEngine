use std::hint::black_box;
use std::time::{Duration, Instant};

use super::string_vec_expr;

const SAMPLE_PAIRS: usize = 101;
const TEMPLATES_PER_SAMPLE: usize = 4_096;

// 生成的片段会被 plugin_selection_template 嵌入导出工程源码；样例锁定调试转义、空值和输入顺序。
#[test]
fn runtime878_export_plugin_list_preserves_debug_escaping_and_order() {
    for values in [
        vec![],
        vec![String::new()],
        vec!["simple".to_string()],
        vec!["a\"quoted".to_string(), "\nnewline".to_string()],
        vec!["embedded\0nul".to_string(), "中文🦀".to_string()],
    ] {
        assert_eq!(string_vec_expr(&values), legacy_expr(&values));
    }
}

#[test]
#[ignore = "release percentile evidence; run through the managed Windows validation lane"]
// 该夹具复用同一组输入比较旧的 format 拼接与当前 String 预留实现；百分位门槛只由受管 release 证据解释。
fn runtime878_export_plugin_list_direct_release_percentiles() {
    let values = (0..32)
        .map(|index| format!("plugin_{index:02}_with_a_long_name"))
        .collect::<Vec<_>>();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&values, legacy_expr));
            optimized.push(measure(&values, string_vec_expr));
        } else {
            optimized.push(measure(&values, string_vec_expr));
            legacy.push(measure(&values, legacy_expr));
        }
    }
    let legacy_p50_ns = percentile(&mut legacy.clone(), 50);
    let legacy_p95_ns = percentile(&mut legacy.clone(), 95);
    let legacy_p99_ns = percentile(&mut legacy, 99);
    let optimized_p50_ns = percentile(&mut optimized.clone(), 50);
    let optimized_p95_ns = percentile(&mut optimized.clone(), 95);
    let optimized_p99_ns = percentile(&mut optimized, 99);
    println!(
        "RUNTIME878_EXPORT_PLUGIN_LIST_DIRECT_BENCH_V1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} optimized_p99_ns={optimized_p99_ns}"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns.saturating_mul(110).div_ceil(100));
}

// 旧实现作为输出基线保留，不能改成与当前实现相同的构造路径。
fn legacy_expr(values: &[String]) -> String {
    values
        .iter()
        .map(|value| format!("{value:?}.to_string()"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn measure(values: &[String], transform: fn(&[String]) -> String) -> Duration {
    let started = Instant::now();
    let checksum = (0..TEMPLATES_PER_SAMPLE)
        .map(|_| black_box(transform(black_box(values))).len())
        .sum::<usize>();
    black_box(checksum);
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percent: usize) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * percent).div_ceil(100).saturating_sub(1)].as_nanos()
}
