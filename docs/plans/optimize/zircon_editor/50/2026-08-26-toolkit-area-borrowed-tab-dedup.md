# Editor50 Toolkit Area Borrowed Tab Deduplication Record

- Date: 2026-08-26
- Owner: `root-runtime-events-20260824`
- Source plan: Editor50 descriptor scale qualification, including E-EXT-P1-46 and E-EXT-P2-14
- Status: implementation and release gate authored; batched managed validation pending

## Problem

Every valid `ToolkitArea` declaration cloned all owned tab IDs, sorted the duplicate copy, and
scanned adjacent values solely to detect duplicates. Large extension-generated layouts therefore
paid N string allocations, all tab bytes copied, and O(N log N) comparison work before preserving
the original declaration order in the final area.

## Change

- Build one capacity-sized `HashSet<&str>` over the already-owned input tab vector.
- Detect duplicates and active-tab membership through borrowed keys.
- Preserve declaration order in the final `Arc<[String]>`.
- Preserve the legacy error detail when multiple IDs are duplicated by returning the
  lexicographically smallest duplicate ID.

## Deterministic Performance Evidence

| 32,768 unique tab IDs | Before | After | Reduction |
|---|---:|---:|---:|
| Temporary tab `String` clones | 32,768 | 0 | 100% removed |
| Temporary tab text bytes copied | 1,048,576 | 0 | 100% removed |
| Duplicate detection | O(N log N) sort | average O(N) hash admission | sort removed |
| Final declaration order | input order | input order | unchanged |

The ignored release gate alternates 17 clone/sort and borrowed-hash samples. It emits
`EDITOR50_TOOLKIT_AREA_BORROWED_TAB_DEDUP_BENCH_V1`; acceptance requires borrowed P95 to be at most
60% of legacy P95. Exact Windows timings remain pending the batched coordinator run.

## Acceptance

- `optimization_batch_20260826g_editor50_borrowed_tab_dedup_preserves_order_and_errors` covers
  valid order, active-tab lookup, and duplicate reporting.
- `optimization_batch_20260826g_editor50_toolkit_area_uses_borrowed_hash_dedup` requires the
  capacity-sized borrowed hash set and rejects the clone/sort path.
- `optimization_batch_20260826g_editor50_borrowed_tab_dedup_performance_evidence` emits both P95
  values, clone/byte counts, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

## Remaining Parent-plan Work

This slice does not close extension snapshot cloning, owner generation, transactional mount,
callback isolation, query pagination, or 100/1K/10K complete contribution qualification.
