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

Runtime11 当前实现已保留 Performance01 的 detached unwind-safe completion guard，并完成最低 scheduler owner 的诊断硬切：

- 删除不能证明 caller identity 的 `tasks.main_thread_wait_ms`，统一为 `tasks.explicit_wait_ms`，不保留 alias 或双计数。
- dependency waiting / ready queue / worker start / terminal 转换分别导出 `tasks.dependency_waiting`、`tasks.queued`、`tasks.active`、`tasks.queue_wait_ms` / `tasks.queue_wait_samples`，并保持 `scheduled = completed + dependency_waiting + queued + active`。
- task panic 与 dependency-cancelled-before-launch 分别维护 `tasks.panicked`、`tasks.cancelled`；cancelled task 不进入 queued/active。
- combined handle 在记录首个 child panic 后仍等待全部 children 进入 terminal，再传播该 panic；`wait_all` 不会提前返回或低估显式同步耗时。
- 重叠 diagnostics writer 通过 acquire/release retirement chain 发布完整 payload；dependency continuation 逐项 containment，先释放全部后续 barrier callback 与 observer，再重抛首个 continuation panic。
- 新增 queue saturation、1/2/4-worker pressure matrix、并发 lifecycle 守恒、worker wait、panic/cancel 与 detached panic 行为测试；hotpath 只使用原子计数，不增加诊断锁。
- 非 Cargo JobSystem audit 已通过 1/1；受管 focused Cargo 与独立复审仍待完成。

Open state: `Runtime11 code complete / managed focused validation and Performance01 fixed return pending`。
