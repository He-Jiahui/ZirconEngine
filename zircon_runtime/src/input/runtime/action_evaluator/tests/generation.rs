//! 在测试构建中比较代际结构、检查源码模式，并提供默认忽略的构建计时样例。
//!
//! # 范围与入口
//! 父 `generation` 模块通过 `cfg(test)` 与显式文件路径接入本模块，允许访问私有代际数据。
//! 常规比较只覆盖本文件的固定输入；借用键守卫只检查提取后的源码文本。
//!
//! # 证据与生命周期
//! 旧构建基线和生产构建共用上下文插入函数，比较不能证明两套独立实现的全部正确性。
//! 计时样例带默认忽略属性；其 release 提示不检查构建优化级别，未实际运行时不能作为性能结果。
//! 局部 fixture、代际和样本由测试作用域拥有，测试辅助函数不提供生产接口。
//!
//! # 参考
//! 参见 [`ActionEvaluationGeneration`]、[`InputActionMap`] 与 [`retired_from_action_map`]。
use std::borrow::Borrow;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;
use crate::input::{InputAction, InputActionContext, InputBinding, InputButton};

/// 计时 fixture 的动作与一一对应按钮绑定数量：4,096。
/// 此值只决定局部配置规模及构建工作量，不是运行时容量上限。
///
/// # 参考
/// 参见 [`benchmark_action_map`] 与 [`runtime56_recovery_batch_borrowed_binding_index_release_benchmark`]。
const BENCHMARK_BINDING_COUNT: usize = 4_096;
/// 计时样例对每个构建实现收集的样本数：11。
/// 每轮各测旧基线和当前构建一次，并交替先后顺序；此值需非零。
/// 按当前百分位算法，11 个样本的 P95 是最大样本。
///
/// # 参考
/// 参见 [`percentile_95`] 与 [`runtime56_recovery_batch_borrowed_binding_index_release_benchmark`]。
const BENCHMARK_SAMPLES: usize = 11;
/// 每个时长样本内连续构建并丢弃代际的次数：64。
/// 影响局部总计时区间，不能解释为运行时帧数或单次构建耗时。
///
/// # 参考
/// 参见 [`measure_generation`] 与 [`BENCHMARK_SAMPLES`]。
const BENCHMARK_ITERATIONS: usize = 64;

/// 对固定配置逐项比较当前借用键构建与旧拥有键基线的代际结果。
///
/// # 输入与断言范围
/// 局部配置显式声明启用的 `gameplay` 与禁用的 `menu`，另有无命名上下文的 `global.pause` 动作。
/// 绑定覆盖未知动作及同一动作的两条绑定；此 fixture 没有未声明的命名上下文，也没有轴绑定。
/// 断言比较上下文槽位、启用位、扁平下标、轴标志、源计数，以及每个动作的位置、上下文槽与绑定切片。
///
/// # 返回与生命周期
/// 正常返回 `()` 表示此输入的断言成立，失败会 panic；局部配置与两份拥有结果随后释放。
/// 两条路径共用 [`insert_context_slot`]，测试也共用结果类型；它不能证明其它输入或完整求值行为。
///
/// # 参考
/// 参见 [`ActionEvaluationGeneration::from_action_map`]、[`retired_from_action_map`] 与 [`CompiledAction::binding_indices`]。
#[test]
fn runtime56_recovery_batch_borrowed_binding_index_matches_retired_generation() {
    let action_map = InputActionMap {
        contexts: vec![
            InputActionContext::new("gameplay"),
            InputActionContext::new("menu").with_enabled(false),
        ],
        actions: vec![
            InputAction::new("gameplay.jump").with_context("gameplay"),
            InputAction::new("menu.confirm").with_context("menu"),
            InputAction::new("global.pause"),
        ],
        bindings: vec![
            InputBinding::button("menu.confirm", InputButton::KeyCode(1)),
            InputBinding::button("gameplay.jump", InputButton::KeyCode(2)),
            InputBinding::button("unknown.action", InputButton::KeyCode(3)),
            InputBinding::button("gameplay.jump", InputButton::KeyCode(4)),
        ],
    };

    let retired = retired_from_action_map(&action_map);
    let optimized = ActionEvaluationGeneration::from_action_map(&action_map);

    assert_eq!(optimized.context_slots, retired.context_slots);
    assert_eq!(optimized.context_enabled, retired.context_enabled);
    assert_eq!(optimized.binding_indices, retired.binding_indices);
    assert_eq!(optimized.has_axis_bindings, retired.has_axis_bindings);
    assert_eq!(optimized.source_binding_count, retired.source_binding_count);
    assert_eq!(optimized.actions.len(), retired.actions.len());
    for (optimized_action, retired_action) in optimized.actions.iter().zip(&retired.actions) {
        assert_eq!(optimized_action.action_index, retired_action.action_index);
        assert_eq!(optimized_action.context_slot, retired_action.context_slot);
        assert_eq!(
            optimized_action.binding_indices(&optimized),
            retired_action.binding_indices(&retired)
        );
    }
}

