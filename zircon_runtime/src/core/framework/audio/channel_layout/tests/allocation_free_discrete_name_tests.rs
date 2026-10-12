//! 检查离散名称解析的边界行为，并保留受管理的 release 性能样本入口。
//!
//! 普通测试分别检查语义样例和源码结构；忽略的基准比较旧名称分配路径与字节解析路径。
//! 源码检查和静态阅读不提供性能验收，实际运行结果还受编译配置和测量环境约束。
use std::hint::black_box;
use std::time::Instant;

use super::{canonical_discrete_name_matches, AudioChannelLayout};

/// 基准为旧路径和优化路径各采集的样本数；31 个样本用于计算同一轮的中位及高分位结果。
///
/// 与 [`percentile`] 的非空输入前提关联；这个常量不表示跨进程或跨环境重复验证的次数。
const SAMPLE_PAIRS: usize = 31;
/// 每个计时样本执行的名称比较次数，供同一批两条路径采用相同工作量。
///
/// 计时包含循环、选择用例和校验和开销；采样结果是整批纳秒数，不是一次比较的延迟。
const CHECKS_PER_SAMPLE: usize = 200_000;

/// 用正数量构造与非法后缀样例约束离散名称优化的可观察语义。
///
/// # 使用时机与结果
///
/// 由测试框架调用，成功时返回单元值，断言失败时指出样例契约发生变化。
/// 覆盖正数量和 `u16` 上界、前导零、溢出、负号、尾随字符、数量不符与空后缀。
/// 这些有限样例不宣称穷举输入；私有解析器与完整布局的正数量门槛仍须分别理解。
///
/// # 参考
///
/// 参见 [`canonical_discrete_name_matches`] 与 [`AudioChannelLayout::is_canonical_discrete_layout`]。
#[test]
fn optimization_batch_20260829y_runtime298_discrete_name_check_preserves_contract() {
    for channel_count in [1, 16, u16::MAX] {
        assert!(AudioChannelLayout::discrete(channel_count).is_canonical_discrete_layout());
    }

    for (name, channel_count) in [
        ("discrete_01", 1),
        ("discrete_65536", u16::MAX),
        ("discrete_-1", 1),
        ("discrete_1x", 1),
        ("discrete_2", 1),
        ("discrete_", 1),
    ] {
        assert!(!canonical_discrete_name_matches(name, channel_count));
    }
}

/// 检查离散布局校验源码仍调用字节解析辅助函数，并避免恢复名称构造路径。
///
/// # 使用时机与约束
///
/// 测试框架读取编译时嵌入的源码，按函数标记截取区间后检查文本；函数名和区间是测试前提。
/// 它提供局部结构回归信号，不等同于运行时分配计数，也不证明所有校验路径没有分配。
///
/// # 返回值与参考
///
/// 返回单元值；标记缺失或断言不满足时失败。参见 [`AudioChannelLayout::is_canonical_discrete_layout`]。
#[test]
fn optimization_batch_20260829y_runtime298_discrete_name_check_avoids_formatting() {
    let source = include_str!("../../channel_layout.rs");
    let implementation = source.split("#[cfg(test)]").next().expect("implementation");
    let body = implementation
        .split("pub fn is_canonical_discrete_layout")
        .nth(1)
        .and_then(|body| body.split("pub fn is_valid_contract_layout").next())
        .expect("discrete layout validator");

    assert!(body.contains("canonical_discrete_name_matches"));
    assert!(!body.contains("format!("));
}

