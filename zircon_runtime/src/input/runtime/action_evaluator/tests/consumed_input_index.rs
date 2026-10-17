//! 核对消费输入下标的稳定重载和布局改变后的查询，并保留源码形状与计时样例。
//!
//! # 约束与生命周期
//! 功能 fixture 使用三个不同键，比较再次加载后的排列和交换轴布局后的重建结果；没有实际写入或排序计数器。
//! 源码守卫的首个相邻条件属性/模块分隔不符合当前路径属性布局。
//! 计时默认忽略，重载固定已准备样本，不检查实际 release 构建；结果只覆盖实际执行的固定重载样例。
//!
//! # 参考
//! 参见 [`ConsumedInputIndex`]、[`refresh_sorted_indices`] 与 [`measure_repeated_load`]。
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;
use crate::input::{GamepadAxis, GamepadId};

/// 为计时准备 4096 个固定轴身份；当前幂二长度配合奇数乘数和位掩码生成重排顺序。
///
/// # 生命周期
/// 仅测试编译期常量；不限制生产输入数量或动态控制索引行为。
///
/// # 参考
/// 参见 [`measure_repeated_load`]。
const BENCHMARK_INPUT_COUNT: usize = 4_096;
/// 每种实现收集 11 个耗时样本；当前 nearest-rank 95% 在此长度取最大值。
///
/// # 生命周期
/// 仅测试编译期常量；不限制生产输入数量或动态控制索引行为。
///
/// # 参考
/// 参见 [`percentile_95`]。
const BENCHMARK_SAMPLES: usize = 11;
/// 每个样本计时内重复加载同一不可变源切片 128 次；首次准备在计时外。
///
/// # 生命周期
/// 仅测试编译期常量；不限制生产输入数量或动态控制索引行为。
///
/// # 参考
/// 参见 [`measure_repeated_load`]。
const BENCHMARK_ITERATIONS: usize = 128;

/// 比较固定消费按钮、轴的成员查询以及稳定重载和布局交换后的下标结果。
///
/// # 参数
/// 无外部参数；fixture 提供三个不同按键码和固定轴身份，首次生成排序下标后再次加载，再交换轴源位置并重新加载。
///
/// # 返回
/// 返回 `()`；断言排序下标、存在/不存在查询以及交换后轴排列符合固定期望；失败使测试失败。
///
/// # 使用时机与生命周期
/// 源向量和下标副本为局部拥有；交换后先 load 再查询，保持当前切片与下标配对。
/// 相同排列断言不独立计数重建、排序或写入；此 fixture 不覆盖重复键、错误切片或损坏排列。
///
/// # 参考
/// 参见 [`ConsumedInputIndex::load`]、[`ConsumedInputIndex::button_is_consumed`]、[`ConsumedInputIndex::axis_is_consumed`]。
#[test]
fn runtime56_recovery_batch_reused_consumed_input_permutation_preserves_queries_and_rebuilds_when_needed(
) {
    let mut index = ConsumedInputIndex::default();
    let buttons = vec![
        InputButton::KeyCode(30),
        InputButton::KeyCode(10),
        InputButton::KeyCode(20),
    ];
    let mut axes = vec![axis_input(30), axis_input(10), axis_input(20)];

    index.load(&buttons, &axes);
    assert_eq!(index.button_indices, vec![1, 2, 0]);
    assert_eq!(index.axis_indices, vec![1, 2, 0]);
    assert!(index.button_is_consumed(&buttons, &InputButton::KeyCode(20)));
    assert!(index.axis_is_consumed(&axes, axis_input(20)));
    assert!(!index.button_is_consumed(&buttons, &InputButton::KeyCode(99)));
    assert!(!index.axis_is_consumed(&axes, axis_input(99)));

    let stable_button_indices = index.button_indices.clone();
    let stable_axis_indices = index.axis_indices.clone();
    index.load(&buttons, &axes);
    assert_eq!(index.button_indices, stable_button_indices);
    assert_eq!(index.axis_indices, stable_axis_indices);

    axes.swap(0, 1);
    index.load(&buttons, &axes);
    assert_eq!(index.axis_indices, vec![0, 2, 1]);
    assert!(index.axis_is_consumed(&axes, axis_input(30)));
}

