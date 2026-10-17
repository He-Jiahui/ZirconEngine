//! 对原始记录容量裁剪进行固定输入比较、源码文本检查及默认忽略的计时比较。
//!
//! # 入口与范围
//! 父记录器模块通过 `cfg(test)` 与显式文件路径接入本模块，测试可访问其私有裁剪函数和导入类型。
//! 功能比较操作本地队列副本，源码检查读取父文件文本；计时样例只在显式选择时运行。
//!
//! # 约束与生命周期
//! 固定输入的比较不能覆盖所有容量、队列和录制周期，也不证明管理器或帧缓冲行为。
//! 源码检查依赖精确分隔文本；当前父文件的路径属性会打断它期待的测试模块分隔布局。
//! 忽略属性中的 release 提示不检查优化级别；未实际运行时没有性能结果。
//! 局部记录、队列和计时样本随函数作用域释放，不修改管理器持有的生产记录器。
//!
//! # 参考
//! 参见 [`super::trim_records_to_capacity`]、[`super::InputEventRecorder`] 与 [`InputEventRecord`]。
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

/// 每个裁剪实现收集的时长样本数：11。
/// 每轮各测旧循环与批量裁剪一次，并交替先后顺序；此值须非零。
/// 本文件的百分位算法在 11 个样本中选择最大值。
///
/// # 参考
/// 参见 [`percentile_95`] 与 [`runtime56_recovery_batch_bulk_recorder_capacity_trim_release_benchmark`]。
const BENCHMARK_SAMPLES: usize = 11;
/// 每个时长样本裁剪独立基础队列副本的数量：16。
/// 副本在计时开始前创建；测量的是 16 次裁剪的总区间，不能当作单次或逐帧耗时。
///
/// # 参考
/// 参见 [`measure_trim`] 与 [`BENCHMARK_CAPACITY`]。
const BENCHMARK_ITERATIONS: usize = 16;
/// 计时基础 FIFO 的记录条目数：16,384。
/// 序号取从零开始的连续区间；此值是测试压力规模，不是生产记录容量或上限。
///
/// # 参考
/// 参见 [`event_record`]、[`BENCHMARK_CAPACITY`] 与 [`measure_trim`]。
const BENCHMARK_RECORD_COUNT: usize = 16_384;
/// 每个计时副本裁剪后的条目上限：256。
/// 两实现以同一容量比较，当前基础队列每次超额 16,128 条；该常量不配置生产记录器。
///
/// # 参考
/// 参见 [`BENCHMARK_RECORD_COUNT`]、[`measure_trim`] 与 [`super::trim_records_to_capacity`]。
const BENCHMARK_CAPACITY: usize = 256;

/// 对固定超额队列比较旧逐条裁剪与生产批量裁剪的结果。
///
/// # 输入与断言范围
/// 局部队列保存序号 1 至 8，容量为 3，初始丢弃计数为 `u64::MAX - 2`。
/// 比较完整保留记录和计数，并明确检查仅保留序号 6、7、8 及计数饱和到 `u64::MAX`。
/// 此输入不覆盖零容量、无需裁剪或实际录制配置周期。
///
/// # 返回与生命周期
/// 正常返回 `()` 表示本次固定输入的断言成立，失败会 panic；两队列和计数均为局部状态。
/// 函数直接调用裁剪辅助实现，不创建或修改管理器持有的记录器。
///
/// # 参考
/// 参见 [`event_record`]、[`retired_trim_records_to_capacity`] 与 [`super::trim_records_to_capacity`]。
#[test]
fn runtime56_recovery_batch_bulk_recorder_capacity_trim_preserves_retired_records_and_counter() {
    let records = (1..=8).map(event_record).collect::<VecDeque<_>>();
    let mut retired_records = records.clone();
    let mut optimized_records = records;
    let mut retired_discarded = u64::MAX - 2;
    let mut optimized_discarded = retired_discarded;

    retired_trim_records_to_capacity(&mut retired_records, 3, &mut retired_discarded);
    trim_records_to_capacity(&mut optimized_records, 3, &mut optimized_discarded);

    assert_eq!(optimized_records, retired_records);
    assert_eq!(optimized_discarded, retired_discarded);
    assert_eq!(
        optimized_records
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>(),
        vec![6, 7, 8]
    );
    assert_eq!(optimized_discarded, u64::MAX);
}

