# Editor07 Pending Edit Page Capacity Optimization Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plan: `docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md`, E-PLAY-P1-32
- Status: implementation and release gate authored; batched managed validation pending

## Problem

Pending-edit pagination chained retry and pending entries, filtered by cursor,
and called `collect::<Vec<_>>()`. Because `Filter` reports a zero lower size
bound, the page result could grow progressively even though the product clamps
every page to at most 128 entries.

## Change

- Read the candidate iterator's upper size bound and clamp it to the existing
  page limit.
- Allocate the page buffer once with that bounded capacity, then push entries
  while retaining the same iterator for the existing next-page probe.
- Preserve zero allocation for an empty candidate set, the 1..=128 limit,
  retry-before-pending ordering, cursor filtering, entry payloads, and
  `next_cursor` semantics.

## Deterministic Performance Evidence

| Workload | Before | After | Reduction |
|---|---:|---:|---:|
| Full interactive page | capacity starts at 0 and grows progressively | capacity reserved once for 128 entries | intermediate page-buffer growth removed |
| Empty page | capacity 0 | capacity 0 | no empty-path regression |
| Maximum reserved entries | allocator-dependent growth up to 128 | exactly bounded by 128 | no unbounded reservation |

The ignored release gate runs 17 alternating sample pairs, each materializing
4,096 isolated 128-entry page buffers. Acceptance requires reserved-buffer
nearest-rank P95 to be at most 80% of progressive-growth P95, a minimum 20%
reduction. Exact Windows timing values remain pending the batched coordinator
run; the benchmark deliberately isolates page-buffer allocation from the
required per-entry operation-id projection.

## Acceptance

- `optimization_batch_20260826b_editor07_pending_edit_page_reserves_bounded_capacity`
  locks the production upper-bound reservation and explicit bounded push loop.
- `optimization_batch_20260826b_editor07_pending_edit_page_preserves_cursor_order`
  covers a 130-entry queue across the 128-entry boundary.
- `optimization_batch_20260826b_editor07_pending_edit_page_capacity_performance_evidence`
  emits `EDITOR07_PENDING_EDIT_PAGE_CAPACITY_BENCH_V1`, all raw samples, page
  dimensions, and the 20% P95 threshold.
- Exact-file Rust 1.94.1 rustfmt, source contracts, and scoped diff checks must
  pass before managed validation submission.

## Remaining Plan Work

This slice does not close Editor07. Product pending-edit routing, revision-safe
replay, typed Play transport, Game View, multi-instance authority, and the
large-scene/long-run Play performance matrix remain open.