/// 对生产构建器的源码区段检查三个借用键相关的文本模式。
///
/// # 检查范围
/// 读取相邻 `generation.rs`，从构建器签名标记之后提取到 `actions` 方法签名之前。
/// 要求借用键树映射的文本存在，禁止当前指定的逐绑定克隆与拥有键映射文本；新增在 getter 前的文档也会进入该区段。
///
/// # 返回与生命周期
/// 正常返回 `()` 表示这三个文本检查成立；分隔标记缺失或断言失败会 panic。
/// 源码以 `include_str!` 静态嵌入，临时模式字符串只在函数作用域使用。
/// 该检查不执行构建器，不能证明分配次数、运行时行为或所有等价写法均被识别。
///
/// # 参考
/// 参见 [`ActionEvaluationGeneration::from_action_map`] 与 [`ActionEvaluationGeneration::actions()`]。
#[test]
fn runtime56_recovery_batch_generation_binding_index_borrows_action_keys() {
    let source = include_str!("../generation.rs");
    let implementation = source
        .split_once("pub(super) fn from_action_map")
        .expect("generation builder")
        .1
        .split_once("    pub(super) fn actions")
        .expect("generation builder end")
        .0;
    let cloned_binding_action = ["entry(binding.action", ".clone())"].concat();
    let owned_string_index = ["BTreeMap::<", "String, Vec<usize>>"].concat();

    assert!(
        !implementation.contains(&cloned_binding_action),
        "generation compilation must not clone one action string per binding"
    );
    assert!(
        !implementation.contains(&owned_string_index),
        "the temporary binding index must borrow action strings"
    );
    assert!(implementation.contains("BTreeMap::<&str, Vec<usize>>"));
}

/// 以固定配置交替采样旧基线与当前代际构建，并比较所选 P95 样本。
///
/// # 使用时机与约束
/// `#[ignore]` 使普通测试默认跳过此函数；属性说明建议显式使用 release 构建，函数本身不检查优化级别。
/// fixture 含 4,096 个独有动作及按钮绑定，每侧采集 11 个样本，每个样本连续构建并丢弃 64 份代际。
/// fixture 构造在计时区间外；样本包含代际构建、销毁及循环开销，11 样本的所选 P95 是各侧最大值。
///
/// # 返回、输出与失败
/// 返回 `()`，向标准错误打印时长及整数比例；日志中的克隆次数是固定描述，未由分配计数器测量。
/// 按代码中的饱和整数运算要求当前构建 P95 不超过基线的 85%，不满足时断言 panic。
/// 这只是本次实际运行环境的比较阈值；未运行的样例没有测得结果，也不保证其它机器达到阈值。
///
/// # 生命周期
/// 样本向量和局部配置在函数结束时释放。
///
/// # 参考
/// 参见 [`benchmark_action_map`]、[`measure_generation`]、[`percentile_95`] 与 [`retired_from_action_map`]。
#[test]
#[ignore = "release-only performance evidence"]
fn runtime56_recovery_batch_borrowed_binding_index_release_benchmark() {
    let action_map = benchmark_action_map();
    let mut retired_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            retired_samples.push(measure_generation(|| retired_from_action_map(&action_map)));
            optimized_samples.push(measure_generation(|| {
                ActionEvaluationGeneration::from_action_map(&action_map)
            }));
        } else {
            optimized_samples.push(measure_generation(|| {
                ActionEvaluationGeneration::from_action_map(&action_map)
            }));
            retired_samples.push(measure_generation(|| retired_from_action_map(&action_map)));
        }
    }

    let retired_p95 = percentile_95(&mut retired_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / retired_p95.as_nanos().max(1),
    );
    eprintln!(
        "RUNTIME56_BORROWED_ACTION_BINDING_INDEX_BENCH_V1 \
samples={BENCHMARK_SAMPLES} iterations={BENCHMARK_ITERATIONS} \
bindings={BENCHMARK_BINDING_COUNT} retired_binding_action_clones_per_build=4096 \
optimized_binding_action_clones_per_build=0 retired_p95_ns={} optimized_p95_ns={} \
reduction_basis_points={reduction_basis_points}",
        retired_p95.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= retired_p95.as_nanos().saturating_mul(85),
        "borrowed action binding index must reduce generation-build P95 by at least 15%: \
retired={retired_p95:?}, optimized={optimized_p95:?}"
    );
}

