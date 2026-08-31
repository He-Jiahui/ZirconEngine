---
title: Editor23 Replay Artifact Borrowed Undo
category: zircon_editor
report_id: Editor23-replay-artifact-borrowed-undo-2026-08-26
date: 2026-08-26
session_id: root-runtime-events-20260824
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Editor23 Replay Artifact Borrowed Undo

## Scope

This slice removes deep undo-stack cloning from bug-report initial-source reconstruction. Reverse
transition order, source replay semantics, first-error reporting, current session state, undo/redo
availability, and exported artifact contents remain unchanged.

## Change

- Expose a crate-private borrowed reverse iterator over undo transitions.
- Reconstruct the initial source by applying those borrowed transitions directly.
- Remove both the full undo/redo stack clone and each `undo_record` transition clone.

## Deterministic Performance Evidence

| 2,048 edits, 1,024-byte effect payloads, four reconstructions per sample | Before | After |
|---|---:|---:|
| Full undo-stack clones per sample | 4 | 0 |
| Undo transition clones per sample | 8,192 | 0 |
| Effect payload bytes deeply cloned per sample | 25,165,824 | 0 |
| Source transitions applied per sample | 8,192 | 8,192 |

The ignored release gate runs 17 alternating sample pairs and emits
`EDITOR23_REPLAY_ARTIFACT_BORROWED_UNDO_BENCH_V1`. Acceptance requires borrowed reconstruction P95
to be at least 80% below stack cloning. Exact Windows timings remain pending the coordinator run.

## Acceptance

- `optimization_batch_20260826av_replay_artifact_borrowed_undo_preserves_initial_source` covers
  reverse reconstruction and proves the original undo/redo availability is not mutated.
- `optimization_batch_20260826av_replay_artifact_borrows_undo_transitions` requires borrowed reverse
  iteration and rejects stack/record cloning in the owned reconstruction path.
- `optimization_batch_20260826av_replay_artifact_borrowed_undo_p95` reports paired P50/P95 samples
  and enforces the 80% P95 reduction gate.

## Remaining Parent-plan Work

Editor23 still owns schema-backed preview data, typed diagnostics, incremental validation, preview
fidelity, bindings, transactions, cook artifacts, and large-asset gates. This slice only converges
bug-report initial-source reconstruction.
