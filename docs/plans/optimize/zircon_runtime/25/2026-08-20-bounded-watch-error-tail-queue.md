# Runtime25 Bounded Watch Error Tail-Queue Record

- Date: 2026-08-20
- Owner: `optimize-runtime25-watch-error-tail-queue-r1-01a00797-20260820`
- Source plan: `docs/plans/optimize/zircon_runtime/25-watch-error-tail-queue.md`
- Status: implementation and release-gate definition complete; combined managed validation pending

## Problem

The project watcher activation retained at most 64 errors in a `Vec`. Every
overflow removed index zero while holding the activation mutex, moving all 63
retained error records before admitting the next error.

## Change

- `ProjectWatcherActivationState.errors` is now a `VecDeque`.
- Overflow uses `pop_front()` and admission uses `push_back()`.
- The existing overflow side effect remains unchanged: the partial watch state
  is marked for reconciliation.
- `take_work` retains FIFO iteration and moves the queue out without cloning
  error payloads.

## Deterministic Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| 200,000 admissions, capacity 64 | 12,595,968 record moves | 0 record moves | 100% |
| Overflow admission | O(capacity) | O(1) amortized | one complexity class |

The move count is deterministic: `(200,000 - 64) * (64 - 1)`.

## Acceptance

- `activation_error_overflow_discards_oldest_and_preserves_fifo_order` locks
  capacity, oldest eviction, FIFO publication, and reconciliation.
- `watch_error_tail_queue_release_benchmark_evidence` emits 21 alternating
  sample pairs and raw nanosecond CSV for both variants.
- Managed child validator:
  `.codex/state/session-coordinator/cargo-runs/zircon-validation-runtime25-watch-error-tail-queue.ps1`,
  SHA256 `9A60784BBA1D4D04183C8B9340D7F20EADDEA8EDD356EF499ADB1F5159BB6789`.
- The child validator recomputes nearest-rank P50/P95 and requires optimized
  P95 to be no more than 75% of legacy P95.
- Exact-file Rustfmt, scoped diff, Cargo regression, and measured P50/P95 are
  pending the combined managed validation batch; no per-task Cargo is used.

## Remaining Scope

This record does not close Runtime25's broader watch mapping, mount generation,
filesystem provider, secure-open, or I/O scheduling findings.
