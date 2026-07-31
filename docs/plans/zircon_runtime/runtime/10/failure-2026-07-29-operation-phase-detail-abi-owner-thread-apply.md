---
handoff_kind: failure
status: open
created_at: 2026-07-29
summary_slug: operation-phase-detail-abi-owner-thread-apply
origin_plan: docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
fixing_plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
origin_child_dir: docs/plans/zircon_runtime/runtime/11
fixing_child_dir: docs/plans/zircon_runtime/runtime/10
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/runtime_api/operation.rs
  - zircon_runtime/src/dynamic_api/session/operation.rs
  - zircon_runtime/src/operation
  - zircon_runtime/src/navigation/operation/handler.rs
tests:
  - bounded operation status ABI layout and no-allocation poll contract
  - worker prepare and owner-thread apply terminal-state parity
  - cancellation, deadline, TTL, panic, and count-plus-byte budget matrix
---

# Runtime10: Operation Phase/Detail ABI and Owner-Thread Apply Contract

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 来源执行切片：Runtime11 operation service bounded worker and terminal-state recovery.
- 修复责任计划：`docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md`
- 交接原因：operation poll 的跨 ABI phase/detail 表示与 session/world owner-thread 边界属于 Runtime10 dynamic API contract；Runtime11 不得在 service 内私自扩展或保持双版本 ABI。

## 失败现象与复现证据

`RuntimeOperationService::poll` 在 FFI `with_session` 闭包和 `level.with_world_mut` 内直接调用 `RuntimeOperationHandler::execute`。当前 handler 接收 `RuntimeOperationContext`，其中包含 `&mut World`；第二次 poll 因此执行完整业务工作、持有 session/world 锁，并把 `ZrRuntimeOperationProgressV1` 的 `String` 经 `serde_json::to_vec` 写回 owned byte buffer。

现行 V1 没有可由 Runtime11 worker 和 Runtime10 FFI 共同消费的 phase/detail DTO，也没有能表达 cancel、deadline、TTL、预算拒绝或 worker panic 的稳定终态。将现有 handler 直接提交到线程池会把 `&mut World` 跨线程移动，既不安全也改变 navigation bake/restore 的 owner-thread 语义。

## 最低共享层根因

Runtime10 只公开了 JSON-shaped `ZrRuntimeOperationProgressV1`，而 Runtime11 task owner 只有同步 `execute(context, payload)` callback。缺少一个 hard-cut 的 fixed-layout status ABI，以及“owner-thread snapshot/apply / worker pure prepare”两阶段协议；因此 Runtime11 无法实现 plan-required 的 bounded asynchronous execution，而不让 poll 承担 handler 工作或复制可变 World。

## Runtime10 ABI wire contract

Runtime10 必须在 `zircon_runtime_interface` 定义并在当前 API table 中唯一导出以下状态 out-param；字段顺序和 wire 值属于 ABI，不得以 Rust enum layout、serde tag 或 pointer 表示代替：

```rust
#[repr(C)]
pub struct ZrRuntimeOperationStatusV2 {
    pub abi_version: u32,     // exactly 2
    pub phase: u32,           // values listed below
    pub detail_kind: u32,     // values listed below
    pub reserved: u32,        // always zero
    pub handle: u64,
    pub completed_work: u64,
    pub total_work: u64,
    pub detail_value: u64,
}

pub type ZrRuntimePollOperationFnV2 = unsafe extern "C" fn(
    ZrRuntimeSessionHandle,
    ZrRuntimeOperationHandle,
    *mut ZrRuntimeOperationStatusV2,
) -> ZrStatus;
```

`ZrRuntimeOperationStatusV2` is exactly 48 bytes, contains no pointer or owned allocation, and its zeroed `reserved` field is reserved for a later atomically-versioned ABI. The current table replaces the V1 JSON `poll_operation` slot with `ZrRuntimePollOperationFnV2`; no V1 alias, forwarder, or parallel table may remain after the Runtime10/Runtime11/Runtime03/App/Editor cutover.

| `phase` value | Meaning | Terminal |
|---:|---|---|
| 1 | queued; admitted but not snapshot-ready | no |
| 2 | preparing; immutable input is owned by the bounded pool | no |
| 3 | ready-to-apply; worker output awaits the runtime tick budget | no |
| 4 | completed; bounded harvest result is retained | yes |
| 5 | failed; bounded failure result is retained | yes |
| 6 | cancelled; no apply is permitted | yes |
| 7 | expired; deadline or result-retention TTL has removed executable/result data | yes |
| 8 | harvested; the result was consumed and only a metadata tombstone remains | yes |

| `detail_kind` value | `detail_value: u64` meaning |
|---:|---|
| 0 none | always `0` |
| 1 queue-depth | current queued ticket count |
| 2 admission-count-limit | configured maximum admitted ticket count |
| 3 admission-byte-limit | byte count that failed admission or is currently retained |
| 4 deadline-elapsed | elapsed monotonic milliseconds since the ticket deadline |
| 5 cancelled | always `0` |
| 6 worker-panic | always `0`; diagnostic text belongs only to terminal harvest/diagnostics |
| 7 owner-apply-failed | stable owner error code, or `0` when no code exists |
| 8 terminal-ttl-elapsed | elapsed monotonic milliseconds since terminal result creation |
| 9 harvested | original terminal `phase` value (4 through 7) |
| 10 worker-channel-lost | always `0`; the bounded pool completion channel closed before the ticket reached a terminal result |

