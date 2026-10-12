//! 测试工作区动作槽位的增长、缩减、复用、源码形状与重置计时。
//!
//! 此文件由父模块在测试构建接入，可访问其私有槽位和辅助函数；功能夹具用旧算法作对照，计时夹具默认忽略。
//! 所有向量与样本均由当前测试调用拥有，夹具只覆盖局部动作存储。
//! 源码守卫按文本分隔符截取生产代码；当前分隔符与带 path 属性的模块声明不匹配，不能据此声称该测试已通过。
//!
//! # 参考
//!
//! [`ActionEvaluationWorkspace`]、[`reset_action_storage`]、[`action_signatures`]。
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

/// 计时夹具每轮重建的65,536个动作槽位。
///
/// 用于放大默认写入次数差异，预分配容量和回调目标长度共用此值；它不限制生产映射的动作数。
///
/// # 参考
///
/// [`measure_action_reset`]、[`BENCHMARK_ITERATIONS`]。
const BENCHMARK_ACTION_COUNT: usize = 65_536;
/// 计时夹具为每种算法收集的11个交替顺序样本。
///
/// 值须为正以供分位数索引；当前样本数下经验P95取最大值，不代表生产帧数或置信区间。
///
/// # 参考
///
/// [`percentile_95`]、[`BENCHMARK_ITERATIONS`]。
const BENCHMARK_SAMPLES: usize = 11;
/// 每个计时样本内的128轮清空、回调重置和观测循环。
///
/// 只控制测试工作量；回调目标槽位数由另一常量给定，不表示生产求值频率。
///
/// # 参考
///
/// [`measure_action_reset`]、[`BENCHMARK_ACTION_COUNT`]。
const BENCHMARK_ITERATIONS: usize = 128;

/// 对照增长、缩减、重复长度及归零时的槽位重置结果。
///
/// 先用四个非默认槽位初始化，按 `[9, 3, 3, 17, 0]` 比较生产和旧算法，再为下一轮重新填种子。只检查结果等价，不直接断言容量、增长计数或耗时。
///
/// # 参数
///
/// 无参数；由测试框架运行。
///
/// # 返回
///
/// 返回 `()`；任一逐槽位模式不等时断言失败。
///
/// # 生命周期
///
/// 两份动作向量只在测试调用内变更并释放；每轮比较后重新填种子，避免上一轮默认值掩盖漏重置。
///
/// # 参考
///
/// [`ActionEvaluationWorkspace::prepare_actions`]、[`EvaluatedAction`]、[`action_signatures`]、[`retired_prepare_actions`]。
#[test]
fn runtime56_recovery_batch_growth_aware_action_reset_preserves_grow_shrink_and_reuse_behavior() {
    let seed = EvaluatedAction {
        pressed: true,
        just_activated: true,
        just_deactivated: true,
        value: 0.75,
    };
    let mut workspace = ActionEvaluationWorkspace {
        actions: vec![seed; 4],
        ..ActionEvaluationWorkspace::default()
    };
    let mut retired = workspace.actions.clone();

    for action_count in [9, 3, 3, 17, 0] {
        workspace.prepare_actions(action_count);
        retired_prepare_actions(&mut retired, action_count);
        assert_eq!(
            action_signatures(&workspace.actions),
            action_signatures(&retired)
        );
        workspace.actions.fill(seed);
        retired.fill(seed);
    }
}

/// 按嵌入生产文件的文本检查委托及旧前缀重置形状。
///
/// 文件在编译期嵌入，不执行动作求值。当前首个预期分隔符跳过了生产声明中的 path 属性，不能取得生产片段；即使后续模式独立存在，也不能把该夹具记为通过。注释文字可能改变首次标记及模式搜索。
///
/// # 参数
///
/// 无参数；由测试框架运行。
///
/// # 返回
///
/// 返回 `()`；分隔符、函数标记或形状断言不匹配时 panic。
///
/// # 生命周期
///
/// 嵌入文本为编译期常量，函数只借读；当前分隔失败先于后续模式断言，不产生动作状态或有效验证回执。
///
/// # 参考
///
/// [`std::include_str`]、[`ActionEvaluationWorkspace::prepare_actions`]、[`reset_action_storage`]。
#[test]
fn runtime56_recovery_batch_growth_aware_action_reset_source_contract() {
    let source = include_str!("../workspace.rs");
    let production = source
        .split_once("#[cfg(test)]\nmod tests")
        .expect("production module end")
        .0;
    let prepare_actions = production
        .split_once("fn prepare_actions")
        .expect("prepare actions")
        .1
        .split_once("fn prepare_contexts")
        .expect("prepare actions end")
        .0;
    let reset_storage = production
        .split_once("fn reset_action_storage")
        .expect("reset action storage")
        .1;

    assert!(prepare_actions.contains("reset_action_storage"));
    assert!(reset_storage.contains("let reset_len"));
    assert!(reset_storage.contains("actions[..reset_len].fill"));
    assert!(!reset_storage.contains("actions.fill"));
}

