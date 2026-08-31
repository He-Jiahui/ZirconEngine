# Runtime43 Borrowed World Invalidation Page Probe Optimization Record

- Date: 2026-08-27
- Owner: `root-runtime43-borrowed-invalidation-page-probes-20260827`
- Source plan: `docs/plans/optimize/zircon_runtime/43-dynamic-runtime-session-registry-ffi-frame-event-extract-host-request-world-sync-ui-shader-prewarm-product-integration-review.md`, DYN-P1-057 / DYN-GATE-038
- Status: implementation and isolated release-model evidence complete; managed Cargo validation pending

## Problem

When the maximum world-invalidation page exceeded the encoded-byte budget,
page sizing rebuilt an owned `Vec<InvalidationBatch>` for the minimum page and
for every binary-search candidate. Each rebuild allocated the page and token
vectors and cloned every included `WorldFact`, including owned resource strings,
while the dynamic session owner remained borrowed.

## Change

- Candidate probes now serialize `BorrowedWorldInvalidationPage` directly from
  the private reversed tail queue.
- `BorrowedWorldInvalidationItems` emits each borrowed token or fact in reverse
  iterator order, preserving the canonical public ordering without a probe-time
  collection.
- Binary search retains only the best item limit and encoded bytes.
- The final owned page used by commit/rollback is materialized at one shared
  exit after page sizing completes.
- A Rust unit contract compares borrowed and owned JSON bytes for every partial
  page boundary across both dirty tokens and facts.

## Performance Evidence

Isolated `rustc -O` model:
`.codex/state/session-coordinator/runtime43-borrowed-page-probe-model.rs`.
The workload uses 128 batches and the same 13-candidate sizing pattern for both
implementations. The legacy path materializes every candidate; the optimized
path borrows every candidate and materializes the accepted page once.

| Metric | Before | After | Reduction |
|---|---:|---:|---:|
| Allocation calls | 9,217 | 762 | 91.73% |
| Allocated bytes | 986,919 | 81,440 | 91.75% |
| P50 | 1,112,300 ns | 85,800 ns | 92.29% |
| P95 | 3,280,500 ns | 114,000 ns | 96.52% |

The model checksum is identical (`12,335,709`). These measurements isolate
candidate projection and clone cost; they do not claim end-to-end ABI latency.

## Validation

- `python -m unittest tools.tests.test_runtime43_borrowed_world_invalidation_page_probe_performance_contract`: 3/3 passed.
- Exact-file `rustfmt`: passed.
- Standalone release model compiled with `rustc 1.94.1 -O`: passed.
- Managed Cargo regression remains pending in a later combined coordinator
  batch. Current coordinator materialization is globally blocked by foreign
  unmanaged Tooling15 build roots; no validation ticket is replayed here.

## Remaining Scope

Encoding is still synchronous under the dynamic session owner, and the public
page still lacks remaining/backlog, cursor, dropped, wake, and resync receipts.
Those remain separate Runtime43 DYN-P1-057 tasks.
