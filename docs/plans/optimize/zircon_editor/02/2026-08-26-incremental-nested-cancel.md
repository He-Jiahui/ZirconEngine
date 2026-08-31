---
title: Editor02 Incremental Nested Transaction Cancellation
category: zircon_editor
report_id: Editor02-incremental-nested-cancel-2026-08-26
date: 2026-08-26
session_id: root-editor02-incremental-nested-cancel-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor02 Incremental Nested Transaction Cancellation

## Scope

This slice removes the temporary tail `Vec` created when an outer transaction scope cancels its
nested descendants. It does not change target-scope validation, reverse cancellation order,
command revert/finalize behavior, root cancellation events, edit-context ownership, the operation
gate, fault typing, or the rule that successfully canceled descendants stay removed when a later
frame fails to revert.

The adjacent `dirty_batch.rs` and `scope.rs` files already contain unrelated active work and are
deliberately outside this candidate.

## Change

- `EditorTransactionEngine::cancel` validates the requested scope before taking the edit context.
- It then pops one frame at a time from the authoritative active stack instead of draining the tail
  into `collect::<Vec<_>>()`.
- No engine mutex crosses a command callback. Each pop uses a short lock and cancellation work runs
  after the guard is dropped.
- On failure, only the current frame is pushed back. Unprocessed lower frames never left the active
  stack, while already canceled descendants remain removed exactly as before.
- A typed invariant failure restores the context and faults the engine if the validated target
  unexpectedly disappears while the operation gate is held.
- A Python source contract prevents tail materialization from returning and pins the existing
  nested-success and revert-failure Rust behavior oracles.

## Deterministic Performance Evidence

The independent release model uses 12 nested frames. Each sample performs 32,768 cancellations;
each canceled frame executes the same 384 arithmetic steps to represent non-empty command work.
Twenty-one samples are paired and alternate baseline/optimized order to reduce time-order bias.

| Evidence | Tail `Vec` baseline | Incremental pop | Result |
| --- | ---: | ---: | ---: |
| Allocations per cancellation | 1 | 0 | 100% removed |
| Run 1 P50 | 130.684 ms | 130.425 ms | 0.199% faster |
| Run 1 P95 | 194.624 ms | 153.250 ms | 21.259% faster |
| Run 2 P50 | 125.492 ms | 123.384 ms | 1.680% faster |
| Run 2 P95 | 195.541 ms | 154.428 ms | 21.026% faster |
| Run 3 P50 | 127.071 ms | 120.037 ms | 5.535% faster |
| Run 3 P95 | 192.797 ms | 198.833 ms | 3.131% slower |

The implementation trades one tail allocation for one short mutex acquisition per canceled frame.
The model includes that cost. The managed gate requires exact allocation counts of 1 and 0, no P50
regression, and optimized P95 no more than 15% above baseline. All three local runs meet those
targets; the third run records the observed scheduler-sensitive P95 variance rather than hiding it.

## Acceptance

- TDD RED observed two missing incremental-cancel contract failures while the existing behavior
  oracle check passed.
- `tools.tests.test_editor02_incremental_nested_cancel_performance_contract` passes 3/3 locally.
- Exact production/model `rustfmt --check`, Python compilation, PowerShell parsing, three paired
  model runs, and scoped diff checks pass locally.
- The complete transaction-engine Rust test group, source contracts, formatting, performance model,
  and scoped diff checks are submitted together in one coordinator validation ticket with one Cargo
  command.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

Editor02 still owns the unified close/save coordinator, byte-budgeted history, durable journal and
replay, product fault recovery, autosave payload admission, residual-session recovery UX,
source-control/CAS behavior, and large-project fault-injection qualification.
