---
title: Runtime74 Watch Invalidation Borrowed Hash Admission
category: zircon_runtime
report_id: Runtime74-watch-invalidation-borrowed-hash-admission-2026-08-26
date: 2026-08-26
session_id: root-runtime74-three-task-hash-streaming-batch-20260830
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime74 Watch Invalidation Borrowed Hash Admission

## Scope

This slice removes ordered membership sets from UI asset watch invalidation and avoids allocating
owned strings for duplicate admission attempts. It preserves change, rebuild, and removal vectors in
first-seen order. It does not close Runtime74's transactional tree replacement or state migration
work.

## Change

- Replace the three per-batch `BTreeSet<String>` membership indexes with `HashSet<String>`.
- Probe every set with borrowed `&str` identity before cloning an admitted asset ID.
- Continue publishing vectors directly from the incoming watch-change and cascade streams.
- Keep dependency traversal ordering and dependency-index ownership unchanged.

## Deterministic Performance Evidence

| Representative 65,536 admissions / 8,192 unique assets | Before | After | Reduction |
|---|---:|---:|---:|
| Ordered-set insert attempts | 65,536 | 0 | 100% removed |
| Hash membership probes | 0 | 65,536 | average O(1) |
| Owned string allocations | 73,728 | 16,384 | 77.8% removed |
| Published asset order | first-seen | first-seen | unchanged |

The ignored release gate runs 17 alternating samples and emits
`RUNTIME74_WATCH_INVALIDATION_HASH_ADMISSION_BENCH_V1`. Acceptance requires borrowed hash
admission P95 to be at most 60% of ordered-set admission P95. Exact Windows timings remain pending
the coordinator run.

## Acceptance

- `runtime74_batch_watch_hash_admission_preserves_first_seen_order`
  exercises duplicate change/removal handling through `apply_watch_changes`.
- `runtime74_batch_watch_invalidation_uses_borrowed_hash_admission`
  requires all three production hash boundaries and rejects ordered sets.
- `runtime74_batch_watch_hash_admission_performance_evidence` checks output
  equivalence, reports allocation counts and P95 values, and enforces the 60% threshold.
- Exact-file Rust 1.94.1 formatting, scoped diff checks, and source contracts must pass before
  managed validation submission.

The managed `runtime74_batch_` release gate seals this work together with the resource-reference
visitor and hot-reload slices in one Cargo invocation: three source contracts, nine Rust tests, and
three performance rows. Dynamic Windows marker values, commit attribution, and WeCom publication
remain pending the coordinator result.

## Remaining Parent-plan Work

Runtime74 still needs generation-qualified compiled endpoints, atomic tree replacement, component
state migration, binding reinstallation, old subscription retirement, and last-good rollback.
Product-scale dependency and live-surface qualification remain open.
