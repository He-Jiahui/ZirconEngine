//! 核对轴记录压缩的固定签名，并保留源码形状守卫和默认忽略的计时样例。
//!
//! # 约束与生命周期
//! 功能 fixture 直接调用两个压缩助手，没有经过整帧加载或二分查询；签名省略部分身份字段。
//! 源码守卫的首个分隔当前因路径属性不匹配；后续形状断言不能当作已经运行通过。
//! 计时只覆盖当前值压缩，默认忽略且不检查实际 release 配置；结果只对应实际执行的样例和环境。
//!
//! # 参考
//! 参见 [`retain_latest_value`]、[`retain_latest_transition`] 与 [`measure_value_compaction`]。
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;
use crate::input::{GamepadAxis, GamepadId};

/// 计时助手构造 4096 个不同设备的固定轴记录；值为下标转换的 f32，样本没有重复身份。
///
/// # 生命周期
/// 仅测试编译期常量；不影响生产索引、帧来源或运行时配置。
///
/// # 参考
/// 参见 [`measure_value_compaction`]。
const BENCHMARK_AXIS_COUNT: usize = 4_096;
/// 每种实现收集 11 个计时样本；当前 nearest-rank 95% 规则在此长度取最大值。
///
/// # 生命周期
/// 仅测试编译期常量；不影响生产索引、帧来源或运行时配置。
///
/// # 参考
/// 参见 [`percentile_95`]。
const BENCHMARK_SAMPLES: usize = 11;
/// 每个计时样本原位压缩同一值向量 256 次，不逐次重建源数据。
///
/// # 生命周期
/// 仅测试编译期常量；不影响生产索引、帧来源或运行时配置。
///
/// # 参考
/// 参见 [`measure_value_compaction`]。
const BENCHMARK_ITERATIONS: usize = 256;

/// 比较两个当前压缩助手和测试基线对固定相邻同轴样本的签名结果。
///
/// # 参数
/// 无外部参数；fixture 各含两根固定轴、三条已分组记录，第一轴的较后源记录覆盖前一条。
///
/// # 返回
/// 返回 `()`；分别断言值和转换签名与基线一致并符合给定元组；失败使测试失败。
///
/// # 使用时机与生命周期
/// 记录和基线克隆均为本地拥有；直接测试压缩，不测试 load_frame 的排序、查询或所有字段。
/// 签名仅覆盖设备号、源位置和浮点载荷，不能推导省略的轴字段或其它输入组合均被覆盖。
///
/// # 参考
/// 参见 [`retain_latest_value`]、[`retain_latest_transition`]、[`value_signatures`]、[`transition_signatures`]。
#[test]
fn runtime56_recovery_batch_skipped_frame_axis_self_swaps_preserve_latest_value_and_transition() {
    let first_input = axis_input(1);
    let second_input = axis_input(2);
    let mut values = vec![
        axis_value(first_input, 0, 0.25),
        axis_value(first_input, 2, 0.75),
        axis_value(second_input, 1, -0.5),
    ];
    let mut retired_values = values.clone();
    retain_latest_value(&mut values);
    retired_retain_latest_value(&mut retired_values);

    assert_eq!(value_signatures(&values), value_signatures(&retired_values));
    assert_eq!(value_signatures(&values), vec![(1, 2, 0.75), (2, 1, -0.5)]);

    let mut transitions = vec![
        axis_transition(first_input, 0, 0.0, 0.25),
        axis_transition(first_input, 2, 0.25, 0.75),
        axis_transition(second_input, 1, 0.0, -0.5),
    ];
    let mut retired_transitions = transitions.clone();
    retain_latest_transition(&mut transitions);
    retired_retain_latest_transition(&mut retired_transitions);

    assert_eq!(
        transition_signatures(&transitions),
        transition_signatures(&retired_transitions)
    );
    assert_eq!(
        transition_signatures(&transitions),
        vec![(1, 2, 0.25, 0.75), (2, 1, 0.0, -0.5)]
    );
}

