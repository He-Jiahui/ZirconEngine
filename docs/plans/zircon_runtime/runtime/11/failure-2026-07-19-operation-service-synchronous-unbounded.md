---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: operation-service-synchronous-unbounded
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/11
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/operation
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime_interface/src/runtime_api/operation.rs
tests:
  - bounded operation storm and result retention
  - panic/cancel/deadline terminal-state parity
---

# Runtime11：operation service同步执行与无界驻留

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-435 operation service performance and retention audit
- 修复责任计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：operation dispatch, bounded task ownership, and scheduler integration belong to the Runtime11 unified JobSystem boundary.

## 失败现象与复现证据

第二次poll在caller线程同步执行完整handler并持`&mut World`，dynamic API上层同时持session mutex。task/payload/result未harvest时无capacity/bytes/TTL；没有cancel/deadline，panic后`execution_started=true`且task永久Running。progress每poll还分配message String并跨ABI JSON。

## 最低共享层根因

V1 operation transport combines polling, JSON progress allocation, task execution, and result retention in one caller-owned path. It has no typed ticket state, deadline/cancel/TTL contract, pre-apply result reservation, or byte-bounded payload transport, so Runtime11 alone cannot make the full lifecycle authoritative while V1 remains the public ABI.

## 架构修复验收

- pure prepare进入Runtime11统一bounded pool；World mutation通过有预算、确定序的owner-thread apply，禁止operation私建线程。
- queue/task/result有count+bytes上限、deadline/cancel/TTL；panic必转Failed，unharvest按期回收且可观测。
- Runtime10联动phase id/共享detail的progress ABI，stable poll String/JSON=0。
- 1/1k/100k ops、0/10/1000ms、0/1/64MiB payload/result记录caller work=0、session锁不跨handler、queue/RSS有界、World/error/harvest语义等价；回传PERF-MVP-435。

## 禁止临时方案

- 不得恢复由 poll 驱动的同步 handler 执行，或为 V1 建立私有 operation worker thread。
- 不得以测试专用分支、无界 JSON/结果保留、静默截断或重试掩盖 cancellation、deadline、TTL、World 与 harvest 语义。

## 修复结果与回传

Open state (2026-08-02 source progress): Runtime11 now owns a bounded `queued -> owner snapshot -> worker pure prepare -> owner apply` lifecycle. Runtime10 has hard-cut the public poll surface to V2 fixed-layout status, so poll has no handler, JSON, or owned-buffer work; `submit_json` reserves count and raw bytes before JSON decoding. A worker returns both an apply command and terminal result, and the completion drain reserves their combined bytes before any `World` mutation. Apply consumes only the command on the owner thread. Cancel, deadline, terminal-result TTL, prepare panic, owner-apply panic, raw admission rollback, and tombstone eviction are all explicit status transitions with exact checked accounting.

Worker completion ownership is batch-scoped: only workers of a dispatch batch retain its sender. The service records the corresponding receiver and, when it closes, resolves only outstanding tasks from that batch. Each task carries `prepare_in_flight`, so a closed batch changes still-preparing tasks to `Failed` with `WorkerChannelLost`, while already-cancelled or expired tasks keep their terminal phase and all outstanding capacity slots are released exactly once. The direct unit fixture uses a closed `sync_channel` to drive the production completion drain, and source guards retain the no-owner-bake, pre-apply result reservation, exact-release, and fixed-handler-boundary invariants. The operation source was also split into admission, completion, maintenance, and source-guard modules; every touched Rust file is below the 800-line convention budget.

This handoff remains `open`. The navigation Recast bake path is owned by Plugins05 and still must be projected to worker-pure bake plus generation-checked owner publish under [its canonical failure handoff](../../../zircon_plugins/05/failure-2026-07-19-navigation-runtime-fallback-hotpath.md); Runtime11 must not add a navigation fallback or compatibility branch. The required source-bound managed 1/1k/100k operation matrix has not yet supplied terminal evidence, so neither this record nor PERF-MVP-435 may be returned as fixed or green.