/// 交替计时旧算法和前缀重置，并比较经验P95。
///
/// 默认 `ignored`，按约定使用优化构建显式运行；函数本身不校验构建模式。11个样本的P95为最大值，计时受环境和回调工作影响，读源码不产生性能结果。
///
/// # 参数
///
/// 无参数；由测试框架运行。
///
/// # 返回
///
/// 返回 `()`；输出本次测量文本，当前算法P95大于旧算法75%时断言失败。
///
/// # 生命周期
///
/// 样本和临时向量属于本次调用；显式执行后才有该环境的日志和断言结果，后续运行须保留各自证据。
///
/// # 参考
///
/// [`measure_action_reset`]、[`percentile_95`]、[`retired_prepare_actions`]、[`reset_action_storage`]。
#[test]
#[ignore = "release-only performance evidence"]
fn runtime56_recovery_batch_growth_aware_action_reset_release_benchmark() {
    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_action_reset(retired_prepare_actions));
            optimized_samples.push(measure_action_reset(reset_action_storage));
        } else {
            optimized_samples.push(measure_action_reset(reset_action_storage));
            retired_samples.push(measure_action_reset(retired_prepare_actions));
        }
    }

    let retired_p95 = percentile_95(&mut retired_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
    );
    eprintln!(
        "RUNTIME56_GROWTH_AWARE_ACTION_RESET_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} \
actions={BENCHMARK_ACTION_COUNT} retired_default_writes_per_fresh_reset=131072 \
optimized_default_writes_per_fresh_reset=65536 retired_p95_ns={} optimized_p95_ns={} \
reduction_basis_points={reduction_basis_points}",
        retired_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= retired_p95.as_nanos().saturating_mul(75),
        "growth-aware action reset must reduce fresh-reset P95 by at least 25%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
    );
}

/// 按槽位顺序收集布尔字段及浮点位模式签名。
///
/// 借用只持续本次调用；不修改输入。浮点使用位模式比较，不做归一化或容差判断。
///
/// # 参数
///
/// - `actions`：只读动作结果切片。
///
/// # 返回
///
/// 拥有型签名向量，长度和顺序与输入一致。
///
/// # 生命周期
///
/// 返回向量拥有全部复制签名，不继续借用输入；销毁签名不影响原动作结果。
///
/// # 参考
///
/// [`EvaluatedAction`]、[`f32::to_bits`]。
fn action_signatures(actions: &[EvaluatedAction]) -> Vec<(bool, bool, bool, u32)> {
    actions
        .iter()
        .map(|action| {
            (
                action.pressed,
                action.just_activated,
                action.just_deactivated,
                action.value.to_bits(),
            )
        })
        .collect()
}

/// 执行旧版先调整长度再完整填默认值的重置对照。
///
/// 新增槽位先由resize初始化，再被全填充一次；缩减丢弃尾部，可复用已有容量。本函数只用于测试对照及计时。
///
/// # 参数
///
/// - `actions`：独占借用的动作向量。
/// - `action_count`：目标逻辑长度。
///
/// # 返回
///
/// 返回 `()`；全部逻辑槽位为默认结果。
///
/// # 生命周期
///
/// 动作向量的所有权仍归调用方，方法返回前完成全部默认写入；已有容量可用于后续对照轮次。
///
/// # 参考
///
/// [`EvaluatedAction`]、[`reset_action_storage`]。
fn retired_prepare_actions(actions: &mut Vec<EvaluatedAction>, action_count: usize) {
    actions.resize(action_count, EvaluatedAction::default());
    actions.fill(EvaluatedAction::default());
}

/// 测量预分配向量上固定循环的回调重置总耗时。
///
/// 预分配和首次填充在计时前；每轮先把长度清零而保留容量。回调可以执行任意工作，结果不自动代表生产帧性能；向量和闭包在返回时销毁，其释放不计入已取出的耗时。
///
/// # 参数
///
/// - `reset`：被本函数拥有并同步调用的 `FnMut`，接收向量独占借用及目标长度。
///
/// # 返回
///
/// 本机 `Duration` 总耗时；包含128轮clear、回调与black_box。
///
/// # 生命周期
///
/// 同步回调不保留向量借用；取出 elapsed 后才结束本地向量和闭包的生命周期，释放耗时不进入已返回数值。
///
/// # 参考
///
/// [`Duration`]、[`Instant`]、[`BENCHMARK_ACTION_COUNT`]、[`BENCHMARK_ITERATIONS`]。
fn measure_action_reset(mut reset: impl FnMut(&mut Vec<EvaluatedAction>, usize)) -> Duration {
    let mut actions = Vec::with_capacity(BENCHMARK_ACTION_COUNT);
    actions.resize(BENCHMARK_ACTION_COUNT, EvaluatedAction::default());
    let started = Instant::now();
    for _ in 0..BENCHMARK_ITERATIONS {
        actions.clear();
        reset(&mut actions, BENCHMARK_ACTION_COUNT);
        black_box(&actions);
    }
    started.elapsed()
}

/// 就地排序耗时样本并取上取整秩的经验P95。
///
/// 排序改变输入顺序，结果不保留借用；空输入索引越界会panic，方法不插值或估计置信区间。
///
/// # 参数
///
/// - `samples`：必须非空的独占样本切片；长度乘95也须能以usize表达。
///
/// # 返回
///
/// 按 `ceil(len*95/100)-1` 位置复制的 `Duration`；当前11样本取最大值。
///
/// # 生命周期
///
/// 排序改变借入切片的次序；返回时结束独占借用，复制的时长不依赖切片继续存续。
///
/// # 参考
///
/// [`Duration`]、[`BENCHMARK_SAMPLES`]。
fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}