/// 从父记录器源码中提取生产与裁剪区段，并检查批量裁剪的文本模式。
///
/// # 输入与约束
/// [`std::include_str!`] 在编译时将相邻父文件嵌入为静态字符串。
/// 第一处分隔期待测试条件属性后直接出现测试模块声明；当前父文件在两者之间含路径属性，不能匹配该分隔。
/// 即使分隔匹配，后续检查也只依据文本：调用标记、饱和减法、批量排空、一次加法标记及旧循环模式缺失。
/// 文档文字也在源码中，可能改变首次标记位置或匹配次数；这些检查不是语法分析或动态录制验证。
///
/// # 返回与失败
/// 返回 `()`；分隔或裁剪标记缺失时 `expect` 会 panic，断言不满足时也会 panic。
/// 按当前父文件布局，第一处分隔缺失会先于后续断言触发失败；静态说明不表示本次执行过测试。
///
/// # 生命周期
/// 提取区段借用静态嵌入文本，不改变源文件或生产队列。
///
/// # 参考
/// 参见 [`super::trim_records_to_capacity`] 与 [`std::include_str!`]。
#[test]
fn runtime56_recovery_batch_bulk_recorder_capacity_trim_source_contract() {
    let source = include_str!("../recorder.rs");
    let production = crate::tests::runtime_absorption::structure_convention::rust_source_view::production_section(source);
    let trim = production
        .split_once("fn trim_records_to_capacity")
        .expect("bulk recorder trim helper")
        .1;

    assert!(production.contains("trim_records_to_capacity("));
    assert!(trim.contains("saturating_sub(capacity)"));
    assert!(trim.contains("drop(records.drain(..excess))"));
    assert_eq!(trim.matches("saturating_add").count(), 1);
    assert!(!trim.contains("while records.len() > capacity"));
    assert!(!trim.contains("records.pop_front()"));
}

/// 交替采样旧逐条裁剪与批量裁剪，并按样本时长比较固定容量缩减成本。
///
/// # 使用时机与约束
/// `#[ignore]` 令普通测试默认跳过；属性中的 release 提示不检查实际优化级别。
/// 基础队列含 16,384 条记录，每次保留上限为 256；每侧收集 11 个样本，各样本处理 16 个独立副本。
/// 复制和初始化发生在计时前；11 样本的所选 P95 是最大值，样本表示一批裁剪而非单次调用。
///
/// # 返回、输出与失败
/// 返回 `()`，向标准错误打印固定参数、由规模推算的操作数量及测得的时长和整数比例。
/// 日志中的排空与计数更新次数不是动态计数器测量；未实际运行时没有测得结果。
/// 按代码中的饱和整数运算要求批量实现 P95 不超过旧实现的 80%，失败会断言 panic。
/// 该阈值只评价本次实际运行环境，不能证明所有机器的性能或裁剪功能正确性。
///
/// # 生命周期
/// 基础队列及样本由本函数拥有，副本由计时辅助函数局部管理。
///
/// # 参考
/// 参见 [`measure_trim`]、[`percentile_95`]、[`retired_trim_records_to_capacity`] 与 [`super::trim_records_to_capacity`]。
#[test]
#[ignore = "release-only performance evidence"]
fn runtime56_recovery_batch_bulk_recorder_capacity_trim_release_benchmark() {
    let base = (0..BENCHMARK_RECORD_COUNT as u64)
        .map(event_record)
        .collect::<VecDeque<_>>();
    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_trim(&base, retired_trim_records_to_capacity));
            optimized_samples.push(measure_trim(&base, trim_records_to_capacity));
        } else {
            optimized_samples.push(measure_trim(&base, trim_records_to_capacity));
            retired_samples.push(measure_trim(&base, retired_trim_records_to_capacity));
        }
    }

    let retired_p95 = percentile_95(&mut retired_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    let discarded_per_trim = BENCHMARK_RECORD_COUNT - BENCHMARK_CAPACITY;
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
    );
    eprintln!(
        "RUNTIME56_BULK_RECORDER_CAPACITY_TRIM_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} \
records={BENCHMARK_RECORD_COUNT} capacity={BENCHMARK_CAPACITY} \
retired_front_pops_per_trim={discarded_per_trim} optimized_drain_calls_per_trim=1 \
retired_counter_updates_per_trim={discarded_per_trim} optimized_counter_updates_per_trim=1 \
retired_p95_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
        retired_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= retired_p95.as_nanos().saturating_mul(80),
        "bulk trimming must reduce recorder capacity-shrink P95 by at least 20%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
    );
}

