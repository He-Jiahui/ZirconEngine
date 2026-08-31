---
title: Editor13 Shell Repair Borrowed Hash Admission
category: zircon_editor
report_id: Editor13-shell-repair-borrowed-hash-admission-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor13 Shell Repair Borrowed Hash Admission

## Scope

This slice removes ordered membership and duplicate ID cloning from builtin shell layout repair.
Drawer and main-page tabs remain published in baseline order; the present-instance index is never
iterated to produce layout state.

## Change

- Replace the shell-repair `BTreeSet<ViewInstanceId>` indexes with `HashSet<ViewInstanceId>`.
- Add one shared admission helper that probes by borrowed instance ID before cloning into the set.
- Reuse the helper for both activity-drawer and main-document repair.
- Keep host collection, baseline iteration, active-tab selection, and drawer configuration order
  unchanged.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique instances | Before | After | Reduction |
|---|---:|---:|---:|
| Set-owned ID clones | 65,536 | 8,192 | 87.5% |
| Membership class | O(log n) | average O(1) | n/a |
| Published tab order | baseline order | baseline order | unchanged |

The ignored release gate runs 17 alternating samples and emits
`EDITOR13_SHELL_REPAIR_HASH_ADMISSION_BENCH_V1`. Acceptance requires borrowed hash admission P95
to be at most 60% of ordered-set admission P95. Exact Windows timings remain pending the
coordinator run.

## Acceptance

- `optimization_batch_20260826x_editor13_shell_repair_hash_admission_preserves_first_seen_order`
  verifies duplicate suppression and first-seen output through the production helper.
- `optimization_batch_20260826x_editor13_shell_repair_uses_borrowed_hash_admission` requires both
  production insertion sites to use the shared borrowed hash boundary.
- `optimization_batch_20260826x_editor13_shell_repair_hash_admission_performance_evidence` checks
  admission equivalence, reports clone counts and both P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

Editor13 still needs atomic layout restore/reset, dirty-document decision barriers, complete schema
validation and migration, unknown-plugin placeholders, last-good recovery, and monitor-aware window
placement. This slice only reduces builtin shell repair overhead.