/// 按嵌入生产源码的字符串检查两类压缩器的条件交换形状。
///
/// # 参数
/// 无外部参数；`include_str!` 相对本文件解析到生产轴索引。
/// Rust 字符串中的单个反斜线 n 解释为真实换行，期待相邻条件属性与模块声明。
///
/// # 返回
/// 返回 `()`，但当前生产文件两者之间有路径属性，首个 `expect` 会失败；后续字符串断言因此不代表当前通过的守卫。
///
/// # 使用时机与生命周期
/// 只检查原始文本，注释可改变分隔、命中次数或提取位置；文本匹配只说明这些字符串条件，不能替代编译器控制流或实际性能验证。
/// 包含路径属性的挂载布局不满足首个分隔假设；错误调用附近的 BUG 标签给出失败路径。
///
/// # 参考
/// 参见 [`retain_latest_value`]、[`retain_latest_transition`]。
#[test]
fn runtime56_recovery_batch_skipped_frame_axis_self_swaps_source_contract() {
    let source = include_str!("../frame_axis_index.rs");
    // BUG: [CR-INPUT-INDEX-0001] 执行此源码形状测试时，生产模块的 cfg 与 mod 之间包含 path 属性；
    // include_str! 原文中相邻 cfg→mod 分隔符出现零次，下面 expect 必然先 panic，后续形状断言不可达。
    let production = source
        .split_once("\n#[cfg(test)]\nmod tests")
        .expect("production module end")
        .0;
    let value_compaction = production
        .split_once("fn retain_latest_value")
        .expect("value compaction")
        .1
        .split_once("fn retain_latest_transition")
        .expect("value compaction end")
        .0;
    let transition_compaction = production
        .split_once("fn retain_latest_transition")
        .expect("transition compaction")
        .1;

    assert!(value_compaction.contains("if retained != index"));
    assert!(transition_compaction.contains("if retained != index"));
    assert_eq!(
        value_compaction
            .matches("values.swap(retained, index)")
            .count(),
        1
    );
    assert_eq!(
        transition_compaction
            .matches("transitions.swap(retained, index)")
            .count(),
        1
    );
}

/// 收集值压缩基线和当前实现的耗时样本，并断言样本统计量的降低阈值。
///
/// # 参数
/// 无外部参数；使用固定样本数量和唯一身份记录，按样本奇偶交替先测哪种实现。
/// 测试默认忽略；注解写明 release-only，但函数本身不检查实际构建配置。
///
/// # 返回
/// 返回 `()`；输出 nearest-rank 95% 时间和比值，并断言当前时间不大于基线的 80%。
/// 输出的自身交换次数是固定文字，不来自动态计数器；未运行时没有性能结论。
///
/// # 使用时机与生命周期
/// 局部样本排序后统计；这里只计时当前值压缩，不包含整帧复制、排序、转换查询或整个求值器。
/// 必须实际执行该样例才能取得对应环境的耗时与阈值结果。
///
/// # 参考
/// 参见 [`measure_value_compaction`]、[`percentile_95`]、[`BENCHMARK_SAMPLES`]。
#[test]
#[ignore = "release-only performance evidence"]
fn runtime56_recovery_batch_skipped_frame_axis_self_swaps_release_benchmark() {
    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_value_compaction(retired_retain_latest_value));
            optimized_samples.push(measure_value_compaction(retain_latest_value));
        } else {
            optimized_samples.push(measure_value_compaction(retain_latest_value));
            retired_samples.push(measure_value_compaction(retired_retain_latest_value));
        }
    }

    let retired_p95 = percentile_95(&mut retired_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
    );
    eprintln!(
        "RUNTIME56_SKIP_FRAME_AXIS_SELF_SWAPS_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} axes={BENCHMARK_AXIS_COUNT} \
retired_self_swaps_per_compaction=4096 optimized_self_swaps_per_compaction=0 \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
        retired_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= retired_p95.as_nanos().saturating_mul(80),
        "skipping frame-axis self swaps must reduce unique-axis compaction P95 by at least 20%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
    );
}