/// 为构建计时样例创建具有长动作名称的按钮配置。
///
/// # 返回
/// 返回拥有 [`BENCHMARK_BINDING_COUNT`] 个唯一动作、对应单按钮绑定且没有上下文的 [`InputActionMap`]。
/// 每个名称含零填充序号与 96 个填充字符，按钮键码来自该序号；此规模只用于局部计时 fixture。
///
/// # 生命周期
/// 返回值拥有动作和绑定字符串及向量，由调用方跨采样只读复用；构造此 fixture 的耗时不在 [`measure_generation`] 的区间内。
///
/// # 参考
/// 参见 [`InputAction`]、[`InputBinding::button`] 与 [`runtime56_recovery_batch_borrowed_binding_index_release_benchmark`]。
fn benchmark_action_map() -> InputActionMap {
    let mut actions = Vec::with_capacity(BENCHMARK_BINDING_COUNT);
    let mut bindings = Vec::with_capacity(BENCHMARK_BINDING_COUNT);
    for index in 0..BENCHMARK_BINDING_COUNT {
        let action = format!("gameplay.action.{index:04}.{}", "x".repeat(96));
        actions.push(InputAction::new(action.clone()));
        bindings.push(InputBinding::button(
            action,
            InputButton::KeyCode(index as u32),
        ));
    }
    InputActionMap {
        contexts: Vec::new(),
        actions,
        bindings,
    }
}

/// 连续调用构建闭包并取得整个循环的经过时长。
///
/// # 参数
/// `build` 是可变、无参数并返回 [`ActionEvaluationGeneration`] 的闭包，共调用 [`BENCHMARK_ITERATIONS`] 次。
/// 闭包的副作用和失败由其实现决定；本函数不等待闭包另行启动的后台工作。
///
/// # 返回与失败
/// 返回从循环开始到结束的 [`Duration`]，包含闭包执行、每份结果经 `black_box` 后的销毁及循环开销。
/// 它是整批迭代耗时，不能直接当作单次构建耗时；闭包 panic 会中断采样而不返回时长。
///
/// # 生命周期
/// 闭包由本函数拥有且只在局部调用，每份返回代际在对应迭代语句结束时释放；返回时长不借用闭包或代际。
///
/// # 参考
/// 参见 [`Instant`] 与 [`runtime56_recovery_batch_borrowed_binding_index_release_benchmark`]。
fn measure_generation(mut build: impl FnMut() -> ActionEvaluationGeneration) -> Duration {
    let started = Instant::now();
    for _ in 0..BENCHMARK_ITERATIONS {
        black_box(build());
    }
    started.elapsed()
}

