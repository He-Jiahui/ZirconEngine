---
title: Runtime11 task lock-poison source-guard migration
category: zircon_runtime
report_id: Runtime11-task-lock-poison-guard-repair-2026-09-20
date: 2026-09-20
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_contract_repaired
---

# Runtime11 task lock-poison source-guard migration

## Scope

The Runtime15 structure guard still asserted the pre-cutover `JobStateInner`
and `lock_task` layout. Runtime task state now lives in the folder-backed
`TaskNode` owner, while pending scheduler work lives in
`job_scheduler/pending.rs`. The stale guard would fail as soon as the managed
Runtime test target compiled, despite the production poison-recovery helpers
being present.

## Repair

- Point the lock-poison guard at `job_handle/task_node.rs` and
  `job_scheduler/pending.rs` instead of removed state/lock names.
- Keep regression-test anchors in their actual owners
  (`job_handle/tests.rs` and `job_scheduler/tests.rs`).
- Split the JobSystem mirror anchors so `catch_unwind`/`Condvar` and the
  canonical node lifecycle are checked in `TaskNode`, not the delegating
  `JobHandle` root; move pending terminal anchors to `pending.rs`.
- Preserve the production-only direct-lock-unwrapping check across the
  delegating roots and folder-backed owners. No production task behavior or
  scheduling semantics changed.

## Local validation

- Exact Rustfmt check passes for the three edited Runtime test/guard owners.
- A source-anchor recheck covers `JobHandle`, `TaskNode`, `JobScheduler`,
  `PendingScheduledJob`, and their poison-regression tests with zero missing
  anchors.
- One final batched local Runtime audit covering the JobSystem, task-failure,
  receipt-hard-cut, and asset-pipeline contracts passes `14/14` tests in
  `18.218s`, with zero failures, errors, or skips.
- The final widened non-tooling Runtime/Editor source-model loader covers
  `945` contract files and passes `3955/3955` tests in `57.471s`, with zero
  failures, errors, load errors, or skips; its Cargo command strings are
  contract assertions and no Cargo process was launched.
- After the recent-record Rustfmt convergence, the same loader was rerun and
  passed `3955/3955` in `51.893s`, again with zero failures, errors, load
  errors, or skips.
- After the Runtime853 source/model contract was added, the widened loader was
  rerun in one process and passed `3959/3959` across `946` contract files in
  `37.988s`, with zero failures, errors, load errors, or skips.
- After the Runtime854 source/model contract was added, the latest widened
  loader passed `4038/4038` across `956` contract files in `198.091s`, with
  zero failures, errors, load errors, or skips.
- After the Runtime855 source/model contract was added, the latest widened
  loader passed `4042/4042` across `957` contract files in `125.129s`, with
  zero failures, errors, load errors, or skips.
- The current expanded source-contract loader (performance-or-contract
  filename filter, tooling/export/coordinator excluded) passes `4072/4072`
  across `962` files in `139.499s`, with zero failures, errors, load errors, or
  skips; the `957`/`4042` receipt remains pre-Editor856 historical context.
- `git diff --check` passes for the edited files.
- No managed Windows Cargo/Release validation command was started. Managed
  validation remains pending under the external dirty-worktree admission gate.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/tests/runtime_absorption/structure_convention/lock_poison_policy/core_runtime/task_profiling.rs` | `C900B0D3FE655D16ADE06BBD64073D411BBC722B2A64A0037D6E4B1E28A938E7` |
| `zircon_runtime/src/tests/runtime_absorption/job_system/inventory/task_model.rs` | `1B4E986E43FE37D3A3B1B13187DB0F63FE52339F2A70D107F872C603F3F26686` |
| `zircon_runtime/src/tests/runtime_absorption/job_system/mirror_docs.rs` | `6A4DDF1CC088D21D6ACCEB74CC7547BC1B625B7536C876079B95C9C56C0C0F87` |

## Acceptance boundary

Keep this record at `implementation_complete` / `managed_validation_pending`
until the owner-attributed batched Windows lane compiles the current Runtime
tree and executes the Runtime15 structure guard plus the Runtime11 JobSystem
mirror suite. This repair is a test-contract correction, not product
performance acceptance.