/// 按生产源码原文检查共享排列助手及其提前返回的文本形状。
///
/// # 参数
/// 无外部参数；`include_str!` 相对本测试文件解析到生产消费索引。
/// 字符串中的单反斜线 n 解释为真实换行，首个提取要求条件属性后紧跟模块声明。
///
/// # 返回
/// 返回 `()`，但当前中间有路径属性，首个 `expect` 会失败；后续签名、次数和相对位置断言没有构成当前通过的测试。
///
/// # 使用时机与生命周期
/// 原始文本匹配可受注释影响；提前返回的位置比较不是编译器控制流分析。
/// 挂载布局的路径属性使首个查找为空；错误调用附近的 BUG 标签给出失败路径。
///
/// # 参考
/// 参见 [`refresh_sorted_indices`]、[`ConsumedInputIndex::load`]。
#[test]
fn runtime56_recovery_batch_reused_consumed_input_permutation_source_contract() {
    let source = include_str!("../consumed_input_index.rs");
    let production = crate::tests::runtime_absorption::structure_convention::rust_source_view::production_section(source);
    let loader = production
        .split_once("fn refresh_sorted_indices")
        .expect("shared sorted-index loader")
        .1;
    let return_position = loader
        .find("return")
        .expect("cached permutation early return");
    let clear_position = loader.find("indices.clear()").expect("rebuild clear");

    assert!(production.contains("fn refresh_sorted_indices<T: Ord>"));
    assert_eq!(production.matches("refresh_sorted_indices(&mut").count(), 2);
    assert!(loader.contains(".windows(2)"));
    assert!(loader.contains(".all(|pair|"));
    assert!(return_position < clear_position);
    assert_eq!(loader.matches("sort_unstable_by").count(), 1);
}

/// 比较固定源切片重复加载的基线与当前实现耗时样本。
///
/// # 参数
/// 无外部参数；按样本奇偶交替实现测量先后，输入规模、迭代和样本数来自测试常量。
/// 默认忽略，release-only 注解不检查实际构建配置。
///
/// # 返回
/// 返回 `()`；输出 nearest-rank 95% 时间和饱和比值，并断言当前耗时不大于基线的 35%。
/// 打印的下标写入和排序次数来自固定文字，未由操作计数器测量；未运行时无性能结论。
///
/// # 使用时机与生命周期
/// 只测已准备固定轴切片的重载，不测按钮字符串、成员查询、源数据变化或整个动作求值。
/// 局部样本就地排序后统计，计时结果只代表该样例实际执行时的环境和输入。
///
/// # 参考
/// 参见 [`measure_repeated_load`]、[`percentile_95`]、[`BENCHMARK_SAMPLES`]。
#[test]
#[ignore = "release-only performance evidence"]
fn runtime56_recovery_batch_reused_consumed_input_permutation_release_benchmark() {
    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_repeated_load(retired_load_sorted_indices));
            optimized_samples.push(measure_repeated_load(refresh_sorted_indices));
        } else {
            optimized_samples.push(measure_repeated_load(refresh_sorted_indices));
            retired_samples.push(measure_repeated_load(retired_load_sorted_indices));
        }
    }

    let retired_p95 = percentile_95(&mut retired_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
    );
    eprintln!(
        "RUNTIME56_REUSED_CONSUMED_INPUT_PERMUTATION_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} inputs={BENCHMARK_INPUT_COUNT} \
retired_index_writes_per_reload=4096 optimized_index_writes_per_reload=0 \
retired_sorts_per_reload=1 optimized_sorts_per_reload=0 \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
        retired_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= retired_p95.as_nanos().saturating_mul(35),
        "reusing the consumed-input permutation must reduce stable reload P95 by at least 65%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
    );
}