/// 用一个整数序号创建可比较的光标移动记录，用于局部测试队列。
///
/// # 参数
/// `sequence` 同时作为记录序号和毫秒字段，坐标由其转换成 `f32` 后取正、负值。
///
/// # 返回与约束
/// 返回拥有 [`InputEvent::CursorMoved`] 的 [`InputEventRecord`]，其 `x` 为转换值，`y` 为该值的相反数。
/// 整数转浮点可能舍入；毫秒字段只是 fixture 值，不采样墙钟，也不模拟真实录制序号规则。
///
/// # 生命周期
/// 返回值按值独立拥有事件，不借用参数或共享可变状态。
///
/// # 参考
/// 参见 [`InputEventRecord`] 与 [`runtime56_recovery_batch_bulk_recorder_capacity_trim_preserves_retired_records_and_counter`]。
fn event_record(sequence: u64) -> InputEventRecord {
    InputEventRecord {
        sequence,
        timestamp_millis: sequence,
        event: InputEvent::CursorMoved {
            x: sequence as f32,
            y: -(sequence as f32),
        },
    }
}

/// 逐条移除 FIFO 队首超额记录，作为批量裁剪的局部比较基线。
///
/// # 参数
/// `records` 是可变借用的队列，按插入顺序解释最旧与最新条目。
/// `capacity` 是要保留的条目上限，零会移除全部条目。
/// `discarded_records` 是累积计数，每次移除后使用饱和加一。
///
/// # 返回与生命周期
/// 返回 `()`；只保留队尾至多 `capacity` 条记录，已满足容量时不改队列或计数。
/// 移除记录在本函数内释放，计数可饱和至 `u64::MAX`；不改录制配置或序号，也不排序队列。
/// 函数只用于测试比较与计时，调用方保留两份可变数据的所有权。
///
/// # 参考
/// 参见 [`std::collections::VecDeque`]、[`InputEventRecord`] 与 [`super::trim_records_to_capacity`]。
fn retired_trim_records_to_capacity(
    records: &mut VecDeque<InputEventRecord>,
    capacity: usize,
    discarded_records: &mut u64,
) {
    while records.len() > capacity {
        records.pop_front();
        *discarded_records = discarded_records.saturating_add(1);
    }
}

/// 裁剪一批基础队列副本，并取得这批调用的经过时长。
///
/// # 参数
/// `base` 提供只读可克隆的记录队列；函数在计时前创建 [`BENCHMARK_ITERATIONS`] 个独立副本。
/// `trim` 是接受可变队列、容量和可变丢弃计数的函数指针；各调用使用 [`BENCHMARK_CAPACITY`] 与独立的零初值计数。
/// 函数指针可有副作用或 panic；本辅助函数不等待它另行启动的后台工作。
///
/// # 返回与计时边界
/// 返回 [`Duration`]，从副本与计数初始化之后开始，到全部裁剪和 `black_box` 借用之后结束。
/// 区间包含裁剪调用、其中移除记录的释放及循环开销，不包含基础队列克隆和初始化，也不包含返回时长求值之后的剩余副本销毁。
/// 结果是一批调用的总时长；函数指针 panic 会中断采样而不返回时长。
///
/// # 生命周期
/// 不改变或持有 `base`，局部副本和计数在函数结束时释放，返回值不借用它们。
///
/// # 参考
/// 参见 [`Instant`]、[`std::hint::black_box`] 与 [`runtime56_recovery_batch_bulk_recorder_capacity_trim_release_benchmark`]。
fn measure_trim(
    base: &VecDeque<InputEventRecord>,
    trim: fn(&mut VecDeque<InputEventRecord>, usize, &mut u64),
) -> Duration {
    let mut inputs = (0..BENCHMARK_ITERATIONS)
        .map(|_| base.clone())
        .collect::<Vec<_>>();
    let mut discarded_records = vec![0_u64; BENCHMARK_ITERATIONS];
    let started = Instant::now();
    for (records, discarded) in inputs.iter_mut().zip(&mut discarded_records) {
        trim(records, BENCHMARK_CAPACITY, discarded);
    }
    black_box((&inputs, &discarded_records));
    started.elapsed()
}

/// 原地排序时长样本并取得按秩选择的第 95 百分位值。
///
/// # 参数
/// `samples` 必须非空，且其长度乘以 95 应能由 `usize` 表示；调用方接受顺序被升序排序。
///
/// # 返回与失败
/// 返回 `ceil(len * 95 / 100) - 1` 经饱和减法得到的索引所指向的 [`Duration`] 副本。
/// 空切片会在索引处 panic；当前 [`BENCHMARK_SAMPLES`] 为 11，所选值是最大样本，不做插值。
///
/// # 生命周期
/// 仅改变输入切片顺序，返回值不借用该切片。
///
/// # 参考
/// 参见 [`BENCHMARK_SAMPLES`] 与 [`runtime56_recovery_batch_bulk_recorder_capacity_trim_release_benchmark`]。
fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}