/// 构造固定 LeftStickX 轴的测试身份。
///
/// # 参数
/// `gamepad` 按值作为设备 u64 标识，不执行设备存在性检查。
///
/// # 返回
/// 返回拥有按值设备和固定轴的 `GamepadAxisInput`。
///
/// # 使用时机与生命周期
/// 用于小型 fixture 和计时样本；不保留任何借用，不创建或连接设备。
///
/// # 参考
/// 参见 [`axis_value`]、[`axis_transition`]。
fn axis_input(gamepad: u64) -> GamepadAxisInput {
    GamepadAxisInput::new(GamepadId(gamepad), GamepadAxis::LeftStickX)
}

/// 构造一条当前值压缩测试记录。
///
/// # 参数
/// `input` 按值给出轴身份；`source_index` 是测试指定源位置；`value` 原样保存为 f32，未归一化或验证。
///
/// # 返回
/// 返回按值的 `AxisValue`，不会自动排序或加入索引。
///
/// # 使用时机与生命周期
/// 只用于当前 fixture/计时助手；记录可 Copy，没有输入借用，源位置约束由调用方提供。
///
/// # 参考
/// 参见 [`AxisValue`]、[`retain_latest_value`]。
fn axis_value(input: GamepadAxisInput, source_index: usize, value: f32) -> AxisValue {
    AxisValue {
        input,
        source_index,
        value,
    }
}

/// 构造一条转换压缩测试记录。
///
/// # 参数
/// `input` 给出轴身份；`source_index` 给出源位置；`previous_value` 和 `value` 原样构造转换的前后值。
///
/// # 返回
/// 返回 `AxisTransition`；内部转换的设备和轴取自同一 `input`，不验证数值或转换边沿。
///
/// # 使用时机与生命周期
/// 记录拥有复制字段，没有帧借用；用于固定测试比较，不补齐其它帧状态。
///
/// # 参考
/// 参见 [`AxisTransition`]、[`retain_latest_transition`]。
fn axis_transition(
    input: GamepadAxisInput,
    source_index: usize,
    previous_value: f32,
    value: f32,
) -> AxisTransition {
    AxisTransition {
        input,
        source_index,
        transition: GamepadAxisTransition {
            gamepad: input.gamepad,
            axis: input.axis,
            previous_value,
            value,
        },
    }
}

/// 把值记录投影为便于当前测试比较的元组。
///
/// # 参数
/// `values` 只读借用当前记录序列，投影保持输入顺序。
///
/// # 返回
/// 返回拥有的 `Vec<(u64, usize, f32)>`，包含设备号、源位置和值；省略轴字段。
///
/// # 使用时机与生命周期
/// 调用期间借用，返回向量不借用输入；只比较选定字段，不能替代完整记录相等验证。
///
/// # 参考
/// 参见 [`AxisValue`]。
fn value_signatures(values: &[AxisValue]) -> Vec<(u64, usize, f32)> {
    values
        .iter()
        .map(|value| (value.input.gamepad.0, value.source_index, value.value))
        .collect()
}

/// 把转换记录投影为便于当前测试比较的元组。
///
/// # 参数
/// `transitions` 只读借用输入序列，顺序不改变。
///
/// # 返回
/// 返回拥有的设备号、源位置、前值和末值元组向量；省略轴和内部转换身份字段。
///
/// # 使用时机与生命周期
/// 新向量拥有复制字段，不借用记录；覆盖范围只包含这些元组字段。
///
/// # 参考
/// 参见 [`AxisTransition`]。
fn transition_signatures(transitions: &[AxisTransition]) -> Vec<(u64, usize, f32, f32)> {
    transitions
        .iter()
        .map(|transition| {
            (
                transition.input.gamepad.0,
                transition.source_index,
                transition.transition.previous_value,
                transition.transition.value,
            )
        })
        .collect()
}

