---
handoff_kind: failure
status: open
failure_scope: cross_plan
created_at: 2026-08-15
summary_slug: recovery-test-owner-threshold-drift
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/17
plan_link_mode: child_record_only
related_code:
  - tools/tests/test_editor17_recovery_test_ownership_contract.py
  - zircon_editor/src/core/recovery/mod.rs
  - zircon_editor/src/core/recovery/tests.rs
tests:
  - python -m unittest tools.tests.test_editor17_recovery_test_ownership_contract
---

# Editor17: recovery test owners exceed the structure threshold

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：MVP performance audit recovery ownership preflight
- 修复责任计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 交接原因：Editor17 owns the recovery test split; Performance01 exposed the threshold drift but does not own the foreign Rust test files.

## 失败现象与复现证据

The current recovery ownership contract ran deterministically on 2026-08-15: 3 tests, 1 passed and
2 failed. `zircon_editor/src/core/recovery/tests.rs` is 810 lines and
`zircon_editor/src/core/recovery/tests/autosave_adapter.rs` is 1,023 lines; the contract requires
every named owner to remain at or below 800 lines.

Per-file `rustfmt --edition 2021 --check` independently passed 17/20 recovery files. The only failures
are foreign current `mod.rs` and the same two oversized test owners. Formatting is therefore an
additional owner gate, not evidence that the performance documentation changed Rust source.

HEAD's root `tests.rs` is 752 lines. Current foreign work both modifies that file and introduces the
untracked adapter owner and the untracked contract. The adapter/support-symbol placement checks pass,
so the failure is not a missing `mod` declaration or an accidental move back into the root. The
lowest broken layer is feature ownership inside the two large test files.

## 最低共享层根因

Editor17 owns the split. Preserve `tests.rs` as the recovery/store/session/restore facade and divide
large behavior clusters into named folder-backed owners, for example scheduler/store/catalog/session
and adapter admission/completion/storage fixtures. Shared fixtures may move to a small support module
only when at least two owners use them.

The split must preserve the same production paths and test names/semantics. It must not duplicate
job systems, stores or fake a special autosave success path. Lower support fixtures remain shared;
focused owners import them and validation then runs upward through the complete recovery module.

Performance01 did not edit the foreign Rust tests or contract. This failure is independent of the
managed Cargo/build-helper blocker and must remain visible while non-validation architecture work
continues.

## 架构修复验收

- The Python ownership contract passes all 3 tests, with every named recovery test owner at or below
  800 physical lines.
- `rustfmt --edition 2021 --check` passes all current recovery Rust files after the ownership split.
- No recovery/autosave test is removed, ignored or weakened; test inventory remains at least the
  current 54 `#[test]` functions.
- Focused scheduler/store/catalog/session and adapter admission/completion suites pass through the
  normal production code paths.
- Current managed `zircon_editor` recovery Cargo tests pass from a non-C target root after the
  approved-root build-helper failure is repaired by its owner.

## 禁止临时方案

- Do not raise/remove the 800-line threshold or hide tests behind ignored/default-off features.
- Do not delete assertions, merge unrelated fixtures into one generic helper or create a
  test-specific production API merely to reduce line count.
- Do not mark this fixed from file movement alone; the Python contract and managed Rust suites must
  both pass.

## 修复结果与回传

Open state: Editor17 still needs to complete the owner split and make both the Python ownership contract and managed Rust recovery suites GREEN before returning this handoff as fixed.

## 2026-08-27 in-progress split metadata boundary

The deleted `tests/autosave_adapter.rs` leaf is removed from structured metadata.
Its folder-backed replacement remains foreign and untracked in the shared worktree,
so this record does not claim that replacement as an integrated owner. The tracked
ownership contract, recovery module, and `tests.rs` facade remain the durable anchors;
no Editor17 source bytes or acceptance state changed.
