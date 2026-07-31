---
owner_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
milestone: M3
slice: bounded-fifo-tool-scheduler-core
status: source_complete_static_green_mount_pending
related_code:
  - zircon_editor/src/core/tools/mod.rs
  - zircon_editor/src/core/tools/tool_id.rs
  - zircon_editor/src/core/tools/scheduler.rs
tests:
  - zircon_editor/src/core/tools/tests.rs
  - tools/tests/test_editor08_tool_scheduler_contract.py
---

# Editor08 M3 bounded FIFO ToolScheduler core

Plan: `docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`

Milestone: M3

Status: `source_complete_static_green_mount_pending`

## 范围

本切片实现 Editor08 M3.2 的独占资源调度核心，但暂不修改已有未提交挂载所在的
`zircon_editor/src/core/mod.rs`。它定义 typed `ToolId`、三个固定独占资源、单 holder + 有界 FIFO、
typed acquire/release/withdraw/release_all 结果，以及由调用方同步投递的生命周期事件。

本切片不接管 Editor05 的场景模式行为、不启动 Editor15 导出向导，也不新增第二条消息总线或后台
worker。05/15 consumer 和 Editor02 bus adapter 必须在 core facade 可用后的 successor 中接入。

## 实施阶段

- [x] 注册并领取 7 文件精确 Session scope。
- [x] 先建立静态 RED 合同，确认 folder owner 缺失时测试失败。
- [x] 新增 `ToolId`，拒绝空 id、超过 128 bytes 的 id 与非 ASCII identifier 字符，避免裸
  `String` 身份或单项 retained bytes 无界。
- [x] 新增 `ExclusiveResource::{ViewportInput, ModalSurface, SceneModeSlot}`。
- [x] 每个资源只保留一个 holder 和受 `max_queue_per_resource` 限制的 `VecDeque<ToolId>`。
- [x] acquire 对当前 holder 与已排队 tool 幂等；queue full 返回 typed denial 且不改变状态。
- [x] release 只允许 holder，按 FIFO 激活队首；withdraw 只移除调用方自己的 pending 请求。
- [x] release_all 同步清理一个 tool 的全部 holder/pending 状态，不释放其他 tool 的资源。
- [x] lifecycle event 随 `ToolScheduleReport` 同步返回，scheduler 内部不保留无界 event backlog。
- [x] 静态合同转 GREEN，精确 Rust 文件 rustfmt 与 scoped diff-check 通过。
- [x] 独立初审 `0/3/1`；移除 scheduler `Clone`、增加 ToolId byte cap，并补齐 release_all
  隔离与无副作用回归。增量复审为 `0/1/0`，唯一 Important 是未挂载/未进入 crate compile graph 的
  successor blocker；排除该 blocker 后源码逻辑为 `0/0/0`。
- [ ] 待 `core/mod.rs` 现有 Editor02/13 挂载完成受管提交后，以 successor 挂载 `pub mod tools`。
- [ ] 接 Editor02 message bus adapter、Editor05 scene mode 与 Editor15 export wizard consumer。
- [ ] 运行 source-bound Rust/Cargo 行为门、性能门与 managed commit。

## 测试阶段

- RED：`python -m unittest tools.tests.test_editor08_tool_scheduler_contract -v` 因
  `core/tools/mod.rs` 不存在而失败。
- GREEN：同一静态合同 `6/6` 通过。
- Rust 行为源码覆盖：当前 holder 重复 acquire、FIFO 激活、重复 queued acquire、queue full 无副作用、
  withdraw 精确移除、release_all 跨资源清理、deactivate-before-activate 事件顺序与 ToolId 校验。
- 未运行 Cargo：folder owner 尚未挂载，当前不能把未编译行为源码声明为 Rust GREEN。

## 架构裁决

- scheduler 是独占资源唯一真源；consumer 不得各自维护“当前工具”或第二条等待队列。
- `ToolScheduler` 不实现 `Clone`；共享访问只能指向 EditorContext/service 中的同一 owner。
- FIFO 只在同一 `ExclusiveResource` 内成立；三个资源互不串行，避免 modal surface 阻塞无关 scene mode。
- 重复 acquire 不重复入队、不重复发事件；release 非 holder 不改变任何 holder/queue。
- scheduler 不持有 lifecycle history。每次变更把有限事件放入 report，由 future bus adapter 在调用栈中
  发布；若 adapter 需要 retention，必须复用 Editor02 有界 inbox，而不是在 scheduler 新增长队列。
