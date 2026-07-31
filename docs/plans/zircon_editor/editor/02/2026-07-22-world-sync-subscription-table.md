---
owner_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
milestone: M2
slice: world-sync-subscription-table
status: source_complete_static_green_validation_pending_performance_failure_open
related_code:
  - zircon_runtime/src/scene/inspection/mod.rs
  - zircon_runtime/src/scene/inspection/subscription.rs
tests:
  - zircon_runtime/src/scene/inspection/subscription/tests.rs
  - zircon_runtime/tests/runtime_world_sync_subscription_table.rs
  - tools/tests/test_editor02_world_sync_subscription_table_contract.py
---

# Editor02 world sync subscription table

Plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
Milestone: M2
Status: source_complete_static_green_validation_pending_performance_failure_open
Files: ["docs/zircon_runtime/scene/inspection/subscription.md", "tools/tests/test_editor02_world_sync_subscription_table_contract.py", "zircon_runtime/src/scene/inspection/mod.rs", "zircon_runtime/src/scene/inspection/subscription.rs", "zircon_runtime/src/scene/inspection/subscription/tests.rs", "zircon_runtime/tests/runtime_world_sync_subscription_table.rs"]

本切片完成 M2.1 的 runtime `SubscriptionTable` 最低共享层，并在 r3 接收 PERF-MVP-468 后完成 direct-index 与 bounded-coalesce hard cut；不把尚未接线的 session owner、mutation throat、gateway 或 retained-host pump 写成完成。

## Scope delivered

- `by_token` 是 unwatch authority，world/subtree/component/asset 各自 typed direct index；旧 generic `by_key` 已删除。
- subtree 每 fact 只构造一次 cycle-guarded ancestor chain，再按祖先 root 直查 token，不随 watch 数重复遍历。
- component mutation 使用 borrowed `&str` lookup；aggregate asset reload 只访问 asset index，不扫描异构 key 或 collect 临时 token Vec。
- pending dirty 使用 `BTreeSet` 去重排序；pending facts 按 entity/scene/reload 语义键合并，并受 count/估算 bytes/generation age 三预算约束。
- overflow 保留既有 dirty resync 信号并累计诊断；age breach 每帧只追加一次 world resync 标记，结构事实不静默丢失。
- `unwatch` 同步撤销 pending dirty，session drop 直接回收整表，不建立第二持久化 authority。
- `record_fact` 覆盖结构、scene asset 与批量 reload；`invalidate_subtree`、`invalidate_component_type`、`invalidate_asset` 补齐当前 DTO 不携带的咽喉信息。
- subtree 祖先遍历使用 visited guard，malformed parent cycle 不会卡死。

## Fresh testing evidence

- r2 TDD 从 `1 failed + 3 errors` 收敛为 `5/5 GREEN`；PERF-MVP-468 r3 新合同先得到 `5 failed / 1 passed`，direct-index/budget 实现后为 `6/6 GREEN`。
- Rust 回归已落盘：token lifecycle、typed route、1k unrelated subtree watch 单 ancestry walk、borrowed component lookup、fact coalesce、count/bytes/age overflow、dirty resync 与 malformed cycle guard。
- 公共 integration gate 新增 ignored 100k case，记录 direct key probe、matched token、99,999 次 coalesce、pending peak 与 overflow 状态。
- `runtime_world_sync_subscription_table` 公开 integration gate 只编译 production runtime library，不启用当前受 Render10 测试漂移影响的全量 lib-test modules。
- 精确 Rust 文件使用 Rust 1.94.1 rustfmt；受管 Cargo 尚未取得终态，不声明动态 GREEN。

## Remaining M2 work

- `RuntimeDynamicSession` 持有表并在 session teardown 回收。
- spawn/despawn/reparent/component access/dynamic scene reload 咽喉接线，`LevelSystem::tick` 末尾冲刷。
- gateway query/watch/unwatch/drain 与每帧一次 pump；editor `watch_map` 已作为独立 exact7 source slice 落盘，仍待 Cargo/review。
- hierarchy subtree diff 与 5k 单节点改名重建行数验收。

## 产出记录与时间

| 里程碑 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|
| M2.1 subscription table support slice | `source_complete_static_green_validation_pending_performance_failure_open` | 2026-07-22 | r3 exact8 已 hard cut generic `by_key`，完成四类 typed direct index、单祖先链、borrowed component lookup、bounded semantic coalesce 与 overflow diagnostics；PERF 静态 TDD 从 `5 failed / 1 passed` 收敛至 `6/6 GREEN`。受管 Cargo、100k 动态数据、独立复审、failure fixed return、M2.1 其余 wiring 与 managed commit 待完成，父 M2 保持 pending。 |
