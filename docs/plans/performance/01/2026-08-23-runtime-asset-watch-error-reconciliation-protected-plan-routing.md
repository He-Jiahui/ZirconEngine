---
related_code:
  - zircon_runtime/src/asset/watch
canonical_review:
  - docs/plans/performance/01/2026-08-23-runtime-asset-watch-error-reconciliation-currentness-and-m0.md
protected_targets:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: protected-plan-routing
status: update_requested_not_applied
---

# Runtime asset watch error reconciliation保护计划路由（2026-08-23）

## 请求Performance01纠正

将`zircon_runtime/src/asset/watch/**`记录为M0后19/19 Rust文件、825 physical lines、26,889 bytes、
0 inline tests，ordered path + NUL + raw bytes + NUL SHA256为
`3b37bad0509fcc0b1fd7cd17c40f86ef5fd8662d5de9a277f93e9089d98d089b`；外部
`asset/tests/watcher.rs`为1/1文件、291行、9 tests。

保留Optimize88三项P0的唯一owner，不新建平行架构任务：

- WATCH88-P0-001：本轮已落地provider error -> debounced reconciliation request的前半步；
  scan/import/commit失败后的dirty latch、typed classification、backoff和terminal receipt仍开放。
- WATCH88-P0-002：必须发布真实committed generation delta或`SnapshotRequired`，不能复用raw input。
- WATCH88-P0-003：必须消费Runtime85 source-owner/reverse-dependency generation，不能向上猜`.zmeta`。

## Runtime04与Runtime11接纳请求

- Runtime04把source watcher、source-owner plan、candidate generation、resource mutation、durability和
  committed delta定义成一个generation transaction；所有consumer只消费qualified snapshot/delta/cursor。
- Runtime11提供唯一bounded reconcile operation lane、priority/deadline/cancel/backoff/queue-age与shutdown
  receipt；不得为watcher新增私有线程池。现有OS callback thread只做有界ingress。
- Runtime25/51/64/85/87继续分别拥有provider uncertainty、registry durability、resource authority、
  source dependency和rename identity；Runtime04只能组合这些正式合同，禁止兼容facade或双写。

## 受保护索引状态请求

- `pending.md`：记录`static_current_revalidated / error-reconcile M0 landed / dynamic_and_structural_pending`，
  并链接canonical review与Optimize88。
- `review.md`：在current-source managed Cargo、真实watcher fault injection、WPR/xperf IO/CPU/queue、至少
  31次F4 latency/RSS/power和可见asset reload parity通过前不得加入。
- Performance01主计划：把原始event吞吐与committed generation latency分开计数；error storm验收必须
  证明reconciliation batch有界，不能只证明错误日志可见。

本Session不修改受保护索引、Performance01主计划或owner计划。当前没有动态验收里程碑，因此不提交
git commit，也不发送企微量化通知。
