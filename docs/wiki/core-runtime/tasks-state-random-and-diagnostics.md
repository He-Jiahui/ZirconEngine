---
related_code:
  - zircon_runtime/src/core/runtime/tasks/mod.rs
  - zircon_runtime/src/core/runtime/tasks/task_graph/mod.rs
  - zircon_runtime/src/core/runtime/state_machine/mod.rs
  - zircon_runtime/src/core/runtime/random/mod.rs
  - zircon_runtime/src/core/runtime/diagnostics/mod.rs
  - zircon_runtime/src/core/runtime/runtime.rs
implementation_files:
  - zircon_runtime/src/core/runtime/tasks
  - zircon_runtime/src/core/runtime/state_machine
  - zircon_runtime/src/core/runtime/random
  - zircon_runtime/src/core/runtime/diagnostics
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
tests:
  - zircon_runtime/src/core/runtime/tasks
  - zircon_runtime/src/core/runtime/random/tests
  - zircon_runtime/src/core/runtime/state_machine
  - zircon_runtime/src/tests/runtime_diagnostics
doc_type: module-detail
status: current
---

# 任务、状态、随机数与诊断

## Runtime-owned Task Graph

每个 `CoreRuntime` 创建一个 `EngineTaskGraph`，并由该 runtime 独占 worker budget。多个 runtime 同进程时，应使用 `try_with_task_graph_options` 显式规划 worker，不能各自采用默认值后假设不会超订阅。

### Scope

子系统通过 `TaskGraphScopeDescriptor` 命名任务所有者并设置 active task 容量（默认 1024）。`TaskGraphScope` 是提交、取消、排空和统计的责任边界。

```rust
use zircon_runtime::prelude::{
    TaskDescriptor, TaskGraphScopeDescriptor, TaskId, TaskPoolKind,
};

let scope = runtime.create_task_graph_scope(
    TaskGraphScopeDescriptor::new("asset.import")
        .with_task_capacity(128),
)?;

let handle = scope.submit(
    TaskDescriptor::new(TaskId::new(1), TaskPoolKind::Compute, "asset-import"),
    |cancel| {
        if cancel.is_cancellation_requested() {
            return;
        }
        // bounded background work
    },
)?;

scope.close_admission();
let drained = scope.wait_until_quiescent(
    std::time::Duration::from_secs(2),
);
```

`TaskDescriptor` 的具体构造器和字段应以源码为准；上例展示 scope API 形状。`submit_after` 可在同一个 graph 的 canonical `TaskHandle` 全部成功完成后启动任务；跨 graph dependency 会被拒绝。

### Census 与 shutdown

`scope.census()` 返回 owner、capacity、accepting、submitted、queued、running、completed、failed、cancelled。最后一个 scope clone Drop 时自动关闭 admission；稳定停机仍建议显式关闭并等待 quiescent。

`runtime.shutdown_task_graph(timeout)` 关闭全图 scope admission 并等待 census。失败会使 graph 保持 closing，不代表可以继续像正常运行时一样提交。

### 辅助执行能力

Core 还公开 `JobScheduler`、typed task pool、`parallel_for`/`parallel_map_indices`、bounded keyed I/O、bounded stream I/O、retained byte budget、timer 与任务诊断 observation。业务优先选最窄、带容量和 shutdown 语义的抽象，不应为模块另建无监管线程池。

## Runtime-wide State Machine

任何满足 `'static + Send + Sync + Clone + PartialEq + Eq + Hash + Debug` 的类型自动实现 `StateSpec`。Core 保存当前 `State<T>`、待提交 `NextState<T>` 和最新 transition event。

```rust
use zircon_runtime::core::{OnEnter, OnExit};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum GameState { Loading, Playing }

runtime.insert_state(GameState::Loading);
runtime.register_on_enter(OnEnter(GameState::Playing), |event| {
    println!("entered: {:?}", event.entered);
});
runtime.register_on_exit(OnExit(GameState::Loading), |_event| {});

runtime.set_next_state(GameState::Playing);
let transition = runtime.apply_state_transition::<GameState>();
```

还可用 `OnTransition<T>` 订阅任意该类型的转换，使用 `set_next_state_if_neq` 避免重复排队，使用 `reset_next_state` 撤销待转换状态。

State machine 是 runtime-wide 控制状态，不替代 ECS component state、动画状态机或 UI 局部状态。频繁实体级状态应留在 World/ECS。

## 确定性随机数

`RandomService` 是 runtime 实例的 master seed authority 和唯一 stream registry。构造选择：

- 默认随机服务。
- `with_random_seed(master_seed)` 确定性新会话。
- `with_random_service_state` 恢复 seed authority，但不恢复流进度。
- `with_random_service_checkpoint` 恢复 seed 与已登记流的完整进度。

子系统应按稳定 stream identity 派生/租用随机流，避免多个系统共享一个全局可变 RNG 导致执行顺序改变结果。checkpoint generation 和 stream ownership 的具体接口见 `core/runtime/random` 源码。

## Diagnostic Store

`DiagnosticStore` 以 `DiagnosticPath` 存储按帧测量序列。`record_diagnostic` 接收 frame index、数值、可选单位和 subsystem tags；`diagnostic_store_snapshot` 生成一致的只读快照供 editor/tooling 使用。

```rust
runtime.record_diagnostic(
    "gameplay.active_entities",
    frame_index,
    active_entities as f64,
    Some("entities"),
    ["gameplay", "scene"],
);

let snapshot = runtime.diagnostic_store_snapshot();
```

核心内建的时间和任务路径包括 `time.frame_count`、`time.frame_time`、`time.fps`，以及 queued/active/completed/cancelled/panicked、queue wait、dependency wait、execution timing 等 task 指标。

### Profiling

`core::runtime::diagnostics::profiling` 提供 scope/frame/counter capture、热点分析与导出。实际采集能力可能受 feature（例如 Tracy 集成）和 runtime 配置影响；调用者应检查 `profiling_feature_enabled()` 和 recorder status，不应假定部署构建一定启用 profiler。

### Runtime diagnostics snapshots

诊断域还聚合 render、animation、physics/backend、devtools module/service/plugin catalog 等只读 snapshot。Editor 和工具面板应消费 snapshot，而不是直接锁内部 registry 或渲染状态。

进程级日志 sink、环境变量过滤和崩溃 flush 另见[诊断日志](diagnostic-log.md)；本页的 `DiagnosticStore` 是可观测数据源，不等同于文本日志输出。

## 限制与实现状态

- Task graph 已覆盖显式 scope 工作，但源码声明 process timer 和剩余 private worker 尚未完全 graph-owned。
- Drop scope 会关闭 admission，但不能替代产品级 deadline 和错误处理。
- State hooks 在运行时范围共享，hook 内避免长时间阻塞或递归状态提交。
- 随机确定性依赖调用方使用稳定流身份，并正确保存完整 checkpoint。
- 任务、状态、随机、诊断和 profiling contract 均已实现；不同产品 feature 下的可用采集后端不同。
