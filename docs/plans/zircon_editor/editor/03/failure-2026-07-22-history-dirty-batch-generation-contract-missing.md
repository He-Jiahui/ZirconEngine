---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: history-dirty-batch-generation-contract-missing
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_editor/editor/03
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/core/editing/engine/transaction/dirty_batch.rs
  - zircon_editor/src/core/editing/engine/transaction/save_token.rs
  - zircon_editor/src/core/asset/dirty/registry.rs
tests:
  - tools/tests/test_editor03_saved_top_save_token_contract.py
  - zircon_editor/src/tests/editing/transaction_engine/dirty_batch.rs
---

# Editor03：缺少 saved-top 脏态批量代际契约

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：Editor09 DirtyRegistry 增量投影（PERF-MVP-554 回链）
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：saved-top 与 dirty cursor 的唯一权威属于 Editor03 transaction history，Editor09 只消费增量投影。

## 失败现象与复现证据

Editor09 的旧多文档 dirty snapshot 只能对每个注册文档逐项调用
`EditorTransactionEngine::is_dirty`。因此它先复制全部 external-effect map，再按文档重复获取 Editor03
operation lane；任一 external generation 变化还会重做整批。Editor03 只有逐 history 查询和面向保存
完成的 branch generation，没有可供 consumer 增量读取的 dirty change cursor。

进一步审计确认 `mark_saved_if_unchanged` 改变 `saved_top` 后不会推进 branch generation。这对 save token
幂等是正确的，但若直接把 branch generation 当 dirty generation，会漏掉“保存完成后变干净”事件。

## 最低共享层根因

Editor03 的 saved-top authority 没有发布包含 mark-saved clean transition 的 typed dirty generation/cursor，consumer 只能重复查询完整 history 与 external-effect snapshot。

## 架构修复验收

Editor03 必须作为 saved-top 唯一 authority 提供 engine-lineage-bound 的 typed dirty cursor、
`Unchanged/Delta/Reset` batch kind 和有界 history-change journal。成功 commit、undo、redo、history clear
以及真正改变 saved-top 的保存完成都必须在各自既有原子 mutation 中发布 dirty change；稳定 cursor
必须 O(1) 返回空 batch，增量工作量只随 journal delta 增长，journal 落后时显式 Reset。

## 禁止临时方案

不得在 Editor09 或 UI 再缓存第四份 transaction dirty bool，不得让 mark-saved 的 dirty 事件破坏同一 save token 的重复完成幂等，也不得以缩小重试次数掩盖 false-clean。

## 修复结果与回传

Open：源码已加入 typed batch generation、generation-indexed suffix journal 与 saved-top 事件发布，静态合同及两路独立复审已 GREEN；
Rust 聚焦/整库 current-source Cargo、Editor09 产品规模计数、failure fixed return 和受管提交
仍待完成，因此本记录不得改名为 fixed。

## 产出记录与时间

- 2026-07-22：完成 Editor09→Editor03 最低共享层定位。TDD 静态合同由缺失 owner/API 的 RED 收敛为
  13/13 GREEN；源码实现已覆盖 stable empty、single-change single-visit、changed-history-only delta、
  saved-top clean event、cross-engine cursor rejection 与 4,096 项有界 journal。Coordinator01 validation-copy 的 repo-local
  manifest graph 闭包故障仍阻断 Cargo，状态保持 `open`。
- 2026-07-22：初审 `0/1/2` 与后续 `0/2/0` 的 suffix 全扫描、counter 假绿、helper 可见性、失败矩阵和
  文档 findings 均已关闭；两路最终独立复审为 `0/0/0`。受管 Cargo/产品 trace/fixed return/commit
  尚未完成，因此只提升到 `source_complete_static_green_review_clean_cargo_blocked`，不改名 fixed。
