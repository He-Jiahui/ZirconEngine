---
owner_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
milestone: M1
slice: history-dirty-batch-generation
status: source_complete_static_green_review_clean_cargo_blocked
related_code:
  - zircon_editor/src/core/editing/engine/command.rs
  - zircon_editor/src/core/editing/engine/mod.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/core/editing/engine/transaction/dirty_batch.rs
  - zircon_editor/src/core/editing/engine/transaction/save_token.rs
  - zircon_editor/src/core/asset/dirty/registry.rs
tests:
  - tools/tests/test_editor03_saved_top_save_token_contract.py
  - tools/tests/test_editor09_dirty_registry_contract.py
  - zircon_editor/src/tests/editing/transaction_engine/dirty_batch.rs
  - zircon_editor/src/core/asset/dirty/tests.rs
---

# Editor03 M1 History Dirty Batch Generation

Plan: `docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`

Milestone: M1

Status: `source_complete_static_green_review_clean_cargo_blocked`

## 范围

本切片为 Editor03 saved-top authority 增加 typed 增量 dirty 查询，并让 Editor09 以 cursor delta 合并
external effects。它不增加 UI dirty 状态、磁盘轮询、资产 registry 或另一套保存基线。

## 实施阶段

- [x] 建立 Editor09→Editor03 对应 failure handoff，明确最低共享 owner。
- [x] 先写静态/Rust 行为合同，确认缺失 API 与旧全量接口处于 RED。
- [x] 新增 folder-backed `transaction/dirty_batch.rs`，公开 engine-bound cursor、batch kind 与 state DTO。
- [x] 将 commit、undo/redo、history clear 的 branch generation 与 dirty journal 变更原子记录。
- [x] `mark_saved_if_unchanged` 只推进 dirty journal，不推进 branch generation，保留重复完成幂等。
- [x] 有界 journal 落后时返回 Reset；稳定 cursor 在构造 history 集合前直接返回空 states。
- [x] Editor09 删除 `snapshots/dirty_snapshots` 全量入口，改为 registry-bound cursor delta 与 typed removal。
- [x] external-effect journal 只复制 changed document 的 compact effect pairs；transaction delta 不缓存 bool。
- [x] 1/10,000 文档合同锁定 initial reset、stable empty 与 single-change-only delta。
- [x] 独立初审 `0/1/2`；已关闭 retained-journal 全扫描、失败行为矩阵及旧文档 finding。
- [x] 修正后两路独立复审均为 Critical/Important/Minor=`0/0/0`。
- [ ] Coordinator01 source-copy 闭包修复后运行聚焦及整库 current-source Cargo。
- [ ] 完成产品保存/关闭 trace、failure fixed return 与 exact manifest 受管提交。

## 测试阶段

- RED：新增静态合同为 1 error + 4 failures；缺失 `dirty_batch.rs`、typed façade 与 delta API。
- GREEN：`python -m unittest tools.tests.test_editor03_saved_top_save_token_contract
  tools.tests.test_editor09_dirty_registry_contract -v` 为 13/13。
- scoped `rustfmt --edition 2024 --config skip_children=true` 与 exact diff-check 已通过。
- Rust 行为源码覆盖 initial/reset、stable empty、单 history delta、saved-top clean event、foreign cursor、
  10,000 文档 stable/single delta、typed unregister 与 concurrent external generation retry。
- Cargo 不宣称 GREEN：Coordinator01 validation-copy 仍无法闭合 workspace 的 repo-local/external sibling
  manifest graph；不得用共享脏工作树或旧 job 充当 current-source evidence。

## 架构裁决

- `HistoryStore::saved_top` 仍是 transaction dirty 唯一真源；journal 只记录“哪些 history 可能变了”，
  查询时仍从当前 HistoryStore 计算 dirty。
- save-token branch generation 与 dirty publication generation 分离：前者保护 I/O 期间 history identity，
  后者覆盖 saved-top 本身的变更。
- Editor03 delta 是全局 history journal，不接受每次调用传入全量 document set；稳定查询 O(1)，delta
  只去重 journal 中的 changed history。
- Editor09 首次/reset 才发布全量注册文档；稳定 cursor 返回空，external delta 只点查 changed document，
  transaction delta 只合并已注册 document。无旧整批 API 或兼容别名。

## 产出记录与时间

- 2026-07-22：状态 `source_complete_static_green_review_pending_cargo_blocked`。exact30 union successor
  保留前序 Editor03 save-token 与 Editor09 dirty-owner 记录；完成 TDD RED→GREEN（13/13）、typed history dirty journal/cursor/batch、saved-top clean event、
  Editor09 cursor delta 硬切与 10,000 文档规模合同。独立 review、受管 Cargo、产品 trace、fixed return
  和 commit 尚未完成，父里程碑不得提升为完成。
- 2026-07-22 独立初审：Critical/Important/Minor=`0/1/2`。Important 确认 live cursor 虽有 4,096 上限，
  仍扫描完整 retained journal；现以连续 generation 计算 suffix 起点，并用 test-only visit counter 锁定
  stable=0、single-change=1。两个 Minor 通过 undo/redo/clear/dirty-generation-exhaustion/type-mismatch
  no-delta 行为矩阵与 Editor09 当前状态/模块文档更新关闭；等待复审，不提前写 review clean。
- 2026-07-22：状态 `source_complete_static_green_review_clean_cargo_blocked`。第一路复审继续发现
  counter 自报算术与 sibling test helper 可见性问题；最终实现将生产遍历和计数统一到
  `VecDeque::range(start..)` 的唯一 helper，在真实 iterator yield 时递增，并修正 `pub(super)` 可见性。
  两路最终复审均为 `0/0/0`；Editor03/09 专项静态 13/13、相关静态 48/48、exact rustfmt/diff-check
  GREEN。受管 Cargo、产品 trace、fixed return 与 managed commit 仍 pending，故 failure 保持 open。
- 2026-07-22：candidate snapshot968 已冻结 exact29 business manifest，preview 29/29 全部
  `would_change=false`。registered exact30 中的 Editor03 父计划是 foreign-mixed 治理路径，明确排除于
  snapshot/commit；本记录写入后将刷新 attribution 并创建 final source snapshot。该证据只证明当前源码
  哈希闭合，不替代受管 Cargo、产品 trace、fixed return 或 managed commit。