/// 在受管理的 Windows release 测量中比较旧名称构造与直接字节解析的同批性能。
///
/// # 使用时机与约束
///
/// 这是带忽略标记的显式基准入口，普通测试不自动执行；七个固定用例分别走两条路径。
/// 每轮收集 31 对各 200000 次比较的样本，交替先后次序减轻顺序偏差，但不消除环境噪声。
///
/// # 返回值与可观察输出
///
/// 返回单元值，输出原始样本、p50 和 p95；优化 p95 不超过旧路径 70% 是此入口的断言条件。
/// 输出中的每次分配数量是路径结构声明，没有在此安装分配器计数或测量任意外部调用。
///
/// # 生命周期与参考
///
/// 样本和格式化输出仅存活于本次测量，不修改产品状态；静态审查不代表基准验收。
/// 参见 [`measure`]、[`percentile`] 与 [`sample_csv`]。
#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829y_runtime298_allocation_free_discrete_channel_name_check_bench() {
    let cases = [
        ("discrete_1", 1),
        ("discrete_16", 16),
        ("discrete_65535", u16::MAX),
        ("discrete_01", 1),
        ("discrete_65536", u16::MAX),
        ("discrete_4095x", 4095),
        ("custom_layout", 12),
    ];
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    // 每对样本互换两条路径的计时先后，避免始终由同一路径承担首次运行的条件差异。
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false, &cases));
            optimized_samples.push(measure(true, &cases));
        } else {
            optimized_samples.push(measure(true, &cases));
            legacy_samples.push(measure(false, &cases));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    // 结果字符串在计时样本完成后生成；固定分配字段描述路径结构，不是分配器采样回执。
    println!(
        "RUNTIME298_ALLOCATION_FREE_DISCRETE_CHANNEL_NAME_CHECK_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} case_count={} \
legacy_result_allocations_per_check=1 optimized_result_allocations_per_check=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        cases.len(),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

/// 保留优化前构造名称再比较的行为，供基准在同一进程中取得对照样本。
///
/// # 参数与返回值
///
/// - `name`：待比较名称；`channel_count`：生成规范名称的数量，零也能生成名称。
/// - 返回输入是否等于生成的名称；临时字符串在本次调用结束前释放。
///
/// # 使用时机与参考
///
/// 只供测试对照，不参与产品布局校验；参见 [`measure`] 与 [`canonical_discrete_name_matches`]。
fn legacy_discrete_name_matches(name: &str, channel_count: u16) -> bool {
    name == format!("discrete_{channel_count}")
}

/// 对选定比较路径执行一批固定数量的调用，取得包含循环开销的计时样本。
///
/// # 参数
///
/// - `optimized`：为真时使用字节解析，否则使用旧名称构造；不改变用例列表。
/// - `cases`：借用的名称与数量列表，必须非空；调用入口提供七个固定用例。
///
/// # 返回值与约束
///
/// 返回整批经过时间的纳秒数，最小为一，以避免后续比例判断使用零样本。
/// 输入选择、比较结果和校验和经过黑盒消费，减少编译器移除测量工作；不保证消除一切优化影响。
///
/// # 生命周期与参考
///
/// 用例只在本次调用借用，局部校验和不输出产品状态；参见 [`CHECKS_PER_SAMPLE`] 与基准入口。
fn measure(optimized: bool, cases: &[(&str, u16)]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for index in 0..CHECKS_PER_SAMPLE {
        let (name, channel_count) = black_box(cases[index % cases.len()]);
        let matches = if optimized {
            canonical_discrete_name_matches(name, channel_count)
        } else {
            legacy_discrete_name_matches(name, channel_count)
        };
        checksum = checksum.wrapping_add(usize::from(black_box(matches)));
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

/// 从一轮计时样本计算向上取整秩的分位值，供基准报告 p50 与 p95。
///
/// # 参数
///
/// - `samples`：必须非空的整批纳秒样本；调用入口提供 31 个样本。
/// - `percentile`：本入口使用 50 或 95；辅助函数未验证范围，调用方须保证算出的索引可取。
///
/// # 返回值与生命周期
///
/// 复制并排序样本，返回对应秩的值，原切片不被修改；零分位会取首项，不做插值。
/// 空输入或超出范围的秩可能因索引失败而 panic；副本在调用结束时释放。
///
/// # 参考
///
/// 参见 [`SAMPLE_PAIRS`] 与基准入口的两个分位调用。
fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

/// 把原始计时样本按采集次序转为逗号分隔文本，供测量输出保留后续分析依据。
///
/// # 参数与返回值
///
/// - `samples`：借用的整批纳秒样本，允许为空；返回拥有的字符串，空输入得到空字符串。
///
/// # 使用时机与生命周期
///
/// 格式化分配发生在采样结束后的输出阶段，不计入 [`measure`] 的计时区间。
/// 结果独立拥有文本，原始顺序不变；排序仅由 [`percentile`] 的局部副本执行。
fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
