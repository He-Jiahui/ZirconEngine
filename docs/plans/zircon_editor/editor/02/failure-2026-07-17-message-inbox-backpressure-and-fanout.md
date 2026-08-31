---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: message-inbox-backpressure-and-fanout
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/02
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editor_message/bus.rs
  - zircon_editor/src/core/editor_message/inbox.rs
  - zircon_editor/src/core/editor_message/retention.rs
  - zircon_editor/src/core/editor_message/shared.rs
  - zircon_editor/src/core/editor_message/message/delivery.rs
tests:
  - python -m unittest tools.tests.test_editor02_message_backpressure_contract -v
  - paused subscriber bounded-memory pressure test
  - cargo test -p zircon_editor --lib managed_fanout_allocation_rss_queue_age_and_publish_p95_report --locked --jobs 1 -- --ignored --nocapture --test-threads=1
  - delivery ordering and dirty-set coalescing regression
---

# Editor02：message inbox 无界且 fanout 深拷贝

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：Editor message 25 个生产 Rust 文件逐文件静态审查
- 修复责任计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 交接原因：subscriber inbox、delivery ownership、dirty coalescing 和 backpressure 是 Editor02 消息协议责任。

## 失败现象与复现证据

每个 subscriber inbox 是无容量限制的 `Vec<EditorMessageDelivery>`；没有 queue depth、age、drop 或 stale-subscriber 诊断。publish/broadcast 先分配 delivered-id `Vec`，随后为每个 subscriber clone topic/message；custom JSON、job label/progress/error 字符串会随 fanout 深拷贝。

`SharedEditorMessageBus::deliveries_for` 还会 clone 整个 inbox，虽当前生产使用很少，但若用于每帧面板轮询会重复放大成本。

## 最低共享层根因

消息协议没有区分 lossless transaction/document edge、latest/coalesced progress/focus/dirty state 与可丢 telemetry，也没有 shared immutable payload 或 subscriber 消费预算。

## 架构修复验收

- paused subscriber 与 1/5/100 fanout 压测报告 RSS、allocations、queue age 和 publish p95。
- 按 topic/protocol 定义 lossless、bounded、latest/coalesced 语义；dirty mask 继续 union。
- 评估共享不可变 message/delivery payload，避免大 JSON/string 为每 subscriber 深 clone。
- unregister、request、broadcast、ordering 和 retained UI 刷新合同全部回归。

## 禁止临时方案

- 不得统一清空或静默丢弃所有 inbox；transaction/document terminal 语义必须保留。
- 不得让各 pane 自建私有去重规则，delivery policy 必须在 Editor02 owner。

## 修复结果与回传

Open state: `2026-08-05 forward repair source static green / independent second review 0/0/0 / managed Cargo pending`。