/// 构造固定 LeftStickX 的测试设备轴身份。
///
/// # 参数
/// `gamepad` 是直接保存的 u64 设备号，不检验设备是否实际连接。
///
/// # 返回
/// 返回按值的 `GamepadAxisInput`，轴固定为 LeftStickX。
///
/// # 使用时机与生命周期
/// 用于固定 fixture 和计时源；不保留借用或创建设备服务。
///
/// # 参考
/// 参见 [`GamepadAxisInput`]、[`measure_repeated_load`]。
fn axis_input(gamepad: u64) -> GamepadAxisInput {
    GamepadAxisInput::new(GamepadId(gamepad), GamepadAxis::LeftStickX)
}

/// 以每次清空、重写和排序完整下标的测试基线准备轴排列。
///
/// # 参数
/// `indices` 独占借用测试排列；`values` 只读提供轴身份，当前旧下标不被复用。
///
/// # 返回
/// 返回 `()`；清空后生成所有源下标，再按源身份排序，保留重复源条目；相等键顺序不稳定。
///
/// # 使用时机与生命周期
/// 不保存源借用或复制键，保留向量容量；用于当前计时基线，不代表历史性能结果。
///
/// # 参考
/// 参见 [`refresh_sorted_indices`]、[`ConsumedInputIndex::load`]。
fn retired_load_sorted_indices(indices: &mut Vec<usize>, values: &[GamepadAxisInput]) {
    indices.clear();
    indices.extend(0..values.len());
    indices.sort_unstable_by(|left, right| values[*left].cmp(&values[*right]));
}

/// 计时回调对同一已准备轴切片的重复下标加载。
///
/// # 参数
/// `load` 按值提供 FnMut 回调，接收本地下标向量的独占借用和固定源切片的只读借用。
/// 回调可修改下标、产生其它副作用或 panic；此助手不隔离或核验回调行为。
///
/// # 返回
/// 返回循环结束后的 `Duration`；源构造、首次回调加载和首次 black_box 均在计时外，循环调用与 black_box 在计时内。
///
/// # 使用时机与生命周期
/// 固定轴源在本次测量内不重建，当前 4096 个设备号由奇数乘数和位掩码重排。
/// 首次加载建立排列和容量，随后重复 128 次；借用只在各次回调内，返回值不借用任何向量。
/// 此计时不保证无排序、无写入或达到断言阈值，具体工作由所传回调决定。
///
/// # 参考
/// 参见 [`BENCHMARK_INPUT_COUNT`]、[`BENCHMARK_ITERATIONS`]、[`axis_input`]。
fn measure_repeated_load(mut load: impl FnMut(&mut Vec<usize>, &[GamepadAxisInput])) -> Duration {
    let values = (0..BENCHMARK_INPUT_COUNT)
        .map(|index| {
            let permuted = index.wrapping_mul(4_053) & (BENCHMARK_INPUT_COUNT - 1);
            axis_input(permuted as u64)
        })
        .collect::<Vec<_>>();
    let mut indices = Vec::new();
    load(&mut indices, &values);
    black_box((&values, &indices));

    let started = Instant::now();
    for _ in 0..BENCHMARK_ITERATIONS {
        load(&mut indices, &values);
        black_box(&indices);
    }
    started.elapsed()
}

/// 就地排序耗时样本并读取 nearest-rank 95% 位置。
///
/// # 参数
/// `samples` 必须非空，且长度乘 95 能表示为 usize；函数不检查这两个前提。
///
/// # 返回
/// 返回 `ceil(len * 95 / 100) - 1` 位置的 `Duration` 副本，空切片索引会 panic。
/// 当前 11 个样本取位置 10，即最大值；输入样本顺序被排序改变。
///
/// # 使用时机与生命周期
/// 调用期间独占借用样本，返回后不保留借用；不执行计时或验证样本来源。
///
/// # 参考
/// 参见 [`BENCHMARK_SAMPLES`]。
fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}