/// 在测试中以包含自身交换的基线形状压缩相邻值记录。
///
/// # 参数
/// `values` 须按身份分组；表达最后源记录还需组内按源序排列，助手不排序或验证。
///
/// # 返回
/// 返回 `()`；同组最后项覆盖前缀位置，新组无条件交换并增加保留长度，最终截断。
///
/// # 使用时机与生命周期
/// 只独占借用本地向量，容量保留；基线用于签名比较和固定样本计时，不代表历史运行结果。
///
/// # 参考
/// 参见 [`retain_latest_value`]、[`AxisValue`]。
fn retired_retain_latest_value(values: &mut Vec<AxisValue>) {
    let mut retained = 0;
    for index in 0..values.len() {
        if retained > 0 && values[retained - 1].input == values[index].input {
            values[retained - 1] = values[index];
        } else {
            values.swap(retained, index);
            retained += 1;
        }
    }
    values.truncate(retained);
}

/// 以包含自身交换的测试基线形状压缩相邻转换记录。
///
/// # 参数
/// `transitions` 须按身份分组并满足希望保留的源序；不验证输入。
///
/// # 返回
/// 返回 `()`；同组覆盖、新组无条件交换，末尾截断保留前缀。
///
/// # 使用时机与生命周期
/// 只借用本地转换向量，不创建帧索引；用于当前小型签名比较，不在值压缩计时中执行。
///
/// # 参考
/// 参见 [`retain_latest_transition`]、[`AxisTransition`]。
fn retired_retain_latest_transition(transitions: &mut Vec<AxisTransition>) {
    let mut retained = 0;
    for index in 0..transitions.len() {
        if retained > 0 && transitions[retained - 1].input == transitions[index].input {
            transitions[retained - 1] = transitions[index];
        } else {
            transitions.swap(retained, index);
            retained += 1;
        }
    }
    transitions.truncate(retained);
}

/// 计时回调对固定唯一身份值向量的重复原位处理。
///
/// # 参数
/// `retain` 按值传入可变调用回调，每次接收同一个向量的独占借用。
/// 回调可修改规模、产生其它副作用或 panic；此助手不验证回调保留数据或仅执行压缩。
///
/// # 返回
/// 返回循环结束后的 `Duration`；样本构造和首次 black_box 在计时开始前，循环内调用和 black_box 在计时内。
///
/// # 使用时机与生命周期
/// 本地向量包含固定数量唯一身份，初始按下标生成；重复调用不逐次重置数据。
/// 回调借用只在各次调用内，本助手不返回向量借用；只记录当前执行耗时，非性能保证。
///
/// # 参考
/// 参见 [`BENCHMARK_AXIS_COUNT`]、[`BENCHMARK_ITERATIONS`]、[`axis_value`]。
fn measure_value_compaction(mut retain: impl FnMut(&mut Vec<AxisValue>)) -> Duration {
    let mut values = (0..BENCHMARK_AXIS_COUNT)
        .map(|index| axis_value(axis_input(index as u64), index, index as f32))
        .collect::<Vec<_>>();
    black_box(&mut values);

    let started = Instant::now();
    for _ in 0..BENCHMARK_ITERATIONS {
        retain(&mut values);
        black_box(&values);
    }
    started.elapsed()
}

/// 就地排序耗时样本并读取 nearest-rank 95% 位置。
///
/// # 参数
/// `samples` 必须非空，且长度乘 95 不溢出 usize；函数不验证这两个前提。
///
/// # 返回
/// 返回排序后 `ceil(len * 95 / 100) - 1` 位置的 `Duration`；空切片索引会 panic。
/// 当前 11 个样本的位置为 10，即最大值；排序改变输入顺序。
///
/// # 使用时机与生命周期
/// 只在调用期间独占借用样本；返回值为副本，无样本借用，不执行测量或拒绝异常样本。
///
/// # 参考
/// 参见 [`BENCHMARK_SAMPLES`]。
fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}
