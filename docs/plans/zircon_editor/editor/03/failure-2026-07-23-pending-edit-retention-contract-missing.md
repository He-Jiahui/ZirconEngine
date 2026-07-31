---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: pending-edit-retention-contract-missing
origin_plan: docs/plans/zircon_editor/editor/04-pie-and-simulation.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_editor/editor/04
fixing_child_dir: docs/plans/zircon_editor/editor/03
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editing/engine/command.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/core/play/pending_edits/intent.rs
  - zircon_editor/src/core/play/pending_edits/queue.rs
tests:
  - operation retention policy descriptor contract
  - pending edit lossless/latest/bounded/coalescing routing matrix
  - cargo test -p zircon_editor --lib --locked --jobs 1 -- --test-threads=1
---

# Editor03：Pending edit retention contract缺失

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/04-pie-and-simulation.md`
- 来源执行切片：`failure-2026-07-22-play-pending-edit-unbounded-queue`（PERF-MVP-551）
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：延迟编辑的保留规则必须由 operation/transaction authority 声明。Editor04 只拥有 Play 的路由和队列生命周期，不能按 operation path 字符串猜测 transaction 是否可 latest/coalesce/drop。
- 生命周期键：`pending-edit-retention-contract-missing`

## 失败现象与复现证据

当前 `EditorOperationInvocation` 仅持有 `operation_id`、owned JSON `arguments` 与可选 `operation_group`。`PendingEditIntent` 直接保存整个 invocation，`PendingEditQueue` 对所有 target 使用同一 `VecDeque`。没有 typed retention descriptor 来区分用户终态、可替换 property edit、可合并连续 transform 和必须有界拒绝的操作。

因此 Editor04 若直接施加统一 entry/bytes/age 限制，只能静默丢弃某些用户明确事务，或通过 `operation_id` 文本特判建立第二套编辑语义；两者都违背 Transaction/Undo 的唯一 authority。

## 最低共享层根因

Editor03 operation/transaction contract 没有面向异步/延迟 consumer 的 typed retention policy、semantic coalescing key 或 compact payload/share handle。`operation_group` 只描述历史分组，不能判定 lossless/latest/bounded/coalescing，也不能证明不同 target 的顺序与 retry authority。

## 架构修复验收

- Editor03 在 operation descriptor/transaction contract 发布 typed `PendingEditRetention`：`Lossless`、`Latest { key }`、`Bounded { key, max_entries, max_bytes, max_age }` 或明确可合并 policy；禁止 consumer 使用 raw operation-name string branch。
- Policy 同时声明 target/order、terminal preservation、coalescing identity、payload ownership和失败后唯一 retry authority；连续 transform/property 的 latest/coalescing 不得改变 operation-group undo 语义。
- `EditorOperationInvocation` 或其 descriptor 提供共享/compact payload representation，使 PendingEdit 详情无需深 clone owned JSON；Editor04 的 decision UI 只能读取该 descriptor 与 queue page/cursor。
- Contract tests 覆盖 lossless terminal、latest replacement、bounded backpressure、operation group、target ordering、apply/discard/failure/retry；随后由 Editor04 接入 entry/bytes/age queue、paged decision 和 budgeted apply，向上复跑 PIE route/failure gates。

## 禁止临时方案

- Do not encode policy in `operation_id` string prefixes, UI branches, ad hoc allowlists, or a second Play-only operation registry.
- Do not interpret `operation_group` as a retention policy or silently drop terminal/user-confirmed operations under pressure.
- Do not retain owned JSON snapshots as the long-lived compact payload, add compatibility aliases, or weaken transaction/undo ordering tests.

## 修复结果与回传

Open state: `待修复`; Editor04 keeps `play-pending-edit-unbounded-queue` open and must not implement a uniform drop policy. After Editor03 returns this contract, Editor04 owns the queue budget/page/apply integration and its upward PIE acceptance.

## 产出记录与时间

- 2026-07-23 | Editor04 source diagnosis | `open / routed-to-editor03` | Raw `EditorOperationInvocation { operation_id, arguments, operation_group }` was verified insufficient to classify deferred edit retention. The canonical lower-layer descriptor contract is now owned by this Editor03 failure; no Editor04 queue behavior or validation pass is claimed.
