---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: world-batch-mutation-clone-transaction
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/ecs/archetype
  - zircon_editor/src
tests:
  - cargo test -p zircon_runtime --lib scene --locked --jobs 1 -- --nocapture --test-threads=1
  - editor undo/import batch success and failure scale fixtures
---

# Runtime08：World batch mutation零全场clone事务交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime scene world query/records/typed API 4/4逐Rust文件性能审查，PERF-MVP-467
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 共同验收：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：Runtime08拥有World storage/archetype mutation transaction；Editor03拥有command/undo artifact与inverse delta消费边界。
- 生命周期键：`world-batch-mutation-clone-transaction`

## 失败现象与复现证据

`insert_node_records`为保证undo/import批次原子性先深clone完整World，再clone每个NodeRecord并通过普通insert执行全部component写入和中间archetype更新；成功后整体替换World，失败则丢弃整份clone。大世界中的单实体undo/import因此也复制所有节点、组件、dynamic JSON、registry和derived state。

## 最低共享层根因

World只有单项mutation API和“clone whole authority”事务办法，没有可预验证的batch mutation plan、affected-row undo delta、copy-on-write storage page或单次generation commit边界。Editor命令层也无法传递already-owned records/compiled component writes。

## 架构修复验收

- Runtime08提供batch plan：先验证entity identity、schema/reference和每entity最终signature，不修改authority；记录affected rows/components的before delta或共享copy-on-write pages。
- commit只写affected storage/archetype rows并一次发布query/derived/world generation；同entity多record/component更新合并，不产生中间archetype churn。
- Editor03 command/undo传递owned/Arc artifact和inverse delta，不先构造第二份完整World；failure/cancel只丢未发布plan。
- world/records/payload 1/1k/100k、batch 1/1%/100%、success/failure记录World/NodeRecord clone bytes、component writes、archetype moves、rollback bytes和p95：full World clone=0、工作近affected payload、failure authority零变化。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止把clone从World移动到Editor snapshot或serde bytes后声称解决；验收统计端到端owned bytes/RSS。
- 禁止逐record commit后用补偿命令回滚；authority必须在预验证成功前不可见。
- 禁止保留大批次走新transaction、小批次默认clone World的双权威路径。

## 修复结果与回传

Open state: `前向修复中`; no pass is claimed.

- 已完成：`insert_owned_node_records(Vec<NodeRecord>)` 先整体验证 stable identity、重复项、transform、next id 与 mobility，随后将 owned records 一次发布；World generation、derived/query invalidation 与 lifecycle dispatch 在整个 batch 可见后只发布一次。
- 仍未完成：generic component/resource writes、archetype final signature 与 Editor03 inverse delta 尚未进入同一 affected-row COW transaction；当前实现也未提供 1/1k/100k clone/move/rollback probes。因此 full World clone=0 与跨层 zero-partial-mutation 未验收，handoff 保持 `open`。
- 当前证据：仅完成格式、diff 与静态 source guard；未运行声明的受管 Cargo、Editor undo/import 上游门或 scale fixtures。
