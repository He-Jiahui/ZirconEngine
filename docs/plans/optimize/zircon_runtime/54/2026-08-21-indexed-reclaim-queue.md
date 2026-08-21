# Runtime54 indexed reclaim queue

- Owner: `optimize-runtime54-indexed-reclaim-queue-r1-01a00797-20260821`
- Source plan: `54-runtime-scene-event-mirror-registration-subscription-cursor-backlog-overflow-reclaim-abi-consumer-product-integration-review.md`
- Finding: `SEMR-P2-006`
- Status: implementation and deterministic work evidence complete; combined managed Cargo validation pending

## Problem

`RuntimeEventMirrorReclaimQueue::retire_live_record` removed a queued handle with
`VecDeque::retain`. Retiring many subscriptions before the next World reclaim pass therefore scanned
the shrinking queue once per retirement. The reclaim intent was already deduplicated, but cancellation
was still quadratic in the pending handle count.

## Change

The queue now owns a handle-indexed doubly linked FIFO. A pending handle stores its previous and next
handles in a `HashMap`, while explicit head and tail handles preserve insertion order. Retirement
unlinks the indexed node in average constant time; drain walks the FIFO once and removes every index
entry. The live-record set remains a `BTreeSet`, the live-record hard budget is unchanged, duplicate or
stale enqueue remains a no-op, and diagnostics still count the same pending handles without relying on
their iteration order.

## Deterministic evidence

The release workload enqueues 4,096 live handles, then retires 2,048 alternating handles in reverse
order before draining. Legacy `retain` visits every surviving queue entry on each retirement; the
indexed queue performs one unlink per retired handle.

| Metric | Legacy retain queue | Indexed FIFO | Reduction |
| --- | ---: | ---: | ---: |
| Retirement work units | 6,292,480 inspections | 2,048 unlinks | 99.967% |
| Surviving drain order | 2,048 FIFO handles | Same handles and order | Exact equivalence |

The benchmark runs 21 alternating legacy/indexed sample pairs, emits raw timing arrays, and reports
independently recomputable nearest-rank P50/P95 values. The release gate requires indexed retirement
P95 and structural work to each use at most 25% of the legacy path. Timing values remain pending until
the post-Main Windows batch runs; no timing result is inferred from the structural model.

## Acceptance

- `indexed_reclaim_queue_unlinks_retired_handles_without_reordering_survivors` covers removal from the
  linked FIFO at tail, middle, and head positions and checks the exact surviving drain order.
- `runtime_event_mirror_indexed_reclaim_queue_release_benchmark` compares the legacy implementation
  with the production queue over 21 alternating pairs and enforces the 75% P95 threshold.
- The managed ten-task Runtime follow-up batch runs the Event Mirror regressions and ignored release
  gate together; this session launches no per-task Cargo process.

Pinned validation artifacts:

- Runtime54 child: `zircon-validation-runtime54-indexed-reclaim-queue.ps1`, SHA-256
  `C9AE83A19C7301AA863915BA25758588ADB29BE0EE82F1A742413257DAF522A7`.
- Ten-task Runtime batch: `zircon-validation-runtime-rust-followup-ten.ps1`, SHA-256
  `3CC7C1562643BA41677D42A76C46F8C733A9274854A7CE78D2EB7B8EA8AB6E0D`.
- Both scripts parse with zero PowerShell AST errors. Windows release timing, compilation, and test
  results remain pending until the post-Main materialized batch executes.

## Remaining scope

This closes only `SEMR-P2-006`. Runtime54's three P0 blockers, 60 P1 findings, and remaining 15 P2
findings stay open, including product event exposure, acknowledged delivery/resync, shared encoding,
global budgets, and bounded producer fanout. This local reclaim optimization does not represent
acceptance of the broader Event Mirror architecture.