/// 用拥有动作名称键的临时索引生成用于比较与计时的旧构建基线。
///
/// # 参数
/// `action_map` 提供只读源配置；比较时应与当前生产构建使用相同内容。
///
/// # 返回与约束
/// 返回按源动作顺序构建的 [`ActionEvaluationGeneration`]，仍须与该源映射配对使用。
/// 函数对每个源绑定执行名称克隆，再由 [`finish_generation`] 完成结果；它不校验配置。
/// 上下文步骤调用与生产构建相同的 [`insert_context_slot`]，因此基线不能作为独立上下文实现的证明。
///
/// # 生命周期
/// 临时 `String` 键表在本函数结束时释放；返回代际拥有最终上下文键和整数索引，不借用临时表或源字符串。
///
/// # 参考
/// 参见 [`InputActionMap`]、[`finish_generation`] 与 [`ActionEvaluationGeneration::from_action_map`]。
fn retired_from_action_map(action_map: &InputActionMap) -> ActionEvaluationGeneration {
    let mut context_slots = BTreeMap::new();
    let mut context_enabled = Vec::new();
    for context in &action_map.contexts {
        insert_context_slot(
            &mut context_slots,
            &mut context_enabled,
            &context.id,
            context.enabled,
        );
    }
    for action in &action_map.actions {
        if let Some(context) = action.context.as_deref() {
            insert_context_slot(&mut context_slots, &mut context_enabled, context, true);
        }
    }

    let mut bindings_by_action = BTreeMap::<String, Vec<usize>>::new();
    for (index, binding) in action_map.bindings.iter().enumerate() {
        bindings_by_action
            .entry(binding.action.clone())
            .or_default()
            .push(index);
    }

    finish_generation(
        action_map,
        context_slots,
        context_enabled,
        &bindings_by_action,
    )
}

/// 用预先准备的上下文表及泛型键绑定索引表完成测试代际。
///
/// # 参数与约束
/// `action_map` 提供原动作顺序、源轴存在性和绑定条目数。
/// `context_slots` 与 `context_enabled` 按值转入结果，须属于同一配置构建，且槽号与启用位位置对应。
/// `bindings_by_action` 借用动作 id 到源绑定下标组的表；组内下标及顺序须对应同一映射。
/// `K` 须实现 `Borrow<str> + Ord`，其借用字符串排序应与键排序一致，以支持按动作名称查询。
/// 函数不验证这些配对或下标；无匹配产生空范围，重复动作 id 仍可重复拼接同一组。
///
/// # 返回与生命周期
/// 返回拥有上下文表、启用位、动作记录和扁平下标的 [`ActionEvaluationGeneration`]。
/// 轴标志与测试源计数取自全部源绑定，包含未匹配动作的绑定；结果不保留源映射、泛型键或索引表的借用。
/// 此函数复现生产构建的结果形状供旧基线使用，比较结果不能据此推导完整求值行为。
///
/// # 参考
/// 参见 [`Borrow`]、[`Ord`]、[`CompiledAction`] 与 [`retired_from_action_map`]。
fn finish_generation<K>(
    action_map: &InputActionMap,
    context_slots: BTreeMap<String, usize>,
    context_enabled: Vec<bool>,
    bindings_by_action: &BTreeMap<K, Vec<usize>>,
) -> ActionEvaluationGeneration
where
    K: Borrow<str> + Ord,
{
    let mut actions = Vec::with_capacity(action_map.actions.len());
    let mut binding_indices = Vec::with_capacity(action_map.bindings.len());
    for (action_index, action) in action_map.actions.iter().enumerate() {
        let binding_start = binding_indices.len();
        if let Some(indices) = bindings_by_action.get(action.id.as_str()) {
            binding_indices.extend(indices.iter().copied());
        }
        let binding_end = binding_indices.len();
        actions.push(CompiledAction {
            action_index,
            context_slot: action
                .context
                .as_deref()
                .and_then(|context| context_slots.get(context).copied()),
            binding_start,
            binding_end,
        });
    }

    ActionEvaluationGeneration {
        actions,
        binding_indices,
        context_slots,
        context_enabled,
        has_axis_bindings: action_map
            .bindings
            .iter()
            .any(|binding| !binding.axes.is_empty()),
        source_binding_count: action_map.bindings.len(),
    }
}

/// 原地排序时长样本并取得按秩选择的第 95 百分位值。
///
/// # 参数
/// `samples` 必须非空，且 `samples.len() * 95` 应能由 `usize` 表示；调用方接受其顺序被升序排序。
///
/// # 返回与失败
/// 按 `ceil(len * 95 / 100) - 1` 的饱和减法索引返回 [`Duration`] 副本；空切片会在索引处 panic。
/// 当前 [`BENCHMARK_SAMPLES`] 为 11，调用中的所选值就是最大样本；不做插值或生成额外样本。
///
/// # 生命周期
/// 修改仅作用于传入切片顺序，返回值不借用该切片。
///
/// # 参考
/// 参见 [`BENCHMARK_SAMPLES`] 与 [`runtime56_recovery_batch_borrowed_binding_index_release_benchmark`]。
fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() * 95).div_ceil(100).saturating_sub(1)]
}
