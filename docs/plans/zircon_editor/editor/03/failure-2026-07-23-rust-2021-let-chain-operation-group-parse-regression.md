---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: rust-2021-let-chain-operation-group-parse-regression
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/03
plan_link_mode: child_record_only
related_code:
  - zircon_editor/Cargo.toml
  - zircon_editor/src/core/editing/engine/transaction/operation_group.rs
  - zircon_editor/src/tests/editing/transaction_engine/operation_group.rs
tests:
  - rustfmt --edition 2021 --emit stdout zircon_editor/src/core/mod.rs
  - cargo test -p zircon_editor --lib operation_group_first_push_preserves_rollback_failure --locked
  - cargo test -p zircon_editor --lib --locked
---

# Editor03: Rust 2021 let-chain parse regression

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：Editor17 module formatting discovery gate
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：operation-group rollback 的语法与错误优先级由 Editor03 所有，Editor17 只在模块格式化门中发现回归。

## 失败现象与复现证据

`zircon_editor/Cargo.toml` declares edition 2021, but
`core/editing/engine/transaction/operation_group.rs` uses the Rust 2024
let-chain form `if let Err(cleanup_error) = cleanup && !preserve_original`.
Consequently module-aware `rustfmt zircon_editor/src/core/mod.rs` fails before
formatting or compiling unrelated Editor17 settings code.

## 最低共享层根因

Editor03 的 operation-group 实现使用了超出 crate edition 2021 的 let-chain 语法，使 module-aware rustfmt 在到达 Editor17 settings 代码前即解析失败。

## 架构修复验收

Keep the existing rollback precedence while rewriting the condition with syntax
accepted by edition 2021. Add or retain the operation-group rollback regression
that proves a cleanup error is returned only when the original error is not
`RollbackFailed`.

## 禁止临时方案

Do not change the crate edition as a local workaround, suppress module-aware formatting, or move the rollback behavior into Editor17.

## 修复结果与回传

Open state: `source repaired / local module-aware Rust 2021 parse passed / managed validation pending`.

### 2026-09-07 current-source reconciliation

The current production owner already uses nested `if let Err(cleanup_error)` and
`if !preserve_original` conditions, accepted by Rust 2021. Its existing
`operation_group_first_push_preserves_rollback_failure` regression checks that the original
command and rollback errors survive cleanup and that the engine remains faulted. No source
rewrite or edition change was needed. Both files are clean against the current HEAD.

Snapshot `2860` freezes production SHA-256
`e72689a8ea280bf3f0309cc233c4bd88fec490ec227404151adc35dafcd79f17` and test SHA-256
`e0d06f4f78238fee4605664fcf423cb0ce9ead4b7890169380459fa3e43e880b`. The current module-aware
command `rustfmt --edition 2021 --emit stdout zircon_editor/src/core/mod.rs` exited 0;
stdout was discarded and no source file was modified. Exact `rustfmt --check` on the two
snapshot files also exited 0. Neither command executes the rollback regression.

Archived attribution was transferred to the existing fixing Session
`failure-cleanup-editor03-reflection-20260905` by request `7c00cec019984cc080addbb2d3c6ece9`.
The first format submission was rejected for missing current attribution and created no ticket.
After the audited transfer, managed request
`failure-roll-01a07160-editor03-operation-group-rust2021-format-20260907-r2` produced ticket
`7dd51d4a3f08469887b4253c7c0f1624`, manifest hash
`cfc1da996260137c8fb4651e32bdbf2de9c9501b6f751639fb5b68aaf76421dd`. It is queued behind 367
`validation_dependency_failed` blockers; this is not a pass. The exact rollback test and complete
Editor library managed Cargo gates remain required. The user-requested `zr_vm` skip is retained;
no new Cargo attempt repeated its known external-workspace blocker. No fixed return or closeout
commit is claimed for this lifecycle.

## 产出记录与时间

- 2026-07-23 | Editor17 module formatting discovery | `open / routed-to-editor03` | `rustfmt zircon_editor/src/core/mod.rs` reports `let chains are only allowed in Rust 2024 or later` at `operation_group.rs:154`; no Cargo job was started and no Settings validation claim is made.

## 2026-09-20 rolling repair reconciliation

The stable successor Session is `failure-roll-01a084c8-editor03-let-chain-r2`.
The coordinator transfer fingerprint
`62fc897c38f8dded30a2fe54d311e0074199dd6a45e12d83a725ffd6a24a67a3` moved the
current failure record, production operation-group source, and rollback test
from archived owners. Current hashes after transfer are:

- failure record: `53b27cb66805558bd4d3dfec573991af7519eee67ba483b2d4be9576cef17c53`
- `operation_group.rs`: `a7ad1ad33876923356198c67bea61133ed3ade5d59d9039803ab6baaa6e90021`
- transaction-engine test: `bb672c15be87c439daa66ed40b9c5be40e8eb34d92532f021815ffe640d38310`

No source edit was made in this successor. The current production branch uses
nested Rust-2021-compatible `if let Err(cleanup_error)` / `if !preserve_original`
conditions and still returns the original `RollbackFailed` error when cleanup
also fails. The existing `operation_group_first_push_preserves_rollback_failure`
test asserts both nested error payloads and the faulted engine state. A foreign
working-tree change in `zircon_editor/Cargo.toml` is retained and is not
attributed to this failure; the crate still inherits the workspace edition.

Static evidence against the transferred hashes:

- `rustfmt --edition 2021 --emit stdout zircon_editor/src/core/mod.rs` — exit 0.
- `rustfmt --edition 2021 --check` on both owned Rust paths — exit 0.
- `git diff --check` on the owned source/test/record paths — exit 0.
- A source-bound assertion guard confirmed the forbidden let-chain spelling is
  absent, the nested cleanup branch and rollback regression remain present, and
  no `edition = "2024"` override was introduced — `EDITOR03_RUST2021_LET_CHAIN_STATIC_CONTRACT_PASS`.

Independent review Session `review-editor03-let-chain-r2` checked the current
source, test assertions, crate edition boundary, and forbidden workaround list:
Critical/Important/Moderate = `0/0/0`. Review is static only and does not claim
Cargo execution.

The fresh managed focused request used the exact test filter
`operation_group_first_push_preserves_rollback_failure`. Job
`3fe61469ee7944e8bcb5ca7f74f4663f` was accepted with a Windows managed target
`D:\\cargo-targets\\zircon-engine\\pool\\20690791fb4660d3915966c526dc1db3baf41f1e57364778835b264d30228dbb`,
then exited `1` at the supervisor workspace-synchronization layer with a
`compile_workspaces.py` `JSONDecodeError`. It has no `cargo_job_runs`, test
stdout, or stderr receipt; therefore no Rust test result is inferred. The
failure remains open / `waiting_validation` pending a clean managed Cargo run,
the complete Editor library gate, and final return/closeout evidence.
