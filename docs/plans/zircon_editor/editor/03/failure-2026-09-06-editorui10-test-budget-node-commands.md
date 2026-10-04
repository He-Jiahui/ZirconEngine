---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-09-06
summary_slug: editorui10-test-budget-node-commands
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor/03
related_code:
  - zircon_editor/src/tests/editing/node_ops/mod.rs
tests:
  - rustfmt --edition 2024 --check zircon_editor/src/tests/editing/node_ops/mod.rs
  - cargo test -p zircon_editor --lib tests::editing::node_ops --locked
  - cargo test -p zircon_editor --lib editing --locked
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root .
---

# Editor03: node command tests exceed the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：2026-09-06 rolling test-budget audit after the reflected-command split.
- 修复责任计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 交接原因：Editor03 owns node-command effects, history recovery and transaction ordering.

This is the current node-command child of the open
[EditorUI10 test budget failure](../../editor_ui/10/failure-2026-08-13-editor-test-file-budget-gate-missing.md).
Editor03 owns command effects, history recovery, selection snapshots and transaction ordering.
The stable fixing Session is `failure-cleanup-editor03-reflection-20260905`.

## 失败现象与复现证据

The 2026-09-06 structure audit reports `zircon_editor/src/tests/editing/node_ops.rs`
at 882 lines with no exemption. The file contains 20 tests and one gateway fixture.
The pre-edit SHA-256 is
`54e7d363a22a87365c34151d5d1fbbcdd65f11b88fcb4aadaa5a14e7ab8fd03d`.
It has no Git diff against current HEAD `d3174741300b272774c56a37465f043a03debd6c`.

## 最低共享层根因

Gateway recovery, selection exhaustion, Play history, deletion and hierarchy
contracts have accumulated in one flat test owner without module boundaries.

## 架构修复验收

- Split by command behavior into folder-backed tests with a declaration-only `mod.rs`.
- Preserve all 20 tests, gateway methods, assertions and shared editing fixtures.
- Keep every new test file at or below 800 lines; no old flat file, compatibility mount,
  duplicate fixture or budget exemption may remain.
- Run the focused node-command and parent editing managed Rust gates, then the
  originating structure gate after the remaining owners are repaired.
- Require an independent review with Critical/Important/Moderate all zero before
  canonical fixed return and coordinator closeout.

## 禁止临时方案

Do not raise the budget, suppress the audit, remove assertions, duplicate the
gateway fixture, or retain the retired flat module through a compatibility mount.

## 修复结果与回传

Open: `source_split_complete / managed_behavior_validation_pending`.
Coordinator ownership transfer
`502b6a2803034ecfae94faa5bf5c94e4` assigned the flat file and replacement directory
to the stable Editor03 Session using fingerprint
`c279fb098d3e03bc3d983e3ecb7758b8ac47849fa3b4d6ac67fdae39c60597bf`.
The user has deferred `zr_vm`; existing Cargo topology admission and dependency
blockers are retained without repeated submissions.

The flat file is removed. The replacement has a seven-line declaration root,
six behavior leaves and one 76-line gateway fixture owner; the largest leaf is
218 lines. Original test groups are preserved as gateway recovery (4), selection
exhaustion (2), Play history (1), delete history (4), hierarchy (4), and multi-delete
(5). All 20 test bodies and the complete gateway fixture match the pre-edit HEAD
after Rust 2024 rustfmt normalization, apart from the fixture's required
`pub(super)` visibility. Common editing fixtures are imported from their existing
owner. The current editing module document points to the new module root.

Local Rust formatting and the Editor03 scene-transaction plus EditorUI10 test-budget
source-contract suites passed: 18 tests, zero failures, 2.879 seconds. The current
audit reports 11 oversized tests, three oversized production files, two duplicate
test trees and zero budget exemptions. This is migration and source-contract
evidence only; the managed node-command, editing and upward structure Rust tests
have not passed. Independent review, fixed return, closeout and WeCom remain pending.

Snapshot `2846` freezes the eleven owned paths, including the old flat-file deletion.
Managed format request `failure-roll-01a07160-editor03-node-command-format-20260906-r1`
was accepted by coordinator request `a3af4d11ffce4e4b93a9d5bc8fb519f3` as ticket
`b83d47f916234e0a9ace638a217eefcd`, manifest
`0cc9935cea4313e083d961b8f4e5a966dc22e9d638df3d509b876584d1c62e11`.
The receipt is `queued` with `validation_dependency_failed`, including existing
navigation bake, viewport overlay and dynamic-component generation dependencies.
It has not executed; retain the ticket and continue independent work.
