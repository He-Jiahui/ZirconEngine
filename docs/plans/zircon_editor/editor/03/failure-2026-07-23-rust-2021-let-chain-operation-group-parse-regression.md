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
tests:
  - rustfmt zircon_editor/src/core/mod.rs
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

Open state: `待 Editor03 完成 edition-2021 语法修复、rollback 回归、module-aware rustfmt 与 current-source Cargo 证据`。

## 产出记录与时间

- 2026-07-23 | Editor17 module formatting discovery | `open / routed-to-editor03` | `rustfmt zircon_editor/src/core/mod.rs` reports `let chains are only allowed in Rust 2024 or later` at `operation_group.rs:154`; no Cargo job was started and no Settings validation claim is made.