- 已将每次 publication 的 delivery payload 收敛为单一 `Arc`，subscriber fanout 只复制共享句柄。
- 已按 payload/protocol 建立 `Lossless / Latest / Bounded` retention；transaction/document terminal 不丢弃，latest 按语义 key 合并，custom bounded 淘汰可观测。
- 已加入 depth/class depth、drained、coalesced、dropped、backpressured 与 message-age 指标；生产 `SharedEditorMessageBus::deliveries_for` 已硬切为 test-only。
- lane depth 在 enqueue/replace/drain 边界增量维护，capacity/stats 不再对 mixed inbox 做全量 lane 计数；最新 current source 因此 supersede snapshot687 的旧 `inbox.rs`。
- review 后继续硬化为 sequence-keyed global order + bounded lane indexes，默认 single/total logical payload 预算为 2 MiB/16 MiB；subscriber/delivery identity 采用 checked exhaustion。publish/broadcast/request 的 exhaustion 在任何 inbox/dirty/handler 副作用前 typed 返回。第二轮复审继续补齐 same-key Latest replacement 的原子预检/驱逐、dirty-view 动态字符串计量，以及允许小型 O(subscriber) 索引元数据但禁止 payload 深克隆的 allocation 门。
- 已落盘 100 subscriber × 10,000 selection update、lossless backpressure、request handler admission、ordering 与 bounded eviction 行为测试。
- 已落盘独立 ignored 性能证据门：1/5/100 subscriber、1 MiB custom payload allocation、10,000 次 paused latest-state storm、RSS、queue age 与 publish p95 统一输出机器可读 `EDITOR02_FANOUT_BENCHMARK`；尚未把未运行的数字写成验收结论。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据与待办 |
|---|---|---|---|
| 2026-07-22 | `source_complete_static_green_validation_pending` | shared immutable delivery、sequence-keyed bounded inbox、lane indexes、2 MiB/16 MiB logical byte budgets、checked identity、pressure metrics、行为回归及 1/5/100 mixed-backlog 性能门源码完成；Editor02 exact16 业务清单内 4 个 consumer 已硬切 Result registration，另 3 个依赖归属 Editor09/14。 | 独立首审 0/3/2、第二轮 0/3/1、第三轮 0/0/1，最终独立复核 0/0/0；第二轮 RED/GREEN 修复涵盖 same-key atomic replacement、dirty-view bytes、allocator metadata slope 与过时债务措辞，第三轮唯一 Minor 已关闭。snapshot687/copy750454 的旧 12/12 输入及 job4edbb 只作 external-sibling 0-test failure 证据；Coordinator01 failure 保持 open。需 fresh exact16 snapshot/copy、Rust/性能数字后才能回传。 |
| 2026-08-05 CST | `source_forward_repair_static_green / independent_second_review_green / managed_validation_pending` | Shared bus 将订阅/sequence 元数据与每 subscriber inbox 拆分：fanout plan 在 metadata lock 内冻结，enqueue 在 inbox-local lock 外执行；lossless targets 按 subscriber ID 取得全部锁后再 all-or-nothing admission。latest/bounded lane order 改为 sequence-keyed BTree index，out-of-order completion 保留最高 sequence，不再发生 VecDeque 中段移位。request 直接复用 enqueue delivery 的 Arc payload。 | 新回归覆盖 out-of-order latest/bounded sequence 和 shared request payload identity；supporting Python capacity guard 已从废弃的 `>= capacity` 失败分支前向硬切为 `can_enqueue_lossless` 的 `< capacity` 准入与 enqueue 委托，先前 1/7 RED 后当前 `7/7 GREEN`。`rustfmt`、scoped diff 与 failure graph 通过，独立二审 `0/0/0`。受管 Cargo、paused 1/5/100 fanout 与 upward failure return 仍待完成，failure 保持 open。 |
| 2026-08-05 CST | `source_forward_repair_static_green / independent_second_review_green / managed_validation_queued` | 当前 source manifest 已封存 `bus/inbox/retention/shared/delivery` 与 fanout benchmark fixture，1/5/100 paused-subscriber performance gate 由 coordinator 执行。 | Ticket `57ec83d505164a79a43fca34603a2fc8` 已收到 queued receipt；不轮询、不以排队状态推断性能数字或通过结论，failure 保持 open。 |

2026-07-22逐文件复核确认O(1) lane-depth、`latest_by_key` 与 sequence map 止损成立，但failure仍open：global bus mutex仍包全部inbox和fanout，bounded-lane order 的中段 removal 仍有线性移位，request仍先 clone Custom JSON；PERF-MVP-019 的锁粒度与 request ownership 后续优化尚未完成。

2026-08-28 current-source static return first reproduced the supporting Python suite at `6/7`: the sole
error was a stale read of deleted `tests/editor_message/bus/backpressure.rs`. The Rust regressions remain mounted
under the current folder-backed owner: `backpressure/mod.rs` mounts `behavior`, `fixture`, and `performance`;
atomic admission/eviction/identity cases live in `behavior.rs`, while the ignored 1/5/100 fanout metrics live in
`performance.rs`. The guard now follows those owner leaves and preserves every prior semantic assertion; the full
Python contract is GREEN without modifying the foreign-owned message-bus source. This closes only the stale
read-path defect. The failure remains `open / managed_validation_pending`; no performance number or Cargo result
is inferred from static source.
