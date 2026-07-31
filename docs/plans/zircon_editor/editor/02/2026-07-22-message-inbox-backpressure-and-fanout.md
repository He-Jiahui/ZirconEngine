---
owner_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
milestone: M2
slice: message-inbox-backpressure-and-fanout
status: source_complete_static_green_validation_pending
related_code:
  - zircon_editor/src/core/editor_message/bus.rs
  - zircon_editor/src/core/editor_message/inbox.rs
  - zircon_editor/src/core/editor_message/retention.rs
  - zircon_editor/src/core/editor_message/shared.rs
  - zircon_editor/src/core/editor_message/message/delivery.rs
tests:
  - tools/tests/test_editor02_message_backpressure_contract.py
  - zircon_editor/src/tests/editor_message/bus/backpressure.rs
failure:
  - docs/plans/zircon_editor/editor/02/failure-2026-07-17-message-inbox-backpressure-and-fanout.md
---

# Editor02 message inbox backpressure 与 shared fanout

Plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
Milestone: M2
Status: source_complete_static_green_validation_pending
Files: ["docs/plans/zircon_editor/editor/02/failure-2026-07-17-message-inbox-backpressure-and-fanout.md", "docs/zircon_editor/core/editor_message.md", "tools/tests/test_editor02_message_backpressure_contract.py", "zircon_editor/src/core/editor_message/mod.rs", "zircon_editor/src/core/editor_message/bus.rs", "zircon_editor/src/core/editor_message/shared.rs", "zircon_editor/src/core/editor_message/inbox.rs", "zircon_editor/src/core/editor_message/retention.rs", "zircon_editor/src/core/editor_message/message/delivery.rs", "zircon_editor/src/tests/editor_message/bus/mod.rs", "zircon_editor/src/tests/editor_message/bus/backpressure.rs", "zircon_editor/src/tests/editor_message/bus/broadcast.rs", "zircon_editor/src/tests/editor_message/bus/protocol_matrix.rs", "zircon_editor/src/tests/editor_message/bus/publish.rs", "zircon_editor/src/tests/editor_message/bus/request.rs"]

本切片修复 Editor02 消息总线的最低共享层：delivery payload 每次 publication 只构造一次，所有 subscriber 共享同一不可变 `Arc`；inbox 由无界 `Vec` 硬切为具有明确 `Lossless / Latest / Bounded` retention 的有界 owner。transaction/document/play/job terminal 保序且满载时显式 backpressure，selection/focus/dirty/progress 按语义 key 合并，未知 custom 只在 bounded lane 内淘汰。

## Scope delivered

- `EditorMessageDelivery` clone 只复制 immutable payload 的 `Arc`，不再按 subscriber 深克隆 topic/message/custom JSON/job strings。
- `EditorMessageInbox` 独立拥有三类容量、语义 coalescing、显式 drop/backpressure 和 depth/age/cumulative counters。
- 三类 lane depth 在 enqueue/replace/drain 边界增量维护；capacity/stats 不再对混合 inbox 做逐次全量计数，保留 bounded latest/eviction 的确定顺序。
- inbox 以 sequence-keyed `BTreeMap` 保存全局顺序，并用 `latest_by_key/latest_order/bounded_order` 只扫描有界同 lane；默认单 delivery 2 MiB、单 inbox logical retained payload 16 MiB，超限 lossless 显式 backpressure、latest/bounded 显式 drop。
- subscriber ID 使用 checked allocation 并返回 typed `SubscriberIdExhausted`；delivery sequence exhaustion 在 publish/broadcast report 与 request error 中显式返回，失败前不改 inbox、dirty set 或调用 handler。4 个 Editor02 测试 consumer 已硬切到 Result API；另外 3 个外部 consumer 分别归属 Editor09 与 Editor14 的受管切片，不纳入本切片业务清单。
- 同 key Latest replacement 会先排除旧值并预检其他 Latest 驱逐，全部可行后原子提交；需要驱逐时 report 同时记录 coalesced 与 dropped。delivery logical bytes 已纳入 dirty-view 动态字符串，超限消息不会进入 inbox 或 dirty set。
- `retention.rs` 是唯一分类 owner；pane、menu、job 或 retained host 不建立私有去重规则。
- 生产 `SharedEditorMessageBus::deliveries_for` 硬切删除，仅测试配置可观察浅克隆快照；生产消费继续使用 drain。

## Fresh testing evidence

- 静态 architecture contract 已从 RED（3 fail + 1 missing owner）转为初版 `5/5 GREEN`；纳入 maintained-depth 后为 `6/6`，review-driven ID/byte/index/perf guards 后为 `7/7 GREEN`。
- 精确 Rust 文件经 `rustfmt +1.94.1 --edition 2021` 解析和格式化；scoped `git diff --check` 通过。
- Rust 行为测试已经落盘，覆盖 100 subscriber × 10,000 paused storm、shared payload identity、latest 合并/替换驱逐、dirty-view byte rejection、lossless ordering/backpressure、request admission 和 bounded eviction；受管 Cargo 终态仍由本次 M2 workflow 取得。
- ignored 性能证据门以同一测试二进制的 test-only tracking allocator 统计 1/5/100 fanout 的 allocation operations/bytes，以 Windows working set 采样 RSS，并输出 queue age 与 publish p95；该门必须单线程、单测试受管运行后才可填入结果。

## Review

- 独立首审为 `0 Critical / 3 Important / 2 Minor`；第二轮复审为 `0/3/1`，确认首轮其余问题已闭合，并新增定位 same-key Latest replacement、dirty-view byte omission、allocator 门不可能通过和一处过时措辞。第二轮修复与 RED/GREEN 静态约束落盘后，第三轮复审为 `0/0/1`；实现 finding 全部关闭，唯一 Minor 是本记录一处验证措辞未刷新。该措辞修正后的最终独立复核为 `0/0/0`；milestone commit 前仍须完成受管 Cargo 与 open failure lifecycle return。
- snapshot `687` 与 validation copy `750454f293784c78970cdba435c947d2` 保留为历史失败证据：副本 12/12 匹配旧 manifest，但 current `inbox.rs` 已合法演进为 maintained-depth 实现，不得复用为 current-source 验收。
- 最近受管 job `4edbb9ed2f13429eb7b9ca3c6ae3b7d0` / run `961c11d453a14fb7bc7bdf2a7c1065b1` 在 Rust 编译前 0 tests 失败；缺失外部 `zr_vm` sibling 已路由 [Coordinator01 failure](../../../zircon_tooling/session_coordinator/01/failure-2026-07-22-validation-copy-external-sibling-path-dependency.md)，不属于 Editor02 产品错误。

## 产出记录与时间

| 里程碑 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|
| Message inbox backpressure and fanout | `source_complete_static_green_validation_pending` | 2026-07-22 | maintained lane depths、sequence-keyed order + lane index、2 MiB/16 MiB byte budgets、checked subscriber/delivery identity 与 exact16 业务清单已落地；Editor09/14 的 3 个 Result consumer 依赖已分别归属。review-driven same-key atomic replacement、dirty-view byte accounting 与 O(subscriber) metadata allocation 门已补齐；静态合同 `7/7 GREEN`，精确 rustfmt/parser 与 scoped diff check 已刷新并通过。独立首审 0/3/2、第二轮 0/3/1、第三轮 0/0/1，最终独立复核 `0/0/0`。snapshot687/copy750454 仅保留为旧源 0-test failure；外部 sibling 缺失已交接 Coordinator01。fresh snapshot/Cargo 数字、failure return、fixed 移交和 milestone commit 尚待完成。 |
