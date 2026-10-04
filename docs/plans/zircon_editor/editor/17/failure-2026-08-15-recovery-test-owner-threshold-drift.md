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
  - zircon_editor/src/core/recovery/tests/autosave_adapter/mod.rs
  - zircon_editor/src/core/recovery/tests/autosave_adapter/admission.rs
  - zircon_editor/src/core/recovery/tests/autosave_adapter/completion.rs
  - zircon_editor/src/core/recovery/tests/autosave_adapter/outcomes.rs
  - zircon_editor/src/core/recovery/tests/autosave_adapter/scheduling.rs
  - zircon_editor/src/core/recovery/tests/autosave_adapter/shutdown.rs
  - zircon_editor/src/core/recovery/tests/autosave_adapter/support.rs
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

## 2026-09-06 current-source folder-backed ownership evidence

The old `tests/autosave_adapter.rs` path is absent from the current source. Its behavior is now
owned by the tracked folder-backed module with seven files under
`zircon_editor/src/core/recovery/tests/autosave_adapter/`; the root recovery facade is 65 lines
and `tests.rs` is 591 lines. The current ownership contract was run against this source state:

```text
python -m unittest tools.tests.test_editor17_recovery_test_ownership_contract
....
Ran 4 tests in 0.101s
OK
```

The seven adapter owners and the recovery facade were checked with the current structure audit;
the threshold failure described by the original record is no longer present. The original flat
path remains in the handoff history as evidence and is not recreated. Managed rustfmt and the
focused recovery Cargo suites remain required; this static result does not claim those gates or
fixed return.

### 2026-09-06 coordinator static ticket

- Request: `failure-roll-01a07160-editor17-static-20260906-r2`
- Ticket: `c4aea5744f9a47bb893e64fcc0dba685`
- Command: `python -m unittest tools.tests.test_editor17_recovery_test_ownership_contract`
- Coverage: focused static gate with dependency roots `tools` and
  `zircon_editor/src/core/recovery`.
- Admission: queued, with `validation_dependency_failed` blockers from the open Editor17
  prerequisite chain (including the Editor14 autosave-job admission, Editor16 project-session
  lock reuse, Editor00 core-root facade, and Editor09 import-diagnostics handoffs).

The coordinator receipt is retained as evidence of a valid ticket and dependency routing. It is
not a GREEN result and no fixed return, commit, or notification is claimed for this failure.

## 2026-09-11 rolling repair continuation

- Stable Session `failure-roll-01a090ae-editor17-autosave-budget-r1` owns only the seven
  folder-backed autosave test owners and this failure record. The foreign
  `zircon_editor/src/core/recovery/mod.rs` rustfmt-only import reorder is outside the scope;
  root `tests.rs` and the ownership-contract test remain unchanged historical inputs.
- The seven autosave files match their archived baseline-599 attribution hashes with zero drift,
  and the former flat `tests/autosave_adapter.rs` path is absent. Current source check:

  ```text
  py -3 -B -m unittest tools.tests.test_editor17_recovery_test_ownership_contract -v
  ....
  Ran 4 tests in 0.029s
  OK
  ```

  This is static evidence only; it does not establish a Cargo or product gate.
- The old queued ticket `c4aea5744f9a47bb893e64fcc0dba685` belongs to archived Session
  `failure-roll-01a07160-editor17` and is not reused. A fresh current-source ticket is required;
  full recovery rustfmt and the foreign `recovery/mod.rs` edit remain outside this slice.
- Fresh coordinator static ticket `21abd6606f8d413e8fe802da3ba84ba3` sealed the current eight-path
  scope and is `queued` with the existing Editor17 prerequisite-chain blockers. The focused Cargo
  request `failure-roll-01a090ae-editor17-cargo-20260911-r1` was rejected before ticket creation
  or Cargo execution with `validation_ticket_external_worktree_dirty` for `E:\Git\zr_vm`; no
  compile/test result is available. Session status remains `waiting_validation`, this failure
  remains `open`, and no fixed return or closeout is claimed.
