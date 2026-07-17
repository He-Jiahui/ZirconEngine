---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: schedule-executor-frame-allocations
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/ecs/schedule_stage_plan.rs
  - zircon_runtime/src/scene/ecs/schedule_conflict_graph.rs
tests:
  - cloned_task_registry_shares_frozen_task_map_until_mutated
  - 1/10/100 empty-task batch allocation and latency matrix
  - 100/1000-system schedule-build benchmark
---

# Runtime03：ECS schedule executor 的逐 batch 注册表复制与分配

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F2 ECS schedule 四文件静态审查与 Bevy executor 对照
- 修复责任计划：`docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md`
- 交接原因：batch 编译、执行状态复用和 frame schedule 的成本契约属于 Runtime03，并与 Runtime08/11 共同验收。

## 失败现象与复现证据

`run_batches_with_report` 原先为每个 batch 深 clone 完整 `HashMap<String, Arc<Task>>`。性能 Session 已把 registry 改为 `Arc<HashMap>` copy-on-write snapshot，使执行 clone 为 O(1)，并增加 mutation isolation 回归，等待 Cargo 验证。

每次 run 仍创建 abort `Arc<AtomicBool>`，每 batch 创建 result `Arc<Mutex<Option<Result>>>`、复制 system-id vector 并创建依赖 handle。对空任务或极短系统，这些控制面分配可能主导帧成本，当前没有分配/延迟基线。

## 最低共享层根因

stage plan 缓存了拓扑与 batch，但没有缓存可复用的 batch execution slots/稳定 ID ownership；executor 因 `'static` worker closure 在每帧重新构造所有权。该契约必须在 schedule owner 解决，不能让各系统 caller 自建任务表。

## 架构修复验收

- 接收 registry COW 修复并通过现有 parallel/serial/error-order 测试。
- 测量 1/10/100 空任务与代表性系统 batch 的 allocations、p50/p95 和 queue delay。
- 只有证据显示控制面显著时，才把 IDs/result slots 或 task metadata 预编译进可复用 stage execution state。
- 100/1000 system build benchmark 单独报告 conflict/topological 冷构建成本，不与稳定帧混算。

## 禁止临时方案

- 不得回退为主线程全串行来消除分配。
- 不得破坏 batch 错误顺序、abort、deferred command 或 schedule mutation snapshot 语义。

## 修复结果与回传

Open state: `registry COW 已实现待验证；其余逐 batch 分配待 Runtime03/08/11 测量后处理`。