`poll_operation_status_v2` returns `InvalidArgument` and leaves the destination untouched for a null destination or invalid zero handle. It returns `NotFound` and leaves the destination untouched for an unknown handle or a metadata tombstone that has been deterministically evicted. Every live task and retained tombstone returns `Ok` and writes the 48-byte status DTO. Therefore unknown is an FFI outcome, not a fake phase value; harvested is a bounded observable phase.

`harvest` consumes only phases 4 or 5, atomically replaces the retained payload with a phase-8 tombstone, and rejects a second harvest with `Error` plus static `operation already harvested` diagnostics. It rejects phase 6 or 7 with `Error` and static `operation cancelled` or `operation result expired` diagnostics; it returns `NotFound` for unknown/evicted tickets. These error paths do not allocate an owned result buffer.

## 架构修复验收

- Runtime10 定义并原子切换到一个当前 operation status ABI：out-param 为 `#[repr(C)]` 的 fixed-layout DTO，字段仅包含 ABI version、ticket/handle、数值 phase、completed/total work、数值 detail kind 和 detail value。poll 的不变量为 `String/JSON=0`：不构造 `String`、`serde_json::Value` 或 `ZrOwnedByteBuffer`；最终 `harvest` 才可一次性返回 bounded owned result payload。
- 当前 phase 集必须显式覆盖 `queued`、`preparing`、`ready_to_apply`、`completed`、`failed`、`cancelled`、`expired`、`harvested`；unknown 必须按上节作为 `NotFound` FFI outcome，而不是 phase。detail kind 必须覆盖 none、queue/budget、deadline、cancel、worker-panic、worker-channel-lost、owner-apply failure、TTL 和 harvested origin。不要让错误字符串成为 progress truth；可诊断文字仅在 terminal harvest/diagnostics owner 生成一次。
- Runtime11 handler contract 拆成三种不可混用的 owner：runtime tick 在确定预算内从 `World` 取得 immutable prepare input，unified bounded pool 仅对 owned input 执行 pure prepare，runtime tick 再在确定预算内对 `World` apply prepared command。poll、submit 和 harvest 都不得调用 handler 或持有 session lock 跨越 snapshot/prepare/apply。
- task store 对 queued input、prepared command、terminal result 分别执行 count 和 byte admission；deadline、cancel、panic、worker channel loss、apply error 与 TTL 回收都必须进入一个可观察的 terminal transition。terminal payload 被 harvest 或 TTL 清理后，必须仅保留 count-plus-byte bounded metadata tombstone，按最早 terminal timestamp 决定性淘汰并增加 diagnostics counter；淘汰后才返回 unknown/`NotFound`。worker panic 经 `catch_unwind` 转为 failed detail，绝不遗留 preparing/running ticket。
- cancel 或 deadline 在 prepare 完成但 owner apply 前到达时，必须先把 ticket 原子转为 phase 6 或 7 并释放 prepared bytes；completion drain 和 apply drain 都只接受仍为 phase 3、未 cancel、未逾期的 ticket。过期/取消 prepared command 不得触碰 `World`。
- Runtime10 与 Runtime11 的 hard cut 同一变更中删除 V1 JSON poll function/type/table slot，不增加 forwarding alias、compatibility shim 或 call-site fallback。Runtime03/App/Editor consumer 的 status decode 必须在同一 ABI migration 后才重启上层 gate。
- 验收矩阵覆盖 1/1k/100k tickets、0/10/1000 ms deadlines、0/1/64 MiB payload/result、panic/worker-channel-loss/cancel/expired/unknown/harvested status；证明 poll caller work 为 0、session lock 不跨 handler、queue/RSS 有界，且 navigation World mutation/result/harvest 等价。

## 禁止临时方案

- 不要把 `RuntimeOperationContext`、`World`、session guard 或 handler trait object 移入 worker；不要为 navigation 建立私有线程或第二个 pool。
- 不要保留 V1 JSON poll 并在 V2 旁边逐调用方兼容，也不要用 polling side effect、sleep、unbounded channel、unbounded result map 或测试 cfg 跳过来伪装异步。
- 不要在 progress DTO 中保留 owned `String`/JSON detail，或在每次 poll 分配/序列化终态文本。
- 不要削弱现有 navigation operation 语义、panic terminality、unknown-handle 或 early-harvest coverage。

## 修复结果与回传

Open state: `待 Runtime10 冻结 current status ABI 和 Runtime11 两阶段 handler contract；现有 Runtime11 source 仅包含独立的 one-lookup harvest micro-optimization，尚未满足本 handoff。`

- Handoff template validation and fixed-wire static anchors pass; independent design review is Critical 0 / Important 0 / Minor 0.

回传条件：Runtime10 ABI hard cut、Runtime11 bounded task/worker/apply implementation、Runtime03/App/Editor consumer migration 及 source-bound managed validation 均完成后，Runtime11 才可恢复 `operation-service-synchronous-unbounded` 的 PERF-MVP-435 closeout。
