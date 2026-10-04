# 2026-07-17 MVP ECS schedule 静态审查

## 范围与状态

- 已逐文件读取 `schedule.rs`、`schedule_stage_plan.rs`、`schedule_conflict_graph.rs` 与 `schedule_parallel_executor.rs`。
- 这是 `zircon_runtime/src/scene/ecs` 126 个 Rust 文件中的首个调度切片，不代表整个 ECS 目录通过验收；Cargo、分配计数和当前源码产品帧 trace 尚待完成。
- 对照读取 Bevy `dev/bevy/crates/bevy_ecs/src/schedule/executor/{mod,multi_threaded,single_threaded}.rs` 的 executor metadata、reusable bitset 与 ready/running/completed 状态设计。

## 已确认事实

### schedule build 已与逐帧执行分离

`Schedule` 用 `Arc<SceneScheduleStagePlan>` 缓存 stage 分组和拓扑顺序，仅在注册/移除系统且没有被 take 的系统时重建；执行时不重复完成全量 stage scan 和 topological sort。这是正确的 MVP 热路径边界。

`ScheduleConflictGraph` 的同 stage pairwise conflict 检查是 O(n²)，`topological_stage_order` 的 ready-node 选择也会 O(n²) 扫描，并对 outgoing edge 做线性去重；但这些当前属于 schedule definition mutation/build 路径。先用 100/1000 system build benchmark 确认编辑器动态重建成本，再决定是否引入 heap/bitset/index，不能把冷构建算法误报成每帧瓶颈。

### 每个 batch 曾复制完整 task registry

`run_batches_with_report` 为满足 `'static` worker closure，每个 batch 都 clone `ScheduleParallelTaskRegistry`。原实现的 clone 会复制整个 `HashMap<String, Arc<Task>>`，因此一帧有 B 个 batch、N 个注册任务时会重复复制 B 次 N-entry map。

已把 registry 改为 `Arc<HashMap<...>>` copy-on-write snapshot：执行期 clone 只增加引用计数，后续 `register` 通过 `Arc::make_mut` 与已调度 snapshot 隔离。回归 `cloned_task_registry_shares_frozen_task_map_until_mutated` 已写入，待 Cargo 验证。

### 每帧仍有 batch 级分配

executor 每次运行还会创建一个 abort `Arc<AtomicBool>`，并为每个 batch 分配 `Arc<Mutex<Option<Result>>>`、复制 `system_ids` 字符串向量、创建依赖 `JobHandle`。这些操作保持 failure order 和 worker 调度语义，但对小系统 batch 可能比系统工作本身更贵。

Bevy 的参考 executor 缓存 system task metadata、dependency counts 和多组 reusable bitsets，并在每次 run 重置容量内状态。Zircon 不需要照搬其复杂度，但 Runtime03/08/11 应先做 1/10/100 batch 的分配次数与空任务延迟基线，再决定把 batch execution slots/IDs 预编译进 stage plan，或保留当前实现。

## 验收计划

1. 运行 registry COW 回归及现有 `ecs_schedule` 行为/结构测试。
2. 以空任务和代表性系统分别测 1/10/100 batch 的总分配、p50/p95 与 scheduler queue delay。
3. 以 100/1000 system 测 schedule build/conflict/topological 时间，区分冷构建与稳定帧。
4. 当前源码 F2 trace 证明 schedule executor 在 frame top hotspots 中的位置后，再处理剩余 batch allocations。

