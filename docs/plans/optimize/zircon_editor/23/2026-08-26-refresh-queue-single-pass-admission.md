# Editor23 Refresh Queue Single-pass Admission Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plan: Editor23 large-asset authoring qualification and Editor asset-refresh product path
- Status: implementation and release gate authored; batched managed validation pending

## Problem

Every refresh enqueue first collected incoming asset IDs into a temporary `BTreeSet`, then moved
them one by one into the persistent pending `BTreeSet`. A large source/import notification batch
therefore paid two complete ordered-tree admissions and allocated an intermediate tree whose order
was not consumed separately.

## Change

- Admit each incoming owned asset ID directly into the persistent pending tree.
- Remove any deferred retry for that ID in the same pass.
- Track whether the input produced any item so empty batches remain no-ops.
- Preserve one generation increment for every non-empty enqueue, duplicate coalescing, sorted
  request IDs, deferred-retry cancellation, and active request behavior.

## Deterministic Performance Evidence

| 32,768 changed asset IDs | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-tree admissions | 65,536 | 32,768 | 50% removed |
| Intermediate tree nodes | 32,768 | 0 | 100% removed |
| Persistent pending nodes | 32,768 | 32,768 | unchanged |
| Request order | ascending unique | ascending unique | unchanged |

The ignored release gate alternates 17 double-tree and single-pass samples. It emits
`EDITOR23_REFRESH_QUEUE_SINGLE_PASS_ADMISSION_BENCH_V1`; acceptance requires single-pass P95 to be
at most 60% of legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826n_editor23_single_pass_enqueue_preserves_queue_semantics` covers
  duplicate input, generation, deferred cancellation, pending count, and sorted request output.
- `optimization_batch_20260826n_editor23_enqueue_admits_directly_to_pending_tree` requires direct
  admission and rejects the temporary collect/extend path.
- `optimization_batch_20260826n_editor23_refresh_enqueue_single_pass_performance_evidence` emits
  admission/node counts and both P95 values, then enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

This slice does not close Editor23's dependency-generation authority, refresh job admission,
cancel/supersede receipts, source parse/compile budgets, session invalidation, large-document
virtualization, or complete 1k/10k/100k qualification.
