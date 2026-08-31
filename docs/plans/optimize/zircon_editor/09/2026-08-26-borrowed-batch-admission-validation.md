---
title: Editor09 Borrowed Batch Admission Validation
category: zircon_editor
report_id: Editor09-borrowed-batch-admission-validation-2026-08-26
date: 2026-08-26
session_id: root-editor09-borrowed-batch-admission-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor09 Borrowed Batch Admission Validation

## Scope

This slice removes the temporary request-reference `Vec` created inside the pending-admission
ledger when a prepared reservation batch is validated. It does not change admission limits,
reservation identity, request ownership, validation order, age or byte accounting, error types,
atomic commit and rollback behavior, public APIs, or the existing slice-based forwarding contract.

The adjacent `pending.rs` forwarding path already has unrelated active work and is deliberately not
part of this candidate. That path still materializes one request-reference vector before the first
preflight. This slice reduces the full reservation call from two temporary reference-vector
allocations to one without editing or depending on that active file.

## Change

- `PendingAdmissionLedger::reserve_batch` now validates `reservations.iter().map(...)` directly.
- A private cloned exact-size iterator helper preserves the existing two-pass age and byte checks.
- The existing `&[&EditorJobAdmissionRequest]` method delegates through `iter().copied()`, so callers
  and the dirty forwarding layer retain the same signature and behavior.
- Existing Rust tests remain the behavior oracle for atomic rejection, held capacity, Drop release,
  and shutdown release. A Python source contract prevents the ledger-local collector from returning.

## Deterministic Performance Evidence

The independent release model validates 65,536 batches of 16 requests. Both variants execute the
same two validation passes and produce checksum `9,198,632,960`; only the old variant first
collects the request references into a heap vector. Each run contains 21 alternating samples.

| Evidence | Collected references | Borrowed iterator | Result |
| --- | ---: | ---: | ---: |
| Measured allocations | 65,536 | 0 | 100% fewer ledger-local allocations |
| Run 1 P50 | 7.195 ms | 2.251 ms | 68.718% faster |
| Run 1 P95 | 41.867 ms | 3.673 ms | 91.227% faster |
| Run 2 P50 | 7.749 ms | 2.473 ms | 68.080% faster |
| Run 2 P95 | 16.730 ms | 7.138 ms | 57.338% faster |
| Run 3 P50 | 7.024 ms | 2.246 ms | 68.028% faster |
| Run 3 P95 | 17.476 ms | 6.233 ms | 64.335% faster |

The managed gate requires exact allocation counts of 65,536 and 0, an identical checksum, at least
50% P50 improvement, and at least 25% P95 improvement.

## Acceptance

- TDD RED observed two missing borrowed-iterator contract failures while the existing Rust behavior
  oracle check passed.
- `tools.tests.test_editor09_borrowed_batch_admission_performance_contract` passes 3/3 locally.
- Exact production/model `rustfmt --check`, model compilation, three model runs, and scoped diff
  checks pass locally.
- The four reservation behavior tests, source contracts, formatting, performance model, and scoped
  diff checks are submitted together in one coordinator validation ticket with one Cargo command.
- Commit integration and automatic WeCom performance notification remain gated on managed
  validation and the repository's independent-review policy.

## Remaining Parent-plan Work

Editor09 still owns bounded lifecycle history, retry and persistence policy, dependency failure
semantics, plugin lifecycle, product shutdown barriers, resource-vector admission, async observer
fan-out, and long-session stress qualification. The adjacent first-preflight reference
materialization remains available for its current owner to remove after that active file converges.
