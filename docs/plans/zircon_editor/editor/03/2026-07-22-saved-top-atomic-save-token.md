---
owner_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
milestone: M1
slice: saved-top-atomic-save-token
status: source_complete_static_green_review_clean_cargo_blocked
related_code:
  - zircon_editor/src/core/editing/engine/command.rs
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editing/engine/mod.rs
  - zircon_editor/src/core/editing/engine/transaction.rs
  - zircon_editor/src/core/editing/engine/transaction/save_token.rs
  - zircon_editor/src/core/editing/engine/transaction/operation_group.rs
tests:
  - tools/tests/test_editor03_saved_top_save_token_contract.py
  - zircon_editor/src/tests/editing/transaction_engine/history.rs
  - zircon_editor/src/core/asset/dirty/tests.rs
  - zircon_editor/src/tests/editing/transaction_engine/operation_group.rs
---

# Editor03 saved_top atomic save token

本切片修复 Editor09 save/save_all 的最低共享层：保存开始时从 Editor03 transaction engine 捕获
typed token，写盘成功后由同一引擎操作锁比较并更新 `saved_top`。Editor09 不缓存 history cursor、
transaction dirty 或 branch generation。

## 架构合同

- token 绑定不可伪造的 engine lineage、`HistoryContextId`、current transaction identity（空根为
  `None`）与单调 branch generation；跨 engine/context 使用返回 typed mismatch。
- commit、undo、redo、redo 截断、capacity eviction 与 clear 都推进对应 history generation。
- `mark_saved_if_unchanged` 先校验 token history，再在同一 operation/锁内比较 generation 与 identity；
  漂移返回 typed `HistoryChangedDuringSave`，不更新较新的 `saved_top`。
- 首次成功返回 `Marked`，相同 token/相同版本重复完成返回 `AlreadyMarked`。
- capture 与 completion 两端都拒绝 active transaction scope，避免把已 apply、未 commit 的临时世界写盘
  或标成 clean。
- operation group 以 identity-bearing `Initializing/Open/Flushing` reservation 防止首次 command 入栈前
  被并发 flush；`begin_transaction` 在创建 active frame 的同一锁内校验 reservation capability。Busy/
  branch-generation exhaustion 后保留可重试 group，stale cleanup 不删除 successor，也不留下无 owner
  active transaction。
- 删除不安全的公开 `mark_saved(history)` 两步入口，不保留 wrapper、deprecated alias 或 UI 单线程例外。

## 产出记录与时间

| 里程碑 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|
| Editor03 M1 saved_top token / TDD RED | `tdd_red` | 2026-07-22 | exact11 scope 已领取；先落静态合同与 Rust 行为用例，覆盖保存中 commit、same-top branch replacement、undo/redo、空根、cross-document、重复 completion、capacity/clear 与多文档部分成功。生产符号尚未实现，不宣称静态或 Cargo GREEN。 |
| Editor03 M1 saved_top token / source implementation | `source_complete_static_green_review_clean_cargo_blocked` | 2026-07-22 | exact14 将协议提取到 `transaction/save_token.rs` 与 `transaction/operation_group.rs`，主 `transaction.rs` 保持 918 行；`HistorySaveToken` 绑定 engine lineage/context/current transaction/generation，capture/completion 拒绝 active scope，compare-and-mark 在同一 operation/state lock 内返回 `Marked`/`AlreadyMarked` 或 typed mismatch/changed。operation-group 的 pre-begin capability、失败恢复、successor identity 隔离和首 push `RollbackFailed` 原错保留均有确定性源码测试。旧 transaction `mark_saved` 已物理删除；Editor03+Editor09 静态合同 `11/11`、exact rustfmt、旧调用扫描与 diff-check 通过；两路独立终审均为 Critical/Important/Minor=`0/0/0`；candidate snapshot936 preview 为 exact14 `14/14` 无漂移。Coordinator01 validation-copy manifest graph failure与当前受管 Cargo 队列仍阻断 Rust gate，故不宣称 fixed/commit。 |