- `release_all` 用于工具 shutdown/cancel，必须同时撤销 pending 请求并释放已持有资源，防止失活工具
  留下永久 holder。

## 性能与容量

- 资源集合固定为 3；每资源 pending 数由构造参数硬限制，默认上限 64；单个 `ToolId` 上限 128 bytes。
- holder 查询为 `BTreeMap` 固定小集合查找；FIFO push/pop 为摊还 `O(1)`。
- duplicate/withdraw 在单资源有界队列内线性扫描，最坏工作量受 queue cap 限制，不随编辑器运行时长增长。
- lifecycle events 不在 owner 内积压；单次 acquire 至多 1 条，release 至多 2 条，release_all 至多遍历 3 个资源。

## 产出记录与时间

- 2026-07-22：状态 `source_complete_static_green_mount_pending`。完成 exact7 Session、静态
  RED→GREEN（6/6）、typed ToolId、三资源单 holder + bounded FIFO、幂等 acquire、typed queue-full
  denial、FIFO release、withdraw、release_all 和同步 lifecycle report；精确 Rust rustfmt 与 scoped
  diff-check 通过。`core/mod.rs` 当前包含其他 Session 尚未提交的 `script_build/sync` 挂载，本切片严格不
  修改该 facade；因此 owner 尚未挂载，Cargo、consumer wiring、复审闭环与 managed commit 均不宣称完成。
- 2026-07-22 独立初审：`Critical/Important/Minor=0/3/1`。已关闭 scheduler `Clone` 导致仲裁真源
  分叉、ToolId retained bytes 无上限，以及 release_all/非 owner 无副作用回归不足；新增
  `#[must_use] ToolScheduleReport` 强制提示 caller 发布事件。剩余 Important 仅为刻意保留的 facade
  挂载/compile-graph blocker，等待 successor，不将当前静态 GREEN 提升为 Rust/Cargo GREEN。
- 2026-07-22 增量复审：`Critical/Important/Minor=0/1/0`；唯一 Important 是上述未挂载 blocker，
  排除该 blocker 后源码逻辑 `0/0/0`。确认 scheduler/resource state 不可 Clone、ToolId retained bytes
  有界、release_all 只按精确 ToolId 清理且保留前后 FIFO、NotHolder/NotQueued 无事件无副作用；未运行
  Cargo，父 M3.2 与本切片均不提升为完成。

## 2026-07-30 Performance01 current-source supplement

Current source now mounts both `core::tools` and `core::context`, and `EditorContextBuilder` creates the single
`ToolSchedulerService`. A tracked production call-graph search still finds no consumer calling
`EditorContext::tools()` acquire/release APIs; Editor05 scene mode and Editor15 export wizard therefore remain the
real product-wiring gate. Performance01 reread context 4/4 and tools 4/4 at current hashes, including all 18 tests.

The fixed three resources, default 64-entry single/set queue caps and 128-byte ToolId cap make the existing small
BTreeMaps and linear duplicate/withdraw scans intentionally bounded. Fyrox likewise keeps about five interaction
modes in an ordered Vec because linear search is appropriate at that scale; Unreal centralizes compatible active
modes in `FEditorModeTools`. Zircon keeps its deliberate atomic multi-resource FIFO divergence for modal/viewport
contention.

Before accepting M3.2 consumers, add queue `0/1/64`, set size `1/3`, same-tool set requests `0/32/64`, operations
`1/1M`, subscribers `0/1/100/1k`, stall `0/60s` and threads `1/16` counters. Record comparisons/moved rows,
`release_all` passes, topic parses/owned bytes, event clone bytes, scheduler/bus lock wait+hold, inbox entries/bytes/age
and UI p95. `ToolSchedulerService` currently reparses static `editor.tool` per API call and clones topic/event per
publish; cache one typed/shared built-in topic at service construction and leave fanout/backpressure ownership with
PERF-MVP-019. Keep VecDeque unless the 64-cap measurement fails budget, then converge `release_all` to one stable
retain/rebuild pass rather than adding a second scheduler index. Publish must remain outside the scheduler lock;
queued export must launch no process and partial resource ownership remains forbidden.

This supplement is static-only. Current eight-file rustfmt is not GREEN because six external modified/untracked files
have import-order differences; no source was rewritten. Managed Cargo, scale counters, Editor05/15 product wiring,
independent review and F4 product trace remain pending, so the milestone is not promoted.
