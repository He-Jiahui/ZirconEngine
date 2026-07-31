---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: world-sync-subscription-invalidation-scaling
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/02
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/inspection/mod.rs
  - zircon_runtime/src/scene/inspection/subscription.rs
  - zircon_runtime/src/scene/inspection/subscription/tests.rs
  - zircon_runtime/tests/runtime_world_sync_subscription_table.rs
tests:
  - cargo test -p zircon_runtime --test runtime_world_sync_subscription_table --locked --jobs 1 -- --nocapture --test-threads=1
  - 1/1k/100k watch and mutation-storm scale fixtures
---

# Editor02：World sync subscription invalidation规模交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime scene inspection新增subscription增量性能审查，PERF-MVP-468
- 修复责任计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 交接原因：Editor02拥有WatchKey、SubscriptionTable、gateway pump与view dirty协议；Runtime inspection只是最低共享实现落点。
- 生命周期键：`world-sync-subscription-invalidation-scaling`

## 失败现象与复现证据

`invalidate_subtree`为每个WatchKey重新判断variant；每个Subtree watch又新建BTreeSet并从同一entity沿parent chain走到root/cycle，复杂度O(watches×depth×log depth)。`invalidate_all_assets`扫描全部异构key并collect临时Vec，component type invalidation为map lookup分配String。spawn/reparent/reload storm的pending facts在frame flush前没有显式count/bytes预算。

新增实现尚未接入全部mutation throat；本交接不否定其token lifecycle/确定性flush基础合同，也不把尚未运行的动态规模门写成通过。

## 最低共享层根因

单一`BTreeMap<WatchKey,...>`方便通用注册，却让触发端缺少按variant和root/component/asset identity的直接索引；subtree判断以“每watch重新走entity ancestry”实现。事实队列只有帧末flush，没有producer burst预算/coalesce政策。

## 架构修复验收

- SubscriptionTable按variant拥有direct maps：world tokens、subtree root→tokens、component type id→tokens、asset id→tokens；by-token仍为唯一unwatch反查。
- 单结构fact只构造或借用一次bounded ancestor chain，逐ancestor root直接取tokens；reparent before/after各一次，不随watch总数重复走链。
- component type使用interned/borrowed identity，lookup不分配String；asset reload只访问asset index，不扫描其他key或collect临时Vec。
- facts按语义coalesce并有count/bytes/age预算与overflow诊断；critical structure facts不静默丢失。
- watches/depth/facts 1/1k/100k记录ancestor walks/visited alloc、key probes、pending peak/age/drop和p95：ancestor walk≤1/fact、工作近depth+matched tokens、队列/RSS有界。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止保留generic by-key和四张typed map的双注册truth；typed maps可由单一registration authority原子维护。
- 禁止缓存上一次ancestor Vec但无hierarchy generation失效；rename/reparent/despawn必须精确失效。
- 禁止只给pending Vec reserve或提高上限来替代burst budget/coalesce/backpressure。

## 修复结果与回传

Open state: `源码已修复，受管验收与独立复审待完成`; no dynamic pass or fixed return is claimed.

- generic `by_key` 已删除；`by_token` 原子维护 world/subtree/component/asset typed indexes。
- subtree invalidation 每 fact 只构造一条 cycle-guarded ancestor chain；component lookup 借用 `&str`；aggregate reload 只遍历 asset index。
- pending facts 以 entity/scene/reload key 合并，并受 count/估算 bytes/generation age 预算约束；overflow/age breach 留下 dirty resync 与累计诊断。
- r3 静态 TDD 从 `5 failed / 1 passed` 收敛至 `6/6 GREEN`；100k integration fixture 已落盘但尚未取得受管终态。
- failure `related_code` 已从不可哈希的目录占位符收敛为本修复实际拥有的 exact4 Rust 文件，供 source-bound failure priority 与 fixed return 审计；未吸收未修改的 interface/editor 目录。
- 2026-07-22 editor consumer补证：WorldWatchMap同view多token已用borrowed mark把ViewInstanceId clone降为
  unique dirty views；但每batch仍建立seen/duplicate/unknown三套BTreeSet，`InvalidationBatch.dirty`没有
  count/bytes/canonical标志。最终Runtime flush须发布bounded sorted-unique batch/cursor，Editor normal快路不再
  O(D logD)重验，malformed transport才进入诊断慢路；Cargo/100k产品证据未完成，failure保持open。

## 产出记录与时间

| 里程碑 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|
| PERF-MVP-468 / Editor02 M2.1 source repair | `source_complete_static_green_validation_pending` | 2026-07-22 | exact8 源码、回归、模块文档与 failure record 已更新；typed direct routing、single ancestry walk、bounded semantic coalesce、overflow diagnostics 和 100k fixture 已落盘，静态合同 `6/6 GREEN`。Cargo、性能原始输出、独立复审与 canonical failure return 待完成。 |
