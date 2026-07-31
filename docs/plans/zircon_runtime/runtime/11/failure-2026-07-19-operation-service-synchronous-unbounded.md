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

Open state: Runtime11 已将 V1 `poll` 改为快照读取，并把纯 `prepare` 投递到受限 scheduler、把 `apply` 固定到 `RuntimeDynamicSession::tick_frame` 的 owner thread；提交 admission 现在受 task count 和 JSON retained-byte 上限约束，worker/apply panic 转为可 harvest 的失败终态。受管 `cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1` 已以 reservation `1dece3f01ab74ff8b1ddcbd040a5fb90`、job `792f8a5fa4f146b4952847d756c99778`、run `fec70895dfe34049b36e3a306d3e696e` 自然终止 `exit 101`。原始 stderr 的本切片 `E0499` 位于 `operation/service.rs:242`，根因是借用 `task.handler` 的元组表达式跨越 `state.in_flight_prepares` 更新；已改为局部作用域提取 handler/payload 后递增计数，保持相同调度事务顺序。随后 Native Refresh focused job `87a16b87c245479daffe95b5a3d727fe` 的 lib-test compile 还暴露 operation 测试的 `E0282`、`E0422` x2 与 `E0624` x2：poll-loop 输出现已显式标为 `ZrRuntimeOperationProgressV1`，而 `RuntimeOperationLimits` 与 `with_limits` 仅以 `pub(super)` 暴露给 `operation::tests`，并由 sibling `service` 显式导入，未扩大生产 API。两个 job 共见的 `E0365`/`E0603` 是 Runtime08 `scene/world/compiled_binding/{mod,generation}.rs` 的 `SceneBindingGenerations` 可见性边界；Text09 `text/parallel/raster_pool.rs:447` 的 `E0382` 也是独立编译阻断。新的 operation 验证必须使用 fresh reservation，且仅在 Runtime08 node `1122887` 与既有 Text09 raster-queue failure 返回后提交。

Independent review on 2026-07-29 found two unresolved architectural blockers, so no pass is claimed: (1) V1 result-byte rejection can occur only after a handler has mutated `World`, which could report failure after an observable operation and invite an unsafe retry; (2) `serde_json::Value` prepare and FFI deserialization allocate before the current retained-byte admission can reject them. Runtime10 must publish the V2 phase/detail and operation-ticket contract before Runtime11 can make cancellation, deadline, TTL, pre-apply result reservation, and byte-bounded transport authoritative. This handoff remains `open`; it must not be returned until those invariants and the declared 1/1k/100k upward gates have managed evidence.

Current validation state: Runtime10 already owns open node `operation-phase-detail-abi-owner-thread-apply`; Runtime11 must not add a V1 compatibility branch while that hard cut is pending. The pending check reservation `a2a7574c4c41465fb4efdff7ea5cd517` is preserved and renewed in place, but its durable warm-lane ledger head is rejected by consume admission without a job. Coordinator01 node `1126994` (`cpu-reservation-ledger-consume-fifo-divergence`) now owns that lower control-plane repair. No retry or Cargo result is claimed until the Coordinator return and immutable source-binding recheck.
