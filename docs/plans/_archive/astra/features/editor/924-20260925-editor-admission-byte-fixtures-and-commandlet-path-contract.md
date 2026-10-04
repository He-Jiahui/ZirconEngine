---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/261-editor-command-registry-keymap-menu-palette-commandlet-operation-current-working-tree-review.md
related_records:
  - docs/plans/astra/features/editor/923-20260924-editor-play-decision-hook-deadlock-and-resource-fixture-repairs.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
---

# Editor924 Admission Byte Fixtures and Commandlet Path Contract

## 计划完成列表

| Scope | Completed change | Evidence and remaining gate |
| --- | --- | --- |
| Interactive save admission fixtures | Give the running blocker and the queued entry-test occupant explicit one-byte estimates; the separate pending-byte test still admits twelve bytes and requests sixteen. | Both v30 save-adapter tests failed. Their earlier unbounded blocker (and the first test's queued occupant) inherited `EditorJobSpec::new`'s 4,096-byte estimate despite 1,024/16-byte admission budgets, preventing the intended admission error and no-materialization assertions from being reached. No production estimate, adapter logic or expected error was changed. Rust regression pending. |
| Editor Job reservation, merging and cancellation fixtures | Give one accepted reservation its declared eight-byte estimate and set only the low-budget running blockers to one byte across the system submission test and the keyed/entry/byte/age admission tests. | Ten related v30 Job tests reported `FAILED`; source review shows the 4,096-byte default exceeds the corresponding 8/32-byte fixture budgets before the intended merge, cancellation, capacity or wait-age behavior can run. All real admissions and expected errors retain their original bounds; current-source Rust tests pending. |
| Job event-journal gap fixture | Budget the retained and between-gap lifecycle events by their respective `estimated_retained_bytes()`, plus the gap, instead of assuming equal-size labels. | The grouped v32 Jobs batch failed `merged_gap_absorbs_retained_events_between_dropped_sequences`: `between-gaps` is longer than `retained`, so the old two-identical-event budget evicted sequence 1 before the asserted pop. The count/byte limit and production gap-merging behavior are unchanged; this post-seal test fix needs a later current-source Rust run. |
| Commandlet managed-target fixture | Build the test fixture from `commandlet_test_target_root()` and assert its actual parent and containment in the coordinator's `CARGO_TARGET_DIR`. | v30's path test reported `FAILED`; the former test appended directly below a fabricated target root but asserted an additional subdirectory that `fixture_path` never creates. The real helper already validates an absolute D:/E:/F: managed root. No Commandlet production helper or filesystem contents changed; current-source Rust test pending. |

The previously red v30 Editor library test was interrupted before a harness
summary, so those lines establish neither a complete failure inventory nor a
passing repair. Exact-file `rustfmt --edition 2021 --check` and scoped
`git diff --check` passed for all six touched Rust test files. The
read-only four-module Editor admission/save source-contract batch passed
`13/13`; it cannot prove that the Rust fixture regressions pass. The
coordinator-managed Editor library check for sealed digest
`2b1c4bace6ec04f2ca494fdd3f2bdba2c5a4e920b629954f70f6298a736cf52c`
reported `[OK]`; the associated `core::jobs` filtered library tests finished
with `138 passed, 2 failed, 15 ignored, 8819 filtered out` and Cargo test
exit `101` (receipt `2363a129ca9d4a41ba861cde933568c4`). The two
failures are the event-journal fixture above and the production-thread-ownership
guard, which correctly names `core/export/stages/compile_host.rs` and
`core/play/process_backend/output.rs`. The earlier ten admission fixture
failures did not recur, but a failing batch is not an accepted pass. The
thread guard has **not** been weakened: the long-lived process-pipe readers
need a lifecycle/ownership-aware migration and validation. The new journal
fix and logging hook edit were made after this sealed digest. Save Adapter
and Commandlet Rust tests are outside that filter. The focused regression, ignored Windows
Release performance and product p50/p95/p99 gates remain open; a passing
library check and source contracts alone do not establish performance.

The Autosave admission fixture with the same low-budget/default-estimate
pattern is already foreign-modified in
`zircon_editor/src/core/recovery/tests/autosave_adapter/admission.rs`. It was
not changed here; its owner must reconcile it before the whole recovery suite
can pass. No Python tooling was changed, and neither the conflicting Editor207
grid storage contracts nor unrelated UI layout assertions were weakened.
