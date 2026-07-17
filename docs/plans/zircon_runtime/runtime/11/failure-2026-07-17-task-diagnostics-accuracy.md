---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: task-diagnostics-accuracy
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/tasks/job_handle.rs
  - zircon_runtime/src/core/runtime/tasks/diagnostics.rs
  - zircon_runtime/src/core/runtime/tasks/report.rs
tests:
  - detached_spawn_counts_panicked_tasks_as_completed
  - worker-side wait does not contaminate main-thread wait
  - 1/2/N worker queue-delay pressure matrix
---

# Runtime11：任务完成率与主线程等待诊断不准确

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：F0 runtime task scheduler/module/test 逐文件静态审查
- 来源证据：`docs/plans/performance/01/2026-07-17-task-system-static-review.md`
- 修复责任计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：任务终态、等待身份和队列指标属于 Runtime11 模型；上层 runtime/editor 无法可靠推断。

## 失败现象与复现证据

detached `spawn` 原先在任务正常返回后才计 completed，panic 会永久污染 backlog 差值。性能 Session 已用 unwind-safe completion guard 修复并补回归，待 Cargo 验证。

`JobHandle::wait()` 没有 caller identity，却把任意线程等待都计入 `tasks.main_thread_wait_ms`。此外任务模型未暴露 queued、active、lag 和 panic/cancel 分项，主线程异常堆积无法由诊断闭环。

## 最低共享层根因

任务终态与 caller/queue identity 未在 scheduler/handle 层表达；任何上层计数都只能重复猜测，且会在多个消费者之间漂移。

## 架构修复验收

- 接收 detached panic 完成计数修复，保持 Rayon panic 行为不变。
- 显式标记 main-thread identity 后只统计主线程，或硬切换为语义准确的 `explicit_wait_ms` 并迁移消费者。
- 与 Runtime07 增加低成本 queued/active/lag/panicked/cancelled 指标及 1/2/N worker 压测。

## 禁止临时方案

- 不得吞掉 detached panic 来让计数好看。
- 不得继续把所有线程 wait 标成主线程，或用累计差值冒充实时队列。

## 修复结果与回传

Open state: `detached 完成 guard 已实现待验证；wait/queue 指标待 Runtime11/07 修复`。
